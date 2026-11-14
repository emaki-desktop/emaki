//! Bluetooth devices, and phones through KDE Connect: a capsule as a device
//! connects ("AirPods Pro · Connected · 80 %") or goes, and a notification
//! as one runs low, at 20 % and again at 10 %, once each per connection,
//! as phones warn about a headset. Checked whenever amane's `Bluetooth` or
//! the `KdeConnect` service changes.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Once;

use amane::{Bluetooth, Notify, Service, Urgency};
use emaki_kit::icons::device_glyph;
use emaki_kit::settings::Feedback;
use emaki_phone::KdeConnect;

use super::levels::warning;
use super::{Echo, connection_sound, echo, notify};
use crate::services::Wake;

static WAKE: Wake = Wake::new();

/// One connected Bluetooth device, as last seen.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Seen {
    pub name: String,
    pub icon: String,
    pub battery: Option<u8>,
}

/// What changed between two readings, by address.
#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    Connected(String),
    Disconnected(String),
}

/// Starts watching, once.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        Bluetooth::subscribe(|_| WAKE.nudge()).forever();
        KdeConnect::subscribe(|_| WAKE.nudge()).forever();
        std::thread::spawn(|| {
            // the lowest level each device was warned at, by address
            let mut warned: BTreeMap<String, u8> = BTreeMap::new();
            // none until the first reading, which only sets the baseline
            let mut before: Option<BTreeMap<String, Seen>> = None;
            loop {
                let now = connected();
                if let Some(before) = &before {
                    for change in changes(before, &now) {
                        tell(change, before, &now);
                    }
                }
                before = Some(now);
                check_batteries(&mut warned);
                WAKE.wait(None);
            }
        });
    });
}

/// The Bluetooth devices connected now, by address.
fn connected() -> BTreeMap<String, Seen> {
    Bluetooth::read()
        .devices()
        .iter()
        .filter(|device| device.connected())
        .map(|device| {
            (
                device.address().to_string(),
                Seen {
                    name: device.name().to_string(),
                    icon: device.icon().to_string(),
                    battery: device.battery(),
                },
            )
        })
        .collect()
}

/// The devices that connected or went between `before` and `now`.
pub fn changes(before: &BTreeMap<String, Seen>, now: &BTreeMap<String, Seen>) -> Vec<Change> {
    let was: BTreeSet<&String> = before.keys().collect();
    let is: BTreeSet<&String> = now.keys().collect();
    let gone = was
        .difference(&is)
        .map(|address| Change::Disconnected((*address).clone()));
    let came = is
        .difference(&was)
        .map(|address| Change::Connected((*address).clone()));
    gone.chain(came).collect()
}

fn tell(change: Change, before: &BTreeMap<String, Seen>, now: &BTreeMap<String, Seen>) {
    let (seen, connected) = match &change {
        Change::Connected(address) => (now.get(address), true),
        Change::Disconnected(address) => (before.get(address), false),
    };
    let Some(seen) = seen else {
        return;
    };
    echo(
        Feedback::Bluetooth,
        Echo {
            glyph: device_glyph(&seen.icon),
            name: seen.name.clone(),
            detail: detail(connected, seen.battery),
        },
    );
    connection_sound(connected);
}

/// "Connected · 80 %", or "Disconnected".
pub fn detail(connected: bool, battery: Option<u8>) -> String {
    match (connected, battery) {
        (true, Some(percent)) => format!("Connected · {percent} %"),
        (true, None) => String::from("Connected"),
        (false, _) => String::from("Disconnected"),
    }
}

fn check_batteries(warned: &mut BTreeMap<String, u8>) {
    let bluetooth = Bluetooth::read();
    let devices: Vec<(String, String, u8)> = bluetooth
        .devices()
        .iter()
        .filter(|device| device.connected())
        .filter_map(|device| {
            let percent = device.battery()?;
            Some((
                device.address().to_string(),
                device.name().to_string(),
                percent,
            ))
        })
        .collect();
    drop(bluetooth);

    // a phone through KDE Connect warns the same way, unless it's charging
    let phones = KdeConnect::read();
    let devices: Vec<(String, String, u8)> = devices
        .into_iter()
        .chain(phones.devices().iter().filter_map(|phone| {
            let battery = phone.battery().filter(|b| !b.charging)?;
            phone.connected().then(|| {
                (
                    format!("kdeconnect:{}", phone.id()),
                    phone.name().to_string(),
                    battery.percent,
                )
            })
        }))
        .collect();
    drop(phones);

    // a device that disconnected is warned again on its next connection
    warned.retain(|address, _| devices.iter().any(|(a, _, _)| a == address));

    for (address, name, percent) in devices {
        let (now, warn) = warning(warned.get(&address).copied(), percent);
        match now {
            Some(level) => warned.insert(address, level),
            None => warned.remove(&address),
        };
        if let Some(level) = warn {
            let urgency = if level <= 10 {
                Urgency::Critical
            } else {
                Urgency::Normal
            };
            notify(
                Feedback::Bluetooth,
                Notify::new(&format!("{name} Battery Low"))
                    .body(&format!("{percent} % remaining"))
                    .category("device")
                    .urgency(urgency),
                true,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(name: &str) -> Seen {
        Seen {
            name: String::from(name),
            icon: String::from("audio-headphones"),
            battery: Some(80),
        }
    }

    #[test]
    fn tells_of_devices_that_came_and_went() {
        let before = BTreeMap::from([
            (String::from("A"), device("Mouse")),
            (String::from("B"), device("Buds")),
        ]);
        let now = BTreeMap::from([
            (String::from("B"), device("Buds")),
            (String::from("C"), device("Keyboard")),
        ]);
        assert_eq!(
            changes(&before, &now),
            vec![
                Change::Disconnected(String::from("A")),
                Change::Connected(String::from("C")),
            ]
        );
    }

    #[test]
    fn a_battery_change_alone_is_no_change() {
        let before = BTreeMap::from([(String::from("A"), device("Buds"))]);
        let mut lower = device("Buds");
        lower.battery = Some(40);
        let now = BTreeMap::from([(String::from("A"), lower)]);
        assert!(changes(&before, &now).is_empty());
    }

    #[test]
    fn the_detail_names_the_battery_when_known() {
        assert_eq!(detail(true, Some(80)), "Connected · 80 %");
        assert_eq!(detail(true, None), "Connected");
        assert_eq!(detail(false, Some(80)), "Disconnected");
    }
}
