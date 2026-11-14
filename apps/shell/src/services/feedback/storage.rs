//! Removable drives, through amane's `Disks` (UDisks2):
//!
//! - **Plugged in.** Mounted (unless Settings says not to), then a
//!   notification with the drive's name and size, **Open** and **Eject**.
//! - **Eject.** The notification turns into "Ejecting…", then into a
//!   capsule, "Safe to Remove", once the drive is powered off. A drive a
//!   program still has a file open on says so, with Try Again.
//! - **Pulled out while mounted.** A notification that stays: what was
//!   being written may be lost.
//! - **Couldn't be mounted.** A notification with UDisks's reason.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, Once, PoisonError};

use amane::{Disks, Notify, NotifyHandle, Service, ServiceError, Volume};
use emaki_kit::format::bytes;
use emaki_kit::icons::ICON_EJECT;
use emaki_kit::settings::{Feedback, notification_rules};

use super::{Echo, echo, notify};
use crate::services::Wake;

static WAKE: Wake = Wake::new();

// the drives being ejected here, by object: their going is no surprise
static EJECTING: Mutex<BTreeSet<String>> = Mutex::new(BTreeSet::new());

/// What changed between two readings.
#[derive(Debug, PartialEq, Eq)]
pub enum Change {
    /// A filesystem appeared.
    Came(Volume),
    /// One went while it was still mounted.
    PulledOut(Volume),
}

/// Starts watching, once.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        Disks::subscribe(|_| WAKE.nudge()).forever();
        std::thread::spawn(|| {
            let mut before: Option<BTreeMap<String, Volume>> = None;
            loop {
                let now = read();
                if let Some(before) = &before {
                    let ejecting = EJECTING
                        .lock()
                        .unwrap_or_else(PoisonError::into_inner)
                        .clone();
                    for change in changes(before, &now, &ejecting) {
                        tell(change);
                    }
                }
                EJECTING
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .retain(|object| now.contains_key(object));
                before = Some(now);
                WAKE.wait(None);
            }
        });
    });
}

fn read() -> BTreeMap<String, Volume> {
    Disks::read()
        .volumes()
        .iter()
        .map(|volume| (volume.object.clone(), volume.clone()))
        .collect()
}

/// The filesystems that came, and the mounted ones that went without being
/// ejected here.
pub fn changes(
    before: &BTreeMap<String, Volume>,
    now: &BTreeMap<String, Volume>,
    ejecting: &BTreeSet<String>,
) -> Vec<Change> {
    let pulled = before
        .values()
        .filter(|volume| !now.contains_key(&volume.object))
        .filter(|volume| volume.mounted() && !ejecting.contains(&volume.object))
        .cloned()
        .map(Change::PulledOut);
    let came = now
        .values()
        .filter(|volume| !before.contains_key(&volume.object))
        .cloned()
        .map(Change::Came);
    pulled.chain(came).collect()
}

fn tell(change: Change) {
    match change {
        Change::PulledOut(volume) => {
            notify(
                Feedback::Storage,
                Notify::new(&format!("{} Wasn't Ejected", volume.name))
                    .body(
                        "Eject a drive before unplugging it, so nothing being saved to it is lost.",
                    )
                    .icon("drive-removable-media")
                    .category("device.removed"),
                false,
            );
        }
        Change::Came(volume) => {
            // mounting waits on UDisks, and maybe on the drive spinning up
            std::thread::spawn(move || arrived(volume));
        }
    }
}

/// A drive plugged in: mounted to open, unless it was or shouldn't be.
fn arrived(volume: Volume) {
    let mount = !volume.mounted() && volume.auto && !notification_rules().keep_drives_unmounted;
    let place = if mount {
        match Disks::mount(&volume.object) {
            Ok(place) => Some(place),
            Err(error) => {
                couldnt_mount(&volume, &error);
                return;
            }
        }
    } else {
        volume.mount_points.first().cloned()
    };
    ready(&volume, place.as_deref(), None);
}

