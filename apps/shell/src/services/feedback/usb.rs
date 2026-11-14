//! USB devices plugged in and out: a capsule with the device's own name
//! ("MX Keys · Connected"), read from sysfs as the kernel binds it, and
//! remembered so its removal can be named too, once sysfs has let it go.
//!
//! Left out: hubs, root hubs, devices on the computer's own fixed ports
//! (its camera, fingerprint reader and Bluetooth radio, which come and go
//! as their radios do), and drives, which the drives feedback tells of once
//! they can be opened. Several devices at once, like a dock, are one
//! capsule.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Once;
use std::sync::mpsc::RecvTimeoutError;
use std::time::Duration;

use amane::DeviceEvent;
use emaki_kit::icons::{
    ICON_CAMERA, ICON_DRIVE, ICON_GAMEPAD, ICON_HEADPHONES, ICON_KEYBOARD, ICON_MOUSE, ICON_PHONE,
    ICON_PRINTER, ICON_USB,
};
use emaki_kit::settings::Feedback;

use super::{Echo, connection_sound, echo};

/// How long events are gathered before they're told, so a dock's devices
/// are one capsule.
const GATHER: Duration = Duration::from_millis(700);

/// One USB device as it was bound.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsbDevice {
    pub name: String,
    pub kind: Kind,
}

/// What a device is, from its interfaces' classes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Keyboard,
    Mouse,
    Gamepad,
    Audio,
    Camera,
    Printer,
    Phone,
    Storage,
    Other,
}

impl Kind {
    fn glyph(self) -> char {
        match self {
            Self::Keyboard => ICON_KEYBOARD,
            Self::Mouse => ICON_MOUSE,
            Self::Gamepad => ICON_GAMEPAD,
            Self::Audio => ICON_HEADPHONES,
            Self::Camera => ICON_CAMERA,
            Self::Printer => ICON_PRINTER,
            Self::Phone => ICON_PHONE,
            Self::Storage => ICON_DRIVE,
            Self::Other => ICON_USB,
        }
    }
}

/// Starts listening, once; without the kernel's events nothing is told.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        let Some(events) = amane::device_events("usb") else {
            return;
        };
        std::thread::Builder::new()
            .name(String::from("emaki-usb-feedback"))
            .spawn(move || {
                // what each bound device is, by devpath, to name it once it's gone
                let mut known: HashMap<String, UsbDevice> = HashMap::new();
                let mut came: Vec<UsbDevice> = Vec::new();
                let mut went: Vec<UsbDevice> = Vec::new();
                loop {
                    let wait = if came.is_empty() && went.is_empty() {
                        Duration::MAX
                    } else {
                        GATHER
                    };
                    match events.recv_timeout(wait) {
                        Ok(event) => handle(&event, &mut known, &mut came, &mut went),
                        Err(RecvTimeoutError::Timeout) => {
                            tell(&came, true);
                            tell(&went, false);
                            came.clear();
                            went.clear();
                        }
                        Err(RecvTimeoutError::Disconnected) => break,
                    }
                }
            })
            .ok();
    });
}

fn handle(
    event: &DeviceEvent,
    known: &mut HashMap<String, UsbDevice>,
    came: &mut Vec<UsbDevice>,
    went: &mut Vec<UsbDevice>,
) {
    if event.vars.get("DEVTYPE").map(String::as_str) != Some("usb_device") {
        return;
    }
    match event.action.as_str() {
        // bound once its interfaces are there, so their classes can be read
        "bind" => {
            let dir = Path::new("/sys").join(event.devpath.trim_start_matches('/'));
            if let Some(device) = describe(&dir) {
                known.insert(event.devpath.clone(), device.clone());
                if device.kind != Kind::Storage {
                    came.push(device);
                }
            }
        }
        "remove" => {
            if let Some(device) = known.remove(&event.devpath)
                && device.kind != Kind::Storage
            {
                went.push(device);
            }
        }
        _ => {}
    }
}

fn tell(devices: &[UsbDevice], connected: bool) {
    let state = if connected {
        "Connected"
    } else {
        "Disconnected"
    };
    let readout = match devices {
        [] => return,
        [device] => Echo {
            glyph: device.kind.glyph(),
            name: device.name.clone(),
            detail: String::from(state),
        },
        many => Echo {
            glyph: ICON_USB,
            name: format!("{} USB Devices", many.len()),
            detail: String::from(state),
        },
    };
    echo(Feedback::UsbDevices, readout);
    connection_sound(connected);
}

