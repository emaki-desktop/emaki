//! Hardware group on the About page: CPU, GPU, NVMe temperatures and
//! fan RPM, read from `/sys/class/hwmon`. Everything there is
//! world-readable, so this page only reads; nothing is written.

use std::fs;
use std::path::Path;
use std::time::Duration;

use amane::Service;
use emaki_kit::color::Accent;
use emaki_kit::form::{FormRow, Group};
use emaki_kit::icons::{ICON_COMPUTER, ICON_FAN, ICON_STORAGE, ICON_THERMOSTAT};
use emaki_kit::sensors;

use crate::group;

/// How often the sensors re-read. Slower than a dashboard would use;
/// this is a Settings page, not a graph.
const INTERVAL: Duration = Duration::from_secs(2);
/// Stops polling after this long with nobody watching.
const IDLE_STOP: Duration = Duration::from_secs(30);

/// What the user sees on one row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Reading {
    label: String,
    kind: Kind,
    value: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Cpu,
    Gpu,
    Nvme,
    Fan,
    Other,
}

impl Kind {
    fn glyph(self) -> char {
        match self {
            Kind::Cpu => ICON_COMPUTER,
            Kind::Gpu => ICON_THERMOSTAT,
            Kind::Nvme => ICON_STORAGE,
            Kind::Fan => ICON_FAN,
            Kind::Other => ICON_THERMOSTAT,
        }
    }

    fn accent(self) -> Accent {
        match self {
            Kind::Cpu => Accent::Orange,
            Kind::Gpu => Accent::Teal,
            Kind::Nvme => Accent::Blue,
            Kind::Fan => Accent::Gray,
            Kind::Other => Accent::Gray,
        }
    }

    fn order(self) -> u8 {
        match self {
            Kind::Cpu => 0,
            Kind::Gpu => 1,
            Kind::Nvme => 2,
            Kind::Other => 3,
            Kind::Fan => 4,
        }
    }
}

/// Maps a chip name (`/sys/class/hwmon/hwmonN/name`) to a readable kind,
/// as the shell's temperature warning knows it too.
fn kind_of(chip: &str) -> Kind {
    match sensors::chip_of(chip) {
        sensors::Chip::Cpu => Kind::Cpu,
        sensors::Chip::Gpu => Kind::Gpu,
        sensors::Chip::Nvme => Kind::Nvme,
        sensors::Chip::Other => Kind::Other,
    }
}

/// Millidegrees to a short readable string.
fn format_celsius(milli: i32) -> String {
    format!("{} °C", (milli + 500).div_euclid(1000))
}

/// Fan RPM to a short readable string; 0 reads as "off" (many ThinkPads
/// stop the fan in idle).
fn format_fan(rpm: u32) -> String {
    if rpm == 0 {
        String::from("off")
    } else {
        format!("{rpm} rpm")
    }
}

/// The Service that polls hwmon while the About page is read.
pub struct Thermals {
    readings: Vec<Reading>,
    asked: bool,
}

impl Service for Thermals {
    // Scan straight away so opening About doesn't flash a blank group for the
    // interval; `new()` runs on first read, not on process start.
    fn new() -> Self {
        Self {
            readings: scan(),
            asked: true,
        }
    }

    fn interval() -> Duration {
        INTERVAL
    }

    fn idle_stop() -> Option<Duration> {
        Some(IDLE_STOP)
    }

    fn update(&mut self) -> bool {
        let next = scan();
        let changed = !self.asked || next != self.readings;
        self.asked = true;
        self.readings = next;
        changed
    }
}

/// Walks `/sys/class/hwmon/*` and reads every temperature and fan input.
fn scan() -> Vec<Reading> {
    let mut readings = Vec::new();
    let Ok(entries) = fs::read_dir("/sys/class/hwmon") else {
        return readings;
    };

    for entry in entries.flatten() {
        let hwmon = entry.path();
        let chip = read_text(&hwmon.join("name"));
        if chip.is_empty() {
            continue;
        }
        let kind = kind_of(&chip);
        read_temperatures(&hwmon, &chip, kind, &mut readings);
        read_fans(&hwmon, &chip, &mut readings);
    }

    // one row per kind of temperature where possible; ThinkPads expose
    // many near-duplicate temp inputs on acpitz/thinkpad, so dedupe by
    // display label after formatting
    readings.sort_by_key(|r| (r.kind.order(), r.label.clone()));
    readings.dedup_by(|a, b| a.label == b.label && a.kind == b.kind);
    readings
}

