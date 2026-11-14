//! The laptop's own power:
//!
//! - **The charger.** Plugging in shows a capsule with what the charger
//!   agreed to give ("Charging · 100 W"), read from the USB‑C port once
//!   Power Delivery has settled; unplugging shows the time left.
//! - **A slow charger.** One giving less than [`SLOW_WATTS`] is told of
//!   once per plug: the battery may still run down under load.
//! - **Plugged in, not charging.** Still running down a while after being
//!   plugged in, short of the charge limit: told once per plug.
//! - **Running low.** UPower's own warning levels: a notification at Low,
//!   with Power Saver a click away, and a critical one past it. Both reset
//!   once the charger is in.
//! - **Charged.** Charging stopping at full or the charge limit is a
//!   capsule.

use std::sync::Once;
use std::time::{Duration, SystemTime};

use amane::{Battery, BatteryState, Notify, PowerProfile, Service, Urgency, WarningLevel};
use emaki_kit::format::span;
use emaki_kit::icons::battery_glyph;
use emaki_kit::settings::Feedback;
use emaki_kit::usb_c::{UsbCPort, charging_port};

use super::{Echo, echo, notify};
use crate::services::Wake;

/// Below this a charger is told of as slow.
pub const SLOW_WATTS: f32 = 30.0;

/// How long after plugging in the port is read, once Power Delivery has
/// agreed on a voltage.
const NEGOTIATE: Duration = Duration::from_secs(2);

/// How long after plugging in a battery still running down is told of.
const STILL_DRAINING: Duration = Duration::from_secs(90);

const SAVER: &str = "power-saver";

static WAKE: Wake = Wake::new();

/// One reading of the battery.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reading {
    pub on_ac: bool,
    pub percent: u8,
    pub state: BatteryState,
    pub warning: WarningLevel,
    pub time_to_empty: Option<Duration>,
}

/// What's been seen and told, between readings.
#[derive(Debug, Default)]
pub struct Memory {
    /// On the charger at the last reading; none before the first.
    on_ac: Option<bool>,
    /// The warning level last told of while on battery.
    warned: WarningLevel,
    /// Charging at the last reading.
    charging: bool,
    /// When the charger went in, for the checks that wait on it.
    plugged_at: Option<SystemTime>,
    /// The port has been read since the charger went in.
    port_read: bool,
    /// Still running down has been told of since the charger went in.
    draining_told: bool,
}

/// What a reading calls for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    Unplugged,
    Low,
    Critical,
    Charged,
}

/// Starts watching, once.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        Battery::subscribe(|_| WAKE.nudge()).forever();
        // the USB‑C ports' own changes, so the charger's power is read as it settles
        if let Some(events) = amane::device_events("power_supply") {
            std::thread::spawn(move || {
                for _ in events {
                    WAKE.nudge();
                }
            });
        }
        std::thread::spawn(|| {
            let mut memory = Memory::default();
            loop {
                let next = check(&mut memory, SystemTime::now());
                WAKE.wait(next);
            }
        });
    });
}

/// Reads the battery, tells what changed, and gives when to look again.
fn check(memory: &mut Memory, now: SystemTime) -> Option<SystemTime> {
    let battery = Battery::read();
    if !battery.present() {
        return None;
    }
    let reading = Reading {
        on_ac: battery.on_ac(),
        percent: battery.percent(),
        state: battery.state(),
        warning: battery.warning(),
        time_to_empty: battery.time_to_empty(),
    };
    drop(battery);

    for event in events(memory, &reading, now) {
        tell(event, &reading);
    }

    // the charger's own checks wait a while after it goes in
    let plugged_at = memory.plugged_at?;
    if !memory.port_read && now >= plugged_at + NEGOTIATE {
        memory.port_read = true;
        plugged_in(&reading, charging_port());
    }
    if !memory.draining_told && now >= plugged_at + STILL_DRAINING {
        memory.draining_told = true;
        if reading.state == BatteryState::Discharging {
            notify(
                Feedback::Power,
                Notify::new("Plugged In, Not Charging")
                    .body("The charger may be too weak for what the computer is doing.")
                    .category("device"),
                false,
            );
        }
    }
    [
        (!memory.port_read).then_some(plugged_at + NEGOTIATE),
        (!memory.draining_told).then_some(plugged_at + STILL_DRAINING),
    ]
    .into_iter()
    .flatten()
    .min()
}

/// What `reading` calls for, given what's been seen: the charger's own
/// capsule waits for its port to be read, so it isn't here.
pub fn events(memory: &mut Memory, reading: &Reading, now: SystemTime) -> Vec<Event> {
    let mut events = Vec::new();
    let first = memory.on_ac.is_none();
    let was_on_ac = memory.on_ac.replace(reading.on_ac);
    let charging = reading.state == BatteryState::Charging;
    let was_charging = std::mem::replace(&mut memory.charging, charging);

    if reading.on_ac {
        memory.warned = WarningLevel::None;
        if was_on_ac == Some(false) {
            memory.plugged_at = Some(now);
            memory.port_read = false;
            memory.draining_told = false;
        }
        let stopped = matches!(
            reading.state,
            BatteryState::Full | BatteryState::PendingCharge
        );
        if was_charging && stopped && !first {
            events.push(Event::Charged);
        }
        return events;
    }

    memory.plugged_at = None;
    if was_on_ac == Some(true) {
        events.push(Event::Unplugged);
    }
    let level = reading.warning;
    if level >= WarningLevel::Critical && memory.warned < WarningLevel::Critical {
        events.push(Event::Critical);
        memory.warned = level;
    } else if level == WarningLevel::Low && memory.warned < WarningLevel::Low {
        events.push(Event::Low);
        memory.warned = level;
    }
    events
}

