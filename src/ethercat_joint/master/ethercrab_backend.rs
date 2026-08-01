//! ethercrab-backed EtherCAT master (Linux NIC + CAP_NET_RAW recommended).

use super::{EthercatMaster, JointFeedback, JointSetpoint, SlaveInfo};
use crate::ethercat_joint::cia402;
use crate::ethercat_joint::config::JointConfig;
use anyhow::{bail, Context, Result};
use ethercrab::subdevice_group::Op;
use ethercrab::{
    std::{ethercat_now, tx_rx_task},
    DefaultLock, MainDevice, MainDeviceConfig, PduStorage, SubDeviceGroup, Timeouts,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;

const MAX_SUBDEVICES: usize = 16;
const MAX_PDU_DATA: usize = 1100;
const MAX_FRAMES: usize = 16;
const PDI_LEN: usize = 256;

static PDU_STORAGE: PduStorage<MAX_FRAMES, MAX_PDU_DATA> = PduStorage::new();

type OpGroup = SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, DefaultLock, Op>;

/// Live EtherCAT master using [ethercrab](https://crates.io/crates/ethercrab).
///
/// Only one instance may be created per process (PDU storage is split once).
pub struct EthercrabMaster {
    iface: String,
    runtime: Runtime,
    maindevice: Option<Arc<MainDevice<'static>>>,
    inner: Option<Inner>,
    joints: Vec<JointConfig>,
    pulse_fault_reset: bool,
    slave_infos: Vec<SlaveInfo>,
    joint_to_group: Vec<usize>,
}

struct Inner {
    group: OpGroup,
}

impl EthercrabMaster {
    pub fn new(iface: &str) -> Result<Self> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("ethercat-joint")
            .build()
            .context("build tokio runtime for ethercrab")?;
        Ok(Self {
            iface: iface.to_string(),
            runtime,
            maindevice: None,
            inner: None,
            joints: Vec::new(),
            pulse_fault_reset: false,
            slave_infos: Vec::new(),
            joint_to_group: Vec::new(),
        })
    }
}

impl EthercatMaster for EthercrabMaster {
    fn configure(&mut self, joints: &[JointConfig]) -> Result<()> {
        if joints.is_empty() {
            bail!("ethercrab configure: no joints");
        }
        self.joints = joints.to_vec();

        let (tx, rx, pdu_loop) = PDU_STORAGE.try_split().map_err(|_| {
            anyhow::anyhow!(
                "ethercrab PDU storage already in use (only one EthercrabMaster per process)"
            )
        })?;

        let maindevice = Arc::new(MainDevice::new(
            pdu_loop,
            Timeouts {
                wait_loop_delay: Duration::from_millis(2),
                mailbox_response: Duration::from_millis(1000),
                ..Default::default()
            },
            MainDeviceConfig::default(),
        ));

        let iface = self.iface.clone();
        let task = tx_rx_task(&iface, tx, rx).map_err(|e| {
            anyhow::anyhow!(
                "open EtherCAT iface={iface}: {e} (need CAP_NET_RAW / root on Linux)"
            )
        })?;
        self.runtime.spawn(async move {
            if let Err(e) = task.await {
                log::error!("ethercrab TX/RX task ended: {e}");
            }
        });

        let md = Arc::clone(&maindevice);
        let joint_cfgs = joints.to_vec();
        let iface_name = self.iface.clone();
        let (group, slave_infos, joint_to_group) = self.runtime.block_on(async move {
            let group = md
                .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
                .await
                .map_err(|e| anyhow::anyhow!("ethercrab init group: {e}"))?;

            let mut infos = Vec::new();
            for sub in group.iter(&md) {
                infos.push(SlaveInfo {
                    configured_address: sub.configured_address(),
                    name: sub.name().to_string(),
                });
            }
            if infos.is_empty() {
                bail!(
                    "no EtherCAT subdevices found on iface={iface_name}; check cabling and permissions"
                );
            }

            for j in &joint_cfgs {
                let mut found = false;
                for sub in group.iter(&md) {
                    if sub.configured_address() == j.station_address {
                        sub.sdo_write(0x6060u16, 0u8, j.mode.modes_of_operation())
                            .await
                            .map_err(|e| {
                                anyhow::anyhow!(
                                    "SDO write Modes of Operation for {} @ {}: {e}",
                                    j.name,
                                    j.station_address
                                )
                            })?;
                        found = true;
                        break;
                    }
                }
                if !found {
                    bail!(
                        "joint {} station_address {} not found; seen: {:?}",
                        j.name,
                        j.station_address,
                        infos
                            .iter()
                            .map(|s| s.configured_address)
                            .collect::<Vec<_>>()
                    );
                }
            }

            let group = group
                .into_op(&md)
                .await
                .map_err(|e| anyhow::anyhow!("PRE-OP -> OP: {e}"))?;

            let mut joint_to_group = Vec::with_capacity(joint_cfgs.len());
            for j in &joint_cfgs {
                let mut idx = None;
                for (gi, sub) in group.iter(&md).enumerate() {
                    if sub.configured_address() == j.station_address {
                        idx = Some(gi);
                        break;
                    }
                }
                joint_to_group.push(idx.with_context(|| {
                    format!(
                        "joint {} address {} missing after OP",
                        j.name, j.station_address
                    )
                })?);
            }

            Ok::<_, anyhow::Error>((group, infos, joint_to_group))
        })?;

        self.slave_infos = slave_infos;
        self.joint_to_group = joint_to_group;
        self.maindevice = Some(maindevice);
        self.inner = Some(Inner { group });
        log::info!(
            "ethercrab online on {}: {} subdevice(s), {} joint(s)",
            self.iface,
            self.slave_infos.len(),
            self.joints.len()
        );
        Ok(())
    }