/// The device whose sysfs folder is `dir`, unless it's one to leave out: a
/// hub, a root hub or one on a fixed port.
pub fn describe(dir: &Path) -> Option<UsbDevice> {
    let read = |name: &str| {
        fs::read_to_string(dir.join(name))
            .ok()
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
    };

    // a root hub is "usb1"; a device is "1-2" or "1-2.3"
    let folder = dir.file_name()?.to_string_lossy().into_owned();
    if !folder.contains('-') || folder.contains(':') {
        return None;
    }
    if read("removable").as_deref() == Some("fixed")
        || read("bDeviceClass").as_deref() == Some("09")
    {
        return None;
    }

    let interfaces = interfaces(dir);
    let kind = kind(&interfaces);
    let name = match (read("product"), read("manufacturer")) {
        (Some(product), _) => product,
        (None, Some(maker)) => format!("{maker} Device"),
        (None, None) => String::from("USB Device"),
    };
    Some(UsbDevice { name, kind })
}

/// Each interface's class and protocol, as two hex bytes.
fn interfaces(dir: &Path) -> Vec<(u8, u8)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let byte = |path: &Path| {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| u8::from_str_radix(text.trim(), 16).ok())
    };
    entries
        .flatten()
        .filter(|entry| entry.file_name().to_string_lossy().contains(':'))
        .filter_map(|entry| {
            let path = entry.path();
            Some((
                byte(&path.join("bInterfaceClass"))?,
                byte(&path.join("bInterfaceProtocol")).unwrap_or(0),
            ))
        })
        .collect()
}

/// What a device is, by the most telling of its interfaces.
pub fn kind(interfaces: &[(u8, u8)]) -> Kind {
    let has = |class: u8| interfaces.iter().any(|(c, _)| *c == class);
    let hid = |protocol: u8| interfaces.iter().any(|(c, p)| *c == 0x03 && *p == protocol);
    if has(0x08) {
        Kind::Storage
    } else if has(0x0e) {
        Kind::Camera
    } else if has(0x07) {
        Kind::Printer
    } else if has(0x06) {
        // still image: a phone or camera offering its photos
        Kind::Phone
    } else if hid(1) {
        Kind::Keyboard
    } else if hid(2) {
        Kind::Mouse
    } else if has(0x01) {
        Kind::Audio
    } else if has(0x03) || has(0x58) {
        // a HID that's neither, or an Xbox controller's own class
        Kind::Gamepad
    } else {
        Kind::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(folder: &str, files: &[(&str, &str)]) -> std::path::PathBuf {
        let dir = std::env::temp_dir()
            .join(format!("emaki-usb-test-{}", std::process::id()))
            .join(folder);
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        for (name, text) in files {
            let path = dir.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, format!("{text}\n")).unwrap();
        }
        dir
    }

    #[test]
    fn names_a_keyboard_by_its_product() {
        let dir = device(
            "3-1",
            &[
                ("product", "MX Keys"),
                ("manufacturer", "Logitech"),
                ("removable", "removable"),
                ("bDeviceClass", "00"),
                ("3-1:1.0/bInterfaceClass", "03"),
                ("3-1:1.0/bInterfaceProtocol", "01"),
            ],
        );
        assert_eq!(
            describe(&dir),
            Some(UsbDevice {
                name: String::from("MX Keys"),
                kind: Kind::Keyboard,
            })
        );
    }

    #[test]
    fn leaves_out_fixed_ports_hubs_and_root_hubs() {
        let camera = device(
            "1-1",
            &[("product", "Integrated Camera"), ("removable", "fixed")],
        );
        assert_eq!(describe(&camera), None);
        let hub = device("3-5", &[("product", "USB2.0 Hub"), ("bDeviceClass", "09")]);
        assert_eq!(describe(&hub), None);
        let root = device("usb1", &[("product", "xHCI Host Controller")]);
        assert_eq!(describe(&root), None);
    }

    #[test]
    fn a_nameless_device_is_named_by_its_maker_or_not_at_all() {
        let made = device("2-1", &[("manufacturer", "ACME")]);
        assert_eq!(describe(&made).unwrap().name, "ACME Device");
        let bare = device("2-2", &[]);
        assert_eq!(describe(&bare).unwrap().name, "USB Device");
    }

    #[test]
    fn storage_wins_over_anything_else_it_offers() {
        assert_eq!(kind(&[(0x03, 1), (0x08, 0x50)]), Kind::Storage);
        assert_eq!(kind(&[(0x01, 0), (0x03, 0)]), Kind::Audio);
        assert_eq!(kind(&[(0x03, 2)]), Kind::Mouse);
        assert_eq!(kind(&[]), Kind::Other);
    }
}