fn tell(event: Event, reading: &Reading) {
    let percent = reading.percent;
    match event {
        Event::Unplugged => echo(
            Feedback::Power,
            Echo {
                glyph: battery_glyph(percent, false),
                name: String::from("On Battery"),
                detail: match reading.time_to_empty {
                    Some(left) => format!("{} left", span(left)),
                    None => format!("{percent} %"),
                },
            },
        ),
        Event::Charged => echo(
            Feedback::Power,
            Echo {
                glyph: battery_glyph(percent, false),
                name: String::from("Charged"),
                detail: format!("{percent} %"),
            },
        ),
        Event::Low => {
            let body = match reading.time_to_empty {
                Some(left) => format!("{percent} % left, about {}.", span(left)),
                None => format!("{percent} % left."),
            };
            let mut low = Notify::new("Battery Low")
                .body(&body)
                .icon("battery-low")
                .category("device");
            if PowerProfile::read().active() != SAVER {
                low = low.action("saver", "Turn On Power Saver", || PowerProfile::set(SAVER));
            }
            notify(Feedback::Power, low, true);
        }
        Event::Critical => {
            notify(
                Feedback::Power,
                Notify::new("Battery Very Low")
                    .body(&format!(
                        "{percent} % left. Plug in now; the computer will sleep soon."
                    ))
                    .icon("battery-caution")
                    .category("device")
                    .urgency(Urgency::Critical),
                true,
            );
        }
    }
}

/// The charger's capsule, and the slow-charger notice, once its port is
/// read; a charger on a port that isn't USB‑C shows without its power.
fn plugged_in(reading: &Reading, port: Option<UsbCPort>) {
    let charging = reading.state == BatteryState::Charging;
    echo(
        Feedback::Power,
        Echo {
            glyph: battery_glyph(reading.percent, true),
            name: String::from(if charging { "Charging" } else { "Plugged In" }),
            detail: port
                .as_ref()
                .map(UsbCPort::watts_label)
                .unwrap_or_else(|| format!("{} %", reading.percent)),
        },
    );
    if let Some(port) = port.filter(is_slow) {
        notify(
            Feedback::Power,
            Notify::new("Slow Charger")
                .body(&format!(
                    "The charger on USB‑C port {} gives {}. The battery may still run down \
                     while the computer is busy.",
                    port.number,
                    port.watts_label()
                ))
                .icon("battery-caution")
                .category("device"),
            false,
        );
    }
}

/// A charger that gave some power, but less than a laptop wants.
pub fn is_slow(port: &UsbCPort) -> bool {
    port.watts > 0.5 && port.watts < SLOW_WATTS
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reading(on_ac: bool, state: BatteryState, warning: WarningLevel) -> Reading {
        Reading {
            on_ac,
            percent: 50,
            state,
            warning,
            time_to_empty: None,
        }
    }

    #[test]
    fn the_first_reading_tells_nothing() {
        let mut memory = Memory::default();
        let now = SystemTime::now();
        let on = reading(true, BatteryState::Full, WarningLevel::None);
        assert!(events(&mut memory, &on, now).is_empty());
        assert_eq!(memory.plugged_at, None);
    }

    #[test]
    fn unplugging_then_plugging_in_starts_the_chargers_checks() {
        let mut memory = Memory::default();
        let now = SystemTime::now();
        let on = reading(true, BatteryState::Charging, WarningLevel::None);
        let off = reading(false, BatteryState::Discharging, WarningLevel::None);
        events(&mut memory, &on, now);
        assert_eq!(events(&mut memory, &off, now), vec![Event::Unplugged]);
        assert!(events(&mut memory, &on, now).is_empty());
        assert_eq!(memory.plugged_at, Some(now));
    }

    #[test]
    fn warns_once_at_low_and_once_at_critical_until_plugged_in() {
        let mut memory = Memory::default();
        let now = SystemTime::now();
        let low = reading(false, BatteryState::Discharging, WarningLevel::Low);
        let critical = reading(false, BatteryState::Discharging, WarningLevel::Critical);
        let action = reading(false, BatteryState::Discharging, WarningLevel::Action);
        assert_eq!(events(&mut memory, &low, now), vec![Event::Low]);
        assert!(events(&mut memory, &low, now).is_empty());
        assert_eq!(events(&mut memory, &critical, now), vec![Event::Critical]);
        assert!(events(&mut memory, &action, now).is_empty());

        let on = reading(true, BatteryState::Charging, WarningLevel::None);
        events(&mut memory, &on, now);
        let unplugged = events(&mut memory, &low, now);
        assert_eq!(unplugged, vec![Event::Unplugged, Event::Low]);
    }

    #[test]
    fn found_critical_warns_critical_alone() {
        let mut memory = Memory::default();
        let critical = reading(false, BatteryState::Discharging, WarningLevel::Critical);
        assert_eq!(
            events(&mut memory, &critical, SystemTime::now()),
            vec![Event::Critical]
        );
    }

    #[test]
    fn charging_stopping_at_the_limit_is_charged() {
        let mut memory = Memory::default();
        let now = SystemTime::now();
        let charging = reading(true, BatteryState::Charging, WarningLevel::None);
        let held = reading(true, BatteryState::PendingCharge, WarningLevel::None);
        events(&mut memory, &charging, now);
        assert_eq!(events(&mut memory, &held, now), vec![Event::Charged]);
        assert!(events(&mut memory, &held, now).is_empty());
    }

    #[test]
    fn a_slow_charger_gave_something_but_little() {
        let port = |watts| UsbCPort {
            number: 1,
            online: true,
            watts,
            mode: String::from("PD"),
        };
        assert!(is_slow(&port(15.0)));
        assert!(!is_slow(&port(65.0)));
        assert!(!is_slow(&port(0.0)));
    }
}
