//! The USB‑C ports' power, as the kernel's UCSI driver gives it under
//! `/sys/class/power_supply/ucsi-source-psy-*`: which port a charger is on,
//! what it agreed to give, and whether it speaks Power Delivery. Read
//! again on the `power_supply` subsystem's device events; nothing polls.

use std::fs;
use std::path::Path;

/// Where the power supplies are.
const POWER_SUPPLY: &str = "/sys/class/power_supply";

/// One USB‑C port that can charge the computer.
#[derive(Clone, Debug, PartialEq)]
pub struct UsbCPort {
    /// Counted from 1, in the firmware's order.
    pub number: u32,
    /// A charger is plugged in and giving power.
    pub online: bool,
    /// What it agreed to give, in watts: volts times the most current.
    pub watts: f32,
    /// What it's using, like "PD", "PD_PPS" or "C" (plain USB‑C, 15 W at
    /// most).
    pub mode: String,
}

impl UsbCPort {
    /// The negotiated power, as people say it: "65 W".
    pub fn watts_label(&self) -> String {
        format!("{:.0} W", self.watts)
    }
}

/// Every USB‑C port that can charge, in order; none on a computer without
/// UCSI.
pub fn ports() -> Vec<UsbCPort> {
    ports_in(Path::new(POWER_SUPPLY))
}

/// The port giving the most power now, if any is.
pub fn charging_port() -> Option<UsbCPort> {
    ports()
        .into_iter()
        .filter(|port| port.online)
        .max_by(|a, b| a.watts.total_cmp(&b.watts))
}

/// [`ports`], read from `folder` in place of `/sys/class/power_supply`.
pub fn ports_in(folder: &Path) -> Vec<UsbCPort> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut ports: Vec<(String, std::path::PathBuf)> = entries
        .flatten()
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            )
        })
        .filter(|(name, _)| name.starts_with("ucsi-source-psy-"))
        .collect();
    ports.sort();
    ports
        .into_iter()
        .enumerate()
        .map(|(index, (name, path))| read_port(&name, &path, index as u32 + 1))
        .collect()
}

fn read_port(name: &str, path: &Path, fallback: u32) -> UsbCPort {
    let text = |file: &str| {
        fs::read_to_string(path.join(file))
            .map(|text| text.trim().to_string())
            .unwrap_or_default()
    };
    let micro = |file: &str| text(file).parse::<f64>().unwrap_or(0.0) / 1_000_000.0;
    // "ucsi-source-psy-USBC000:002" is the second connector
    let number = name
        .rsplit(':')
        .next()
        .and_then(|n| n.parse().ok())
        .unwrap_or(fallback);
    let online = text("online") == "1";
    let amps = micro("current_max").max(micro("current_now"));
    let watts = if online {
        (micro("voltage_now") * amps) as f32
    } else {
        0.0
    };
    UsbCPort {
        number,
        online,
        watts,
        mode: chosen(&text("usb_type")),
    }
}

/// The bracketed choice in a sysfs list: "C [PD] PD_PPS" is "PD".
fn chosen(list: &str) -> String {
    list.split_whitespace()
        .find_map(|word| word.strip_prefix('[')?.strip_suffix(']'))
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn supply(root: &Path, name: &str, files: &[(&str, &str)]) {
        let dir = root.join(name);
        fs::create_dir_all(&dir).unwrap();
        for (file, text) in files {
            fs::write(dir.join(file), format!("{text}\n")).unwrap();
        }
    }

    #[test]
    fn reads_the_port_a_charger_is_on_and_its_power() {
        let root = std::env::temp_dir().join(format!("emaki-usb-c-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        supply(&root, "AC", &[("online", "1")]);
        supply(
            &root,
            "ucsi-source-psy-USBC000:001",
            &[
                ("online", "0"),
                ("usb_type", "[C] PD PD_PPS"),
                ("voltage_now", "0"),
                ("current_max", "0"),
            ],
        );
        supply(
            &root,
            "ucsi-source-psy-USBC000:002",
            &[
                ("online", "1"),
                ("usb_type", "C [PD] PD_PPS"),
                ("voltage_now", "20000000"),
                ("current_now", "5000000"),
                ("current_max", "5000000"),
            ],
        );
        let ports = ports_in(&root);
        assert_eq!(ports.len(), 2);
        assert!(!ports[0].online);
        assert_eq!(ports[1].number, 2);
        assert_eq!(ports[1].mode, "PD");
        assert_eq!(ports[1].watts_label(), "100 W");
    }

    #[test]
    fn picks_the_bracketed_choice() {
        assert_eq!(chosen("C [PD] PD_PPS"), "PD");
        assert_eq!(chosen("[C] PD"), "C");
        assert_eq!(chosen(""), "");
    }
}
