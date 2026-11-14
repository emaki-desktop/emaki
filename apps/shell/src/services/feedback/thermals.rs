//! The computer running hot: a notification, kept in the center, when the
//! processor, graphics or SSD stays near its limit, so a slowdown or a loud
//! fan has a reason to see.
//!
//! The kernel can't say when an AMD or recent Intel laptop slows to cool
//! off (its firmware does that by itself, past any trip point the kernel
//! knows), so the hottest sensors are read every [`EVERY`] while the
//! session is in use, never while it's locked or idle: a handful of small
//! sysfs reads.
//!
//! - **Hot**: a CPU or GPU at [`HOT_CHIP`] or more, or an SSD within
//!   [`NEAR`] of its own critical limit, for two readings running. At most
//!   once every [`QUIET_FOR`].
//! - **Very hot**: any of them at its critical limit, or the CPU at
//!   [`CRITICAL_CHIP`]: critical, so it shows through Do Not Disturb.

use std::sync::Once;
use std::time::{Duration, Instant};

use amane::{Logind, Notify, Service, Urgency};
use emaki_kit::sensors::{Chip, Temperature, temperatures};
use emaki_kit::settings::{Feedback, notification_rules};

use super::notify;

/// How often the sensors are read while the session is in use.
const EVERY: Duration = Duration::from_secs(20);

/// A CPU or GPU this hot is near where it slows down.
pub const HOT_CHIP: f32 = 95.0;

/// A CPU this hot is at its limit.
pub const CRITICAL_CHIP: f32 = 100.0;

/// How close to its own critical limit a sensor counts as hot.
pub const NEAR: f32 = 3.0;

/// The least time between two warnings of the same level.
const QUIET_FOR: Duration = Duration::from_secs(15 * 60);

/// How hot one reading is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Heat {
    Fine,
    Hot,
    Critical,
}

/// Starts watching, once.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        std::thread::Builder::new()
            .name(String::from("emaki-thermals"))
            .spawn(|| {
                let mut before = Heat::Fine;
                let mut told: Option<(Heat, Instant)> = None;
                loop {
                    std::thread::sleep(EVERY);
                    if !in_use() || !notification_rules().shows(Feedback::Thermals) {
                        before = Heat::Fine;
                        continue;
                    }
                    let found = temperatures();
                    let (heat, hottest) = heat(&found);
                    if let Some(level) = due(before, heat, told, Instant::now())
                        && let Some(hottest) = hottest
                    {
                        warn(level, hottest);
                        told = Some((level, Instant::now()));
                    }
                    before = heat;
                }
            })
            .ok();
    });
}

/// The session is in front of someone: not locked, not idle.
fn in_use() -> bool {
    let logind = Logind::read();
    !logind.present() || (!logind.locked_hint() && !logind.idle_hint())
}

/// How hot one sensor is.
pub fn heat_of(sensor: &Temperature) -> Heat {
    let limit = match (sensor.critical, sensor.chip) {
        (Some(critical), _) => Some(critical),
        (None, Chip::Cpu) => Some(CRITICAL_CHIP),
        _ => None,
    };
    if limit.is_some_and(|limit| sensor.celsius >= limit) {
        return Heat::Critical;
    }
    let hot = match (sensor.critical, sensor.chip) {
        (Some(critical), _) => sensor.celsius >= critical - NEAR,
        (None, Chip::Cpu | Chip::Gpu) => sensor.celsius >= HOT_CHIP,
        _ => false,
    };
    if hot { Heat::Hot } else { Heat::Fine }
}

/// The hottest level now, and the sensor that's at it.
pub fn heat(found: &[Temperature]) -> (Heat, Option<&Temperature>) {
    found
        .iter()
        .map(|sensor| (heat_of(sensor), sensor))
        .max_by(|(a, x), (b, y)| a.cmp(b).then(x.celsius.total_cmp(&y.celsius)))
        .map_or((Heat::Fine, None), |(heat, sensor)| (heat, Some(sensor)))
}

