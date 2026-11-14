//! A screenshot taken: a notification showing it, with Copy, Open and
//! Show in Files, and clicking it opens it. Sent transient, so the center
//! doesn't keep it once it goes. One only copied, never saved, is a
//! capsule. niri says when it takes one (`ScreenshotCaptured`).

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Once;

use amane::{Notify, niri};
use emaki_kit::icons::ICON_SCREENSHOT;
use emaki_kit::settings::Feedback;

use super::{Echo, echo, notify};
use crate::services::launcher::file_search::{open, reveal};

/// Starts following niri, once.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        std::thread::Builder::new()
            .name(String::from("emaki-screenshots"))
            .spawn(|| niri::follow_screenshots(taken))
            .ok();
    });
}

fn taken(path: Option<String>) {
    let Some(path) = path.map(PathBuf::from) else {
        echo(
            Feedback::Screenshots,
            Echo {
                glyph: ICON_SCREENSHOT,
                name: String::from("Screenshot Copied"),
                detail: String::new(),
            },
        );
        return;
    };
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let shown = path.to_string_lossy().into_owned();
    let (copy, opened, revealed, clicked) = (path.clone(), path.clone(), path.clone(), path);
    notify(
        Feedback::Screenshots,
        Notify::new("Screenshot Saved")
            .body(&name)
            .image(&shown)
            .category("transfer.complete")
            .transient(true)
            .action("copy", "Copy", move || copy_image(&copy))
            .action("open", "Open", move || open(&opened))
            .action("reveal", "Show in Files", move || reveal(&revealed))
            .on_click(move || open(&clicked)),
        false,
    );
}

/// Puts the picture on the clipboard, as wl-copy serves it.
fn copy_image(path: &Path) {
    let Ok(file) = File::open(path) else {
        return;
    };
    let _ = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(Stdio::from(file))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|mut child| child.wait());
}