fn read_temperatures(hwmon: &Path, chip: &str, kind: Kind, out: &mut Vec<Reading>) {
    for i in 1..=8 {
        let input = hwmon.join(format!("temp{i}_input"));
        let Some(milli) = read_i32(&input) else {
            continue;
        };
        let sensor_label = read_text(&hwmon.join(format!("temp{i}_label")));
        let label = sensors::label(chip, &sensor_label);
        out.push(Reading {
            label,
            kind,
            value: format_celsius(milli),
        });
    }
}

fn read_fans(hwmon: &Path, chip: &str, out: &mut Vec<Reading>) {
    for i in 1..=4 {
        let input = hwmon.join(format!("fan{i}_input"));
        let Some(rpm) = read_i32(&input) else {
            continue;
        };
        let sensor_label = read_text(&hwmon.join(format!("fan{i}_label")));
        let label = match (chip, sensor_label.as_str()) {
            (_, "") if i == 1 => String::from("Fan"),
            (_, "") => format!("Fan {i}"),
            (_, name) => format!("Fan · {name}"),
        };
        out.push(Reading {
            label,
            kind: Kind::Fan,
            value: format_fan(rpm.max(0) as u32),
        });
    }
}

fn read_text(file: &Path) -> String {
    fs::read_to_string(file)
        .map(|s| s.trim().to_string())
        .unwrap_or_default()
}

fn read_i32(file: &Path) -> Option<i32> {
    fs::read_to_string(file).ok()?.trim().parse().ok()
}

pub fn section() -> Group {
    // narrow to the kinds the page is really about; acpitz / mt7925 / spd5118
    // temps are noisy and don't help the user judge "is this machine hot".
    let readings: Vec<Reading> = Thermals::read()
        .readings
        .iter()
        .filter(|r| matches!(r.kind, Kind::Cpu | Kind::Gpu | Kind::Nvme | Kind::Fan))
        .cloned()
        .collect();
    if readings.is_empty() {
        return Group::new();
    }
    let mut group = group().header("Hardware");
    for (i, reading) in readings.iter().enumerate() {
        let id = format!("settings.about.hardware.{i}");
        group = group.row(
            FormRow::new(id, &reading.label)
                .icon(reading.kind.glyph(), reading.kind.accent())
                .value(&reading.value),
        );
    }
    group.footer("Read from /sys/class/hwmon. Values refresh every two seconds.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_of_known_chips() {
        assert_eq!(kind_of("k10temp"), Kind::Cpu);
        assert_eq!(kind_of("coretemp"), Kind::Cpu);
        assert_eq!(kind_of("amdgpu"), Kind::Gpu);
        assert_eq!(kind_of("nvme"), Kind::Nvme);
        assert_eq!(kind_of("acpitz"), Kind::Other);
        assert_eq!(kind_of(""), Kind::Other);
    }

    #[test]
    fn celsius_rounds_to_the_nearest() {
        assert_eq!(format_celsius(72000), "72 °C");
        assert_eq!(format_celsius(72499), "72 °C");
        assert_eq!(format_celsius(72500), "73 °C");
        assert_eq!(format_celsius(0), "0 °C");
    }

    #[test]
    fn fan_rpm_shows_off_at_zero() {
        assert_eq!(format_fan(0), "off");
        assert_eq!(format_fan(2400), "2400 rpm");
    }

    #[test]
    fn display_folds_composite_into_the_chip_name() {
        assert_eq!(sensors::label("nvme", "Composite"), "NVMe");
        assert_eq!(sensors::label("k10temp", ""), "CPU");
        assert_eq!(sensors::label("k10temp", "Tctl"), "CPU · Tctl");
        assert_eq!(
            sensors::label("coretemp", "Package id 0"),
            "CPU · Package id 0"
        );
    }
}