/// The drive's notification: Open when it's mounted, and Eject.
fn ready(volume: &Volume, place: Option<&str>, replaces: Option<&NotifyHandle>) {
    let slot: Arc<Mutex<Option<NotifyHandle>>> = Arc::default();
    let mut toast = Notify::new(&volume.name)
        .body(&format!("{} drive · Ready", bytes(volume.size)))
        .icon("drive-removable-media")
        .category("device.added")
        .transient(true);
    if let Some(replaces) = replaces {
        toast = toast.replaces(replaces);
    }
    if let Some(place) = place {
        let (open, click) = (place.to_string(), place.to_string());
        toast = toast
            .action("open", "Open", move || {
                emaki_kit::launch::open(open.clone())
            })
            .on_click(move || emaki_kit::launch::open(click.clone()));
    }
    if volume.ejectable || volume.mounted() || place.is_some() {
        let (ejected, handle) = (volume.clone(), slot.clone());
        toast = toast.action("eject", "Eject", move || {
            let handle = handle
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .clone();
            let volume = ejected.clone();
            std::thread::spawn(move || eject(&volume, handle.as_ref()));
        });
    }
    let handle = notify(Feedback::Storage, toast, false);
    *slot.lock().unwrap_or_else(PoisonError::into_inner) = handle;
}

/// Ejects the drive, telling how it went in place of its notification.
pub fn eject(volume: &Volume, shown: Option<&NotifyHandle>) {
    EJECTING
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(volume.object.clone());
    let mut ejecting = Notify::new(&format!("Ejecting {}…", volume.name))
        .icon("media-eject")
        .category("device")
        .transient(true)
        .resident(true);
    if let Some(shown) = shown {
        ejecting = ejecting.replaces(shown);
    }
    let progress = ejecting.desktop_entry(Feedback::Storage.entry()).send();

    match Disks::eject(volume) {
        Ok(()) => {
            progress.close();
            echo(
                Feedback::Storage,
                Echo {
                    glyph: ICON_EJECT,
                    name: volume.name.clone(),
                    detail: String::from("Safe to Remove"),
                },
            );
        }
        Err(error) => {
            EJECTING
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .remove(&volume.object);
            in_use(volume, &error, &progress);
        }
    }
}

/// The drive couldn't be ejected: most often a program has a file open.
fn in_use(volume: &Volume, error: &ServiceError, shown: &NotifyHandle) {
    let busy = error.message.contains("busy") || error.message.contains("Busy");
    let body = if busy {
        String::from("A program still has a file open on it. Close it, then try again.")
    } else {
        error.to_string()
    };
    let (again, slot) = (volume.clone(), Arc::new(Mutex::new(None::<NotifyHandle>)));
    let held = slot.clone();
    let handle = notify(
        Feedback::Storage,
        Notify::new(&format!("Couldn't Eject {}", volume.name))
            .body(&body)
            .icon("drive-removable-media")
            .category("device.error")
            .replaces(shown)
            .action("again", "Try Again", move || {
                let handle = held.lock().unwrap_or_else(PoisonError::into_inner).clone();
                let volume = again.clone();
                std::thread::spawn(move || eject(&volume, handle.as_ref()));
            }),
        true,
    );
    *slot.lock().unwrap_or_else(PoisonError::into_inner) = handle;
}

fn couldnt_mount(volume: &Volume, error: &ServiceError) {
    notify(
        Feedback::Storage,
        Notify::new(&format!("Couldn't Open {}", volume.name))
            .body(&error.to_string())
            .icon("drive-removable-media")
            .category("device.error"),
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(object: &str, mounted: bool) -> Volume {
        Volume {
            object: object.to_string(),
            drive: String::from("/drive"),
            name: String::from("PHOTOS"),
            size: 32_000_000_000,
            mount_points: if mounted {
                vec![String::from("/run/media/me/PHOTOS")]
            } else {
                Vec::new()
            },
            ejectable: true,
            auto: true,
        }
    }

    fn by_object(volumes: &[Volume]) -> BTreeMap<String, Volume> {
        volumes
            .iter()
            .map(|volume| (volume.object.clone(), volume.clone()))
            .collect()
    }

    #[test]
    fn a_new_drive_came() {
        let before = by_object(&[]);
        let now = by_object(&[volume("a", false)]);
        assert_eq!(
            changes(&before, &now, &BTreeSet::new()),
            vec![Change::Came(volume("a", false))]
        );
    }

    #[test]
    fn pulling_out_a_mounted_drive_is_told_unless_ejected_here() {
        let before = by_object(&[volume("a", true), volume("b", false)]);
        let now = by_object(&[]);
        assert_eq!(
            changes(&before, &now, &BTreeSet::new()),
            vec![Change::PulledOut(volume("a", true))]
        );
        let ejecting = BTreeSet::from([String::from("a")]);
        assert!(changes(&before, &now, &ejecting).is_empty());
    }
}