    fn list_slaves(&self) -> Vec<SlaveInfo> {
        self.slave_infos.clone()
    }

    fn set_want_enabled(&mut self, _enabled: bool) {}

    fn request_fault_reset(&mut self) {
        self.pulse_fault_reset = true;
    }

    fn cycle(&mut self, setpoints: &[JointSetpoint], feedback: &mut [JointFeedback]) -> Result<()> {
        let maindevice = self
            .maindevice
            .as_ref()
            .context("ethercrab not configured")?
            .clone();
        let inner = self.inner.as_mut().context("ethercrab not configured")?;
        let joints = &self.joints;
        let map = &self.joint_to_group;
        let pulse = self.pulse_fault_reset;
        self.pulse_fault_reset = false;

        self.runtime.block_on(async {
            inner
                .group
                .tx_rx(&maindevice)
                .await
                .map_err(|e| anyhow::anyhow!("ethercrab tx_rx: {e}"))?;

            for (ji, &gi) in map.iter().enumerate() {
                if ji >= setpoints.len() || ji >= feedback.len() {
                    break;
                }
                let joint = &joints[ji];
                let sub = inner
                    .group
                    .iter(&maindevice)
                    .nth(gi)
                    .context("subdevice index out of range")?;

                let mut io = sub.io_raw_mut();
                let statusword = read_u16(io.inputs(), joint.pdo.statusword);
                let actual = read_i32(io.inputs(), joint.pdo.actual);

                let cw = if pulse {
                    cia402::control::FAULT_RESET
                } else {
                    setpoints[ji].controlword
                };

                write_u16(io.outputs(), joint.pdo.controlword, cw);
                write_i32(io.outputs(), joint.pdo.target, setpoints[ji].target);

                feedback[ji] = JointFeedback {
                    actual,
                    statusword,
                    online: true,
                };
            }
            Ok(())
        })
    }

    fn shutdown(&mut self) {
        self.inner = None;
        self.maindevice = None;
    }
}

fn read_u16(buf: &[u8], off: usize) -> u16 {
    if off + 2 > buf.len() {
        return 0;
    }
    u16::from_le_bytes([buf[off], buf[off + 1]])
}

fn read_i32(buf: &[u8], off: usize) -> i32 {
    if off + 4 > buf.len() {
        return 0;
    }
    i32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

fn write_u16(buf: &mut [u8], off: usize, v: u16) {
    if off + 2 > buf.len() {
        return;
    }
    let b = v.to_le_bytes();
    buf[off] = b[0];
    buf[off + 1] = b[1];
}

fn write_i32(buf: &mut [u8], off: usize, v: i32) {
    if off + 4 > buf.len() {
        return;
    }
    let b = v.to_le_bytes();
    buf[off..off + 4].copy_from_slice(&b);
}
