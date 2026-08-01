//! Apply `XboxJoyRumble` via gilrs force-feedback (Strong/Weak motors).

use anyhow::{Context, Result};
use gilrs::ff::{BaseEffect, BaseEffectType, Effect, EffectBuilder, Repeat, Replay, Ticks};
use gilrs::{GamepadId, Gilrs};
use crate::robot_bus_interface::msg::v1::XboxJoyRumble;

/// Active rumble effect handle (dropped / stopped on replacement).
pub struct ActiveRumble {
    effect: Effect,
}

impl ActiveRumble {
    pub fn stop(&self) {
        if let Err(e) = self.effect.stop() {
            log::debug!("stop rumble: {e}");
        }
    }
}

impl Drop for ActiveRumble {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start or replace rumble from a command message.
///
/// `left_motor` → Strong (low-frequency / large), `right_motor` → Weak (high-frequency / small).
/// Intensities outside 0..1 are clamped. Both zero clears rumble (`Ok(None)`).
pub fn apply_rumble(
    gilrs: &mut Gilrs,
    id: GamepadId,
    cmd: &XboxJoyRumble,
) -> Result<Option<ActiveRumble>> {
    let left = cmd.left_motor.clamp(0.0, 1.0);
    let right = cmd.right_motor.clamp(0.0, 1.0);
    if left == 0.0 && right == 0.0 {
        return Ok(None);
    }

    let gp = gilrs
        .connected_gamepad(id)
        .ok_or_else(|| anyhow::anyhow!("joy {id} disconnected"))?;
    if !gp.is_ff_supported() {
        anyhow::bail!("joy {id} does not support force feedback (unsupported on this OS/driver)");
    }

    let strong = (left * f32::from(u16::MAX)).round() as u16;
    let weak = (right * f32::from(u16::MAX)).round() as u16;

    let duration_ms = cmd.duration_ms;
    let play_ms = if duration_ms == 0 {
        1_000
    } else {
        duration_ms.max(1)
    };
    let scheduling = Replay {
        play_for: Ticks::from_ms(play_ms),
        ..Replay::default()
    };
    let repeat = if duration_ms == 0 {
        Repeat::Infinitely
    } else {
        Repeat::For(Ticks::from_ms(play_ms))
    };

    let effect = EffectBuilder::new()
        .add_effect(BaseEffect {
            kind: BaseEffectType::Strong {
                magnitude: strong,
            },
            scheduling,
            ..BaseEffect::default()
        })
        .add_effect(BaseEffect {
            kind: BaseEffectType::Weak { magnitude: weak },
            scheduling,
            ..BaseEffect::default()
        })
        .gamepads(&[id])
        .repeat(repeat)
        .finish(gilrs)
        .context("create rumble effect")?;
    effect.play().context("play rumble effect")?;

    Ok(Some(ActiveRumble { effect }))
}

#[cfg(test)]
mod tests {
    #[test]
    fn magnitude_scaling() {
        let left = 0.5_f32;
        let strong = (left * f32::from(u16::MAX)).round() as u16;
        assert_eq!(strong, 32768);
    }
}
