//! The temperature sensors under `/sys/class/hwmon`, named the same way
//! wherever they show: Settings' About page lists them, and the shell
//! warns when one runs hot. Everything there is world-readable; nothing is
//! written.

use std::fs;
use std::path::Path;

/// What a sensor's chip measures.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chip {
    /// The processor.
    Cpu,
    /// The graphics.
    Gpu,
    /// An NVMe SSD.
    Nvme,
    /// Anything else, like the mainboard or the Wi‑Fi card.
    Other,
}

/// What the chip named `chip` (`/sys/class/hwmon/hwmonN/name`) measures.
pub fn chip_of(chip: &str) -> Chip {
    match chip {
        "k10temp" | "coretemp" | "zenpower" | "cpu_thermal" => Chip::Cpu,
        "amdgpu" | "nouveau" | "radeon" | "i915" | "xe" => Chip::Gpu,
        "nvme" => Chip::Nvme,
        _ => Chip::Other,
    }
}

/// What people call a sensor: "CPU", "GPU", "NVMe · Sensor 1".
pub fn label(chip: &str, sensor_label: &str) -> String {
    let readable = match chip {
        "k10temp" | "coretemp" | "zenpower" | "cpu_thermal" => "CPU",
        "amdgpu" | "xe" | "i915" => "GPU",
        "nvme" => "NVMe",
        "acpitz" => "Mainboard",
        "thinkpad" => "ThinkPad",
        other => other,
    };
    // boilerplate like "Composite" or the chip's own name adds nothing
    if sensor_label.is_empty()
        || sensor_label.eq_ignore_ascii_case(chip)
        || sensor_label.eq_ignore_ascii_case("composite")
    {
        readable.to_string()
    } else {
        format!("{readable} · {sensor_label}")
    }
}

/// One temperature, raw.
#[derive(Clone, Debug, PartialEq)]
pub struct Temperature {
    /// As [`label`] names it.
    pub label: String,
    /// What its chip measures.
    pub chip: Chip,
    /// In degrees Celsius.
    pub celsius: f32,
    /// The temperature its chip calls critical, when it gives one.
    pub critical: Option<f32>,
}

/// Every CPU, GPU and NVMe temperature now.
pub fn temperatures() -> Vec<Temperature> {
    temperatures_in(Path::new("/sys/class/hwmon"))
}

/// [`temperatures`], read from `folder` in place of `/sys/class/hwmon`.
pub fn temperatures_in(folder: &Path) -> Vec<Temperature> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    for entry in entries.flatten() {
        let hwmon = entry.path();
        let name = text(&hwmon.join("name"));
        let chip = chip_of(&name);
        if chip == Chip::Other {
            continue;
        }
        for i in 1..=8 {
            let Some(celsius) = degrees(&hwmon.join(format!("temp{i}_input"))) else {
                continue;
            };
            found.push(Temperature {
                label: label(&name, &text(&hwmon.join(format!("temp{i}_label")))),
                chip,
                celsius,
                critical: degrees(&hwmon.join(format!("temp{i}_crit")))
                    .filter(|critical| *critical > 0.0),
            });
        }
    }
    found
}

fn text(file: &Path) -> String {
    fs::read_to_string(file)
        .map(|text| text.trim().to_string())
        .unwrap_or_default()
}

// sysfs gives millidegrees
fn degrees(file: &Path) -> Option<f32> {
    let milli: i32 = fs::read_to_string(file).ok()?.trim().parse().ok()?;
    Some(milli as f32 / 1000.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_a_sensor_without_the_boilerplate() {
        assert_eq!(label("k10temp", "Tctl"), "CPU · Tctl");
        assert_eq!(label("nvme", "Composite"), "NVMe");
        assert_eq!(label("amdgpu", ""), "GPU");
        assert_eq!(label("acpitz", "acpitz"), "Mainboard");
    }

    #[test]
    fn reads_the_temperatures_that_matter_with_their_limits() {
        let root = std::env::temp_dir().join(format!("emaki-sensors-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let write = |dir: &str, file: &str, text: &str| {
            fs::create_dir_all(root.join(dir)).unwrap();
            fs::write(root.join(dir).join(file), format!("{text}\n")).unwrap();
        };
        write("hwmon0", "name", "nvme");
        write("hwmon0", "temp1_input", "41850");
        write("hwmon0", "temp1_label", "Composite");
        write("hwmon0", "temp1_crit", "84850");
        write("hwmon4", "name", "k10temp");
        write("hwmon4", "temp1_input", "52125");
        write("hwmon4", "temp1_label", "Tctl");
        write("hwmon9", "name", "acpitz");
        write("hwmon9", "temp1_input", "51000");

        let mut found = temperatures_in(&root);
        found.sort_by(|a, b| a.label.cmp(&b.label));
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].label, "CPU · Tctl");
        assert_eq!(found[0].critical, None);
        assert_eq!(found[1].label, "NVMe");
        assert_eq!(found[1].critical, Some(84.85));
    }
}