/// The warning due now, if any: critical at once, hot only when it held
/// for two readings; neither again within [`QUIET_FOR`] of the last of its
/// level, unless it's worse.
pub fn due(before: Heat, now: Heat, told: Option<(Heat, Instant)>, at: Instant) -> Option<Heat> {
    let level = match now {
        Heat::Critical => Heat::Critical,
        Heat::Hot if before >= Heat::Hot => Heat::Hot,
        _ => return None,
    };
    match told {
        Some((was, when)) if was >= level && at.duration_since(when) < QUIET_FOR => None,
        _ => Some(level),
    }
}

fn warn(level: Heat, hottest: &Temperature) {
    let reading = format!("{} {:.0} °C", hottest.label, hottest.celsius);
    let toast = match level {
        Heat::Critical => Notify::new("Very Hot")
            .body(&format!(
                "{reading}. Close what's working it hard, or the computer may switch off to \
                 protect itself."
            ))
            .icon("dialog-warning")
            .urgency(Urgency::Critical),
        _ => Notify::new("Running Hot")
            .body(&format!(
                "{reading}. It may slow down to cool off. Keep its vents clear."
            ))
            .icon("temperature-symbolic"),
    };
    notify(
        Feedback::Thermals,
        toast
            .category("device")
            .action("open", "Show Temperatures", || {
                amane::Launch::argv(&["emaki-settings", "--page", "about"]).activated();
            }),
        level == Heat::Critical,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sensor(chip: Chip, celsius: f32, critical: Option<f32>) -> Temperature {
        Temperature {
            label: String::from("CPU · Tctl"),
            chip,
            celsius,
            critical,
        }
    }

    #[test]
    fn a_cpu_is_hot_near_its_limit_and_critical_at_it() {
        assert_eq!(heat_of(&sensor(Chip::Cpu, 70.0, None)), Heat::Fine);
        assert_eq!(heat_of(&sensor(Chip::Cpu, 96.0, None)), Heat::Hot);
        assert_eq!(heat_of(&sensor(Chip::Cpu, 100.0, None)), Heat::Critical);
    }

    #[test]
    fn an_ssd_goes_by_its_own_limit() {
        assert_eq!(heat_of(&sensor(Chip::Nvme, 80.0, Some(84.85))), Heat::Fine);
        assert_eq!(heat_of(&sensor(Chip::Nvme, 82.0, Some(84.85))), Heat::Hot);
        assert_eq!(
            heat_of(&sensor(Chip::Nvme, 85.0, Some(84.85))),
            Heat::Critical
        );
        assert_eq!(heat_of(&sensor(Chip::Nvme, 99.0, None)), Heat::Fine);
    }

    #[test]
    fn the_hottest_sensor_at_the_worst_level_is_named() {
        let found = [
            sensor(Chip::Gpu, 96.0, None),
            sensor(Chip::Cpu, 97.0, None),
            sensor(Chip::Nvme, 50.0, Some(84.0)),
        ];
        let (level, hottest) = heat(&found);
        assert_eq!(level, Heat::Hot);
        assert_eq!(hottest.map(|s| s.celsius), Some(97.0));
    }

    #[test]
    fn a_spike_isnt_told_but_a_held_heat_is_once_a_while() {
        let now = Instant::now();
        assert_eq!(due(Heat::Fine, Heat::Hot, None, now), None);
        assert_eq!(due(Heat::Hot, Heat::Hot, None, now), Some(Heat::Hot));
        let told = Some((Heat::Hot, now));
        assert_eq!(
            due(Heat::Hot, Heat::Hot, told, now + Duration::from_secs(60)),
            None
        );
        assert_eq!(
            due(Heat::Hot, Heat::Hot, told, now + QUIET_FOR),
            Some(Heat::Hot)
        );
        // worse is told at once, even right after
        assert_eq!(
            due(
                Heat::Hot,
                Heat::Critical,
                told,
                now + Duration::from_secs(20)
            ),
            Some(Heat::Critical)
        );
    }
}
