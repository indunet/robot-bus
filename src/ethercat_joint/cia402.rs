//! CiA402 / DS402 statusword & controlword helpers and enable state machine.

/// Statusword bits (0x6041).
pub mod status {
    pub const READY_TO_SWITCH_ON: u16 = 1 << 0;
    pub const SWITCHED_ON: u16 = 1 << 1;
    pub const OPERATION_ENABLED: u16 = 1 << 2;
    pub const FAULT: u16 = 1 << 3;
    pub const VOLTAGE_ENABLED: u16 = 1 << 4;
    pub const QUICK_STOP: u16 = 1 << 5;
    pub const SWITCH_ON_DISABLED: u16 = 1 << 6;
}

/// Controlword bits / commands (0x6040).
pub mod control {
    pub const SWITCH_ON: u16 = 1 << 0;
    pub const ENABLE_VOLTAGE: u16 = 1 << 1;
    pub const QUICK_STOP: u16 = 1 << 2;
    pub const ENABLE_OPERATION: u16 = 1 << 3;
    pub const FAULT_RESET: u16 = 1 << 7;

    pub const SHUTDOWN: u16 = ENABLE_VOLTAGE | QUICK_STOP;
    pub const SWITCH_ON_CMD: u16 = SWITCH_ON | ENABLE_VOLTAGE | QUICK_STOP;
    pub const ENABLE_OP: u16 = SWITCH_ON | ENABLE_VOLTAGE | QUICK_STOP | ENABLE_OPERATION;
    pub const DISABLE_OP: u16 = SWITCH_ON | ENABLE_VOLTAGE | QUICK_STOP;
    pub const QUICK_STOP_CMD: u16 = SWITCH_ON | ENABLE_VOLTAGE;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveState {
    NotReady,
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    QuickStopActive,
    FaultReaction,
    Fault,
    Unknown,
}

impl DriveState {
    pub fn from_statusword(sw: u16) -> Self {
        if sw & status::FAULT != 0 {
            return Self::Fault;
        }
        // Mask for state bits commonly used in CiA402 state decode.
        let nibble = sw & 0x006F;
        match nibble {
            0x0000 => Self::NotReady,
            0x0040 => Self::SwitchOnDisabled,
            0x0021 => Self::ReadyToSwitchOn,
            0x0023 => Self::SwitchedOn,
            0x0027 => Self::OperationEnabled,
            0x0007 => Self::QuickStopActive,
            0x000F | 0x002F => Self::FaultReaction,
            _ => Self::Unknown,
        }
    }
}

/// Compute next controlword to progress toward Operation Enabled (or hold / fault reset).
pub fn next_controlword(statusword: u16, want_enabled: bool, pulse_fault_reset: bool) -> u16 {
    if pulse_fault_reset {
        return control::FAULT_RESET;
    }
    let state = DriveState::from_statusword(statusword);
    if !want_enabled {
        return match state {
            DriveState::OperationEnabled => control::DISABLE_OP,
            DriveState::Fault => 0,
            _ => control::SHUTDOWN,
        };
    }
    match state {
        DriveState::Fault => control::FAULT_RESET,
        DriveState::SwitchOnDisabled | DriveState::NotReady | DriveState::Unknown => {
            control::SHUTDOWN
        }
        DriveState::ReadyToSwitchOn => control::SWITCH_ON_CMD,
        DriveState::SwitchedOn | DriveState::QuickStopActive => control::ENABLE_OP,
        DriveState::OperationEnabled => control::ENABLE_OP,
        DriveState::FaultReaction => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enable_path() {
        let sw = status::SWITCH_ON_DISABLED;
        let mut cw = next_controlword(sw, true, false);
        assert_eq!(cw, control::SHUTDOWN);

        assert_eq!(
            DriveState::from_statusword(0x0021),
            DriveState::ReadyToSwitchOn
        );
        cw = next_controlword(0x0021, true, false);
        assert_eq!(cw, control::SWITCH_ON_CMD);

        cw = next_controlword(0x0023, true, false);
        assert_eq!(cw, control::ENABLE_OP);

        cw = next_controlword(0x0027, true, false);
        assert_eq!(cw, control::ENABLE_OP);
    }
}
