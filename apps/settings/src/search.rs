//! Searching Emaki Settings, from the field over the sidebar: what's typed
//! finds pages and the settings on them, by their titles and by
//! other words people use for them ("dark mode", "wifi"). A result opens
//! its page (or sub-page) and lights the row it names for a moment.
//!
//! The Shell pages' settings are listed here by hand; a test checks that
//! every page has some. The System pages' settings come from the
//! configuration as it's read, so they're found by their labels too.

use std::cell::RefCell;
use std::time::{Duration, Instant};

use amane::{Service, TextInput};

use crate::system::nixos::Snapshot;
use emaki_kit::fuzzy::fuzzy;

/// The search field's id.
pub const FIELD: &str = "settings.search";

/// One thing to find: where it is (`--page` names, `page/sub` for a
/// sub-page), its title, the row it lights, and other words for it.
#[derive(Clone, Copy)]
pub struct Entry {
    pub place: &'static str,
    pub title: &'static str,
    pub row: Option<&'static str>,
    pub words: &'static [&'static str],
}

const fn page(place: &'static str, title: &'static str, words: &'static [&'static str]) -> Entry {
    Entry {
        place,
        title,
        row: None,
        words,
    }
}

const fn row(
    place: &'static str,
    title: &'static str,
    row: &'static str,
    words: &'static [&'static str],
) -> Entry {
    Entry {
        place,
        title,
        row: Some(row),
        words,
    }
}

/// Everything a search finds: every page, from the pages' table, then the
/// sub-pages and the Shell pages' settings.
pub fn entries() -> impl Iterator<Item = Entry> {
    let pages = crate::pages::all().map(|(_, def)| page(def.name, def.title, def.words));
    pages.chain(INDEX.iter().copied())
}

/// The sub-pages and the Shell pages' settings.
const INDEX: &[Entry] = &[
    // Appearance
    row(
        "appearance",
        "Wallpaper under Black",
        "settings.appearance.black-wallpaper-row",
        &["oled", "dim wallpaper", "hide wallpaper", "black"],
    ),
    row(
        "appearance",
        "Light, dark, black or auto",
        "settings.appearance.mode",
        &[
            "dark mode",
            "light mode",
            "night",
            "theme",
            "black",
            "true black",
            "oled",
            "amoled",
        ],
    ),
    row(
        "appearance",
        "Accent color",
        "settings.appearance.accent",
        &["colour", "highlight", "tint"],
    ),
    row(
        "appearance",
        "Text size",
        "settings.appearance.text",
        &["font", "bigger", "zoom", "scale text"],
    ),
    row(
        "appearance",
        "Density",
        "settings.appearance.density",
        &["compact", "spacing", "comfortable"],
    ),
    row(
        "appearance",
        "Glass",
        "settings.appearance.glass",
        &["transparency", "translucent", "blur", "material"],
    ),
    row(
        "appearance",
        "Adaptive glass",
        "settings.appearance.adaptive",
        &[],
    ),
    row(
        "appearance",
        "Vibrant labels",
        "settings.appearance.vibrant",
        &["vibrancy"],
    ),
    row(
        "appearance",
        "Motion",
        "settings.appearance.motion",
        &["reduce motion", "animations"],
    ),
    // Desktop
    row(
        "desktop",
        "Show widgets on the desktop",
        "settings.show-widgets",
        &["widgets"],
    ),
    row(
        "desktop",
        "Widgets while windows are open",
        "settings.widget-style",
        &["dim widgets"],
    ),
    row(
        "desktop",
        "Edit widgets",
        "settings.widgets.edit-row",
        &["add widget", "move widget"],
    ),
    // Displays
    row(
        "displays",
        "Resolution",
        "settings.displays.resolution-row",
        &["size", "pixels"],
    ),
    row(
        "displays",
        "Refresh rate",
        "settings.displays.refresh-row",
        &["hz", "fps", "power adapter", "plugged in"],
    ),
    row(
        "displays",
        "Scale",
        "settings.displays.scale-row",
        &["zoom", "hidpi", "bigger"],
    ),
    row(
        "displays",
        "Rotation",
        "settings.displays.rotation-row",
        &["rotate", "portrait"],
    ),
    row(
        "displays",
        "Variable refresh rate",
        "settings.displays.refresh-row",
        &["vrr", "freesync", "adaptive sync", "gsync", "variable"],
    ),
    row(
        "displays",
        "HDR",
        "settings.displays.hdr-row",
        &["high dynamic range"],
    ),
    row(
        "displays",
        "Use this display",
        "settings.displays.enabled-row",
        &["turn off display"],
    ),
    row(
        "displays",
        "Main display",
        "settings.displays.main-row",
        &["primary"],
    ),
    row(
        "displays",
        "Refresh rate on battery",
        "settings.displays.battery-refresh-row",
        &[
            "power saving",
            "battery",
            "hz on battery",
            "slower on battery",
        ],
    ),
    row(
        "displays",
        "Night Light",
        "settings.displays.night.schedule-row",
        &["blue light", "warm", "night shift", "redshift"],
    ),
    row(
        "displays",
        "OLED care",
        "settings.displays.oled.dark",
        &["burn in", "black", "true black", "amoled"],
    ),
    // niri's settings
    row(
        "windows",
        "Gaps between windows",
        "settings.windows.center",
        &["gaps", "spacing"],
    ),
    row(
        "windows",
        "New windows' width",
        "settings.windows.width",
        &["column width", "default width"],
    ),
    row(
        "windows",
        "Center the focused column",
        "settings.windows.center",
        &["centering"],
    ),
    row(
        "windows",
        "Focus ring",
        "settings.windows.focus-ring",
        &["active window outline", "highlight"],
    ),
    row(
        "windows",
        "Border",
        "settings.windows.border",
        &["window outline"],
    ),
    row(
        "windows",
        "Shadows",
        "settings.windows.shadow",
        &["window shadow"],
    ),
    row(
        "windows",
        "Mat",
        "settings.windows.mat",
        &["backing", "column background"],
    ),
    row(
        "windows",
        "Ask apps to leave out their title bars",
        "settings.windows.no-csd",
        &["csd", "decorations", "title bar"],
    ),
    row(
        "windows",
        "Window switcher",
        "settings.windows.switcher",
        &["alt tab", "recent windows"],
    ),
    row(
        "windows",
        "Hot corner",
        "settings.windows.hot-corner",
        &["corner", "overview"],
    ),
    row(
        "mouse",
        "Focus follows the pointer",
        "settings.input.focus-follows-mouse",
        &["focus follows mouse", "hover focus"],
    ),
    row(
        "mouse",
        "Left-handed",
        "settings.input.left-handed",
        &["swap buttons", "left hand"],
    ),
    row(
        "mouse",
        "Middle click by pressing both buttons",
        "settings.input.mouse.middle-emulation",
        &["middle click"],
    ),
    row(
        "mouse",
        "Super and swipe moves windows",
        "settings.input.mod-swipe",
        &["gesture", "three fingers"],
    ),
    row(
        "appearance",
        "Cursor",
        "settings.appearance.cursor.theme",
        &["cursor theme", "pointer", "mouse pointer"],
    ),
    row(
        "appearance",
        "Cursor size",
        "settings.appearance.cursor.size",
        &["big cursor", "pointer size"],
    ),
    row(
        "desktop",
        "Screenshots",
        "settings.desktop.screenshots.clipboard",
        &["screenshot folder", "print screen"],
    ),
    // Network
    row(
        "network",
        "Wi‑Fi",
        "settings.network.wifi",
        &["wireless", "wifi", "airplane"],
    ),
    row(
        "network/saved",
        "Saved networks",
        "settings.network.saved",
        &["forget network", "known networks", "remembered"],
    ),
    row(
        "network/hidden",
        "Join a hidden network",
        "settings.network.hidden.name.row",
        &["hidden ssid", "unlisted network"],
    ),
    row(
        "network",
        "Import a VPN",
        "settings.network.vpn.import",
        &["wireguard", "openvpn", "ovpn", "vpn file"],
    ),
    row(
        "network",
        "Hotspot",
        "settings.network.hotspot",
        &["share connection", "tethering", "access point"],
    ),
    row(
        "network/connection",
        "IP address and DNS",
        "settings.network.ipv4.method",
        &["ip address", "dns", "gateway", "static ip", "dhcp"],
    ),
    // Sound
    row(
        "sound",
        "Output device",
        "settings.sound.output.device",
        &["speakers", "headphones", "audio output"],
    ),
    row(
        "sound",
        "Balance",
        "settings.sound.output.balance",
        &["left", "right", "stereo"],
    ),
    row(
        "sound",
        "Input device",
        "settings.sound.input.device",
        &["microphone", "mic", "audio input"],
    ),
    row(
        "sound",
        "Input level",
        "settings.sound.input.level",
        &["microphone", "mic test", "meter"],
    ),
    row(
        "notifications",
        "Play a sound for notifications",
        "settings.sound.notifications",
        &["alert sound", "notification sound", "chime"],
    ),
    row(
        "notifications",
        "Notification sound",
        "settings.sound.notifications.sound",
        &["alert sound", "chime"],
    ),
    row(
        "notifications",
        "System Feedback",
        "settings.notifications.system.sounds",
        &[
            "usb",
            "bluetooth",
            "device connected",
            "drive",
            "charger",
            "toast",
            "osd",
            "device sound",
            "plug sound",
        ],
    ),
    row(
        "notifications",
        "Indicator light",
        "settings.notifications.light.pulse",
        &["led", "lid logo", "thinkvantage", "blink", "pulse"],
    ),
    // Input
    row(
        "keyboard",
        "Delay before repeat",
        "settings.input.repeat-delay",
        &["key repeat", "hold a key"],
    ),
    row(
        "keyboard",
        "Repeat rate",
        "settings.input.repeat-rate",
        &["key repeat", "typematic"],
    ),
    row(
        "keyboard",
        "Num Lock on at login",
        "settings.input.numlock",
        &["numpad", "number lock"],
    ),
    row(
        "mouse",
        "Tap to click",
        "settings.input.touchpad.tap",
        &["touchpad", "trackpad", "tap"],
    ),
    row(
        "mouse",
        "Natural scrolling",
        "settings.input.touchpad.natural-scroll",
        &["reverse scrolling", "scroll direction", "touchpad"],
    ),
    row(
        "mouse",
        "Off while typing",
        "settings.input.touchpad.dwt",
        &["palm rejection", "disable while typing", "touchpad"],
    ),
    row(
        "mouse",
        "Touchpad acceleration",
        "settings.input.touchpad.accel-profile",
        &["touchpad speed", "sensitivity", "trackpad"],
    ),
    row(
        "mouse",
        "Right click",
        "settings.input.touchpad.click-method",
        &["secondary click", "two finger click", "clickfinger"],
    ),
    row(
        "mouse",
        "Scroll with",
        "settings.input.touchpad.scroll-method",
        &["edge scrolling", "two finger scroll"],
    ),
    row(
        "mouse",
        "Mouse natural scrolling",
        "settings.input.mouse.natural-scroll",
        &["reverse scrolling", "wheel"],
    ),
    row(
        "mouse",
        "Mouse acceleration",
        "settings.input.mouse.accel-profile",
        &["mouse speed", "sensitivity", "pointer speed", "flat"],
    ),
    row(
        "keyboard/shortcuts",
        "Keyboard shortcuts",
        "settings.input.shortcuts.filter",
        &["keybinds", "hotkeys", "key bindings", "niri binds"],
    ),
    row(
        "displays/advanced",
        "Custom scale",
        "settings.displays.custom-scale-row",
        &[],
    ),
    row(
        "displays/advanced",
        "Custom mode",
        "settings.displays.custom-mode-row",
        &["modeline"],
    ),
    row(
        "displays/advanced",
        "Overview backdrop",
        "settings.displays.backdrop-row",
        &["background color"],
    ),
    // Status Bar
    row(
        "status-bar",
        "Bar position",
        "settings.bar.position",
        &["top", "bottom"],
    ),
    row(
        "status-bar",
        "Bar size",
        "settings.bar.size",
        &[
            "height",
            "bigger icons",
            "compact",
            "regular",
            "comfortable",
        ],
    ),
    row(
        "status-bar",
        "Hide the bar automatically",
        "settings.bar.auto-hide",
        &["auto hide", "full screen", "fullscreen", "desktop", "never"],
    ),
    row(
        "status-bar",
        "Bar material",
        "settings.bar.material",
        &[
            "transparent bar",
            "clear",
            "solid",
            "frosted",
            "glass",
            "blur",
        ],
    ),
    row(
        "status-bar",
        "Workspaces in the bar",
        "settings.bar.workspaces",
        &["pips", "workspace shape"],
    ),
    row(
        "status-bar",
        "Scroll to switch workspaces",
        "settings.bar.scroll-workspaces",
        &["scroll", "wheel", "workspaces"],
    ),
    row(
        "status-bar/activities",
        "Now Playing",
        "settings.bar.media-always",
        &["music", "media", "spotify"],
    ),
    row(
        "status-bar/activities",
        "Activities",
        "settings.bar.activities",
        &[
            "activity",
            "live",
            "timer",
            "screen sharing",
            "progress",
            "upload",
        ],
    ),
    row(
        "status-bar/activities",
        "Activities with their text",
        "settings.bar.activities-compact",
        &["compact", "how many"],
    ),
    // the apps' rows are listed as they're seen: the sub-page itself
    page(
        "status-bar/activities",
        "Allow Activities",
        &["apps", "permission"],
    ),
    row(
        "status-bar/clock",
        "Clock",
        "settings.bar.clock-style-row",
        &["time", "analog", "digital"],
    ),
    row(
        "status-bar/clock",
        "24-hour time",
        "settings.bar.clock-24h",
        &["am pm"],
    ),
    row(
        "status-bar/clock",
        "Show seconds",
        "settings.bar.clock-seconds",
        &[],
    ),
    row(
        "status-bar/clock",
        "Flash the time separators",
        "settings.bar.flash",
        &["blink", "colon"],
    ),
    row(
        "status-bar/clock",
        "Show the date",
        "settings.bar.show-date",
        &["day", "when space allows"],
    ),
    row(
        "status-bar/clock",
        "Date in the tooltip",
        "settings.bar.date",
        &["tooltip", "date format", "long date", "short date"],
    ),
    row(
        "status-bar/clock",
        "Show the day of the week",
        "settings.bar.weekday",
        &["weekday"],
    ),
    row(
        "status-bar/weather",
        "Weather",
        "settings.bar.weather",
        &["temperature", "forecast", "open-meteo"],
    ),
    row(
        "status-bar/weather",
        "Weather place",
        "settings.bar.weather-place",
        &["place", "city", "town", "location"],
    ),
    row(
        "status-bar/items",
        "Status bar items",
        "settings.bar.order",
        &["icons", "order", "reorder"],
    ),
    // the apps' rows are listed as they're seen: the sub-page itself
    page(
        "status-bar/items",
        "Allow in the Status Bar",
        &["tray", "system tray", "app icons"],
    ),
    // Dock
    row(
        "desktop",
        "Dock size",
        "settings.dock.magnify",
        &["icon size", "magnification"],
    ),
    row(
        "desktop",
        "Hide the dock automatically",
        "settings.dock.auto-hide",
        &["auto hide"],
    ),
    row(
        "desktop",
        "Dots under open apps",
        "settings.dock.dots",
        &["indicators"],
    ),
    row(
        "desktop",
        "App names over icons",
        "settings.dock.names",
        &["tooltips"],
    ),
    row(
        "desktop",
        "Animate the dock",
        "settings.dock.animations",
        &["bounce"],
    ),
    // Quick Settings
    page(
        "status-bar/quick-settings",
        "Quick Settings",
        &["controls", "toggles", "wifi", "bluetooth", "customize"],
    ),
    row(
        "status-bar/quick-settings",
        "Reset the Quick Settings layout",
        "settings.quick.reset-row",
        &["customize"],
    ),
    // Launcher
    row(
        "launcher",
        "Show applications as",
        "settings.launcher.layout",
        &["grid", "list"],
    ),
    row(
        "launcher",
        "Clipboard history",
        "settings.launcher.clipboard",
        &["copy", "paste"],
    ),
    row(
        "launcher",
        "Quick keys",
        "settings.launcher.keys.none",
        &["shortcuts", "hotkeys"],
    ),
    // Drop
    row(
        "drop",
        "Tiles a row",
        "settings.drop.columns",
        &["columns", "grid size"],
    ),
    row(
        "drop",
        "Copy the link",
        "settings.drop.copy-link",
        &["clipboard", "upload link", "recently shared"],
    ),
    row(
        "drop",
        "Send to Phone",
        "settings.drop.phone",
        &["kde connect", "phone", "share"],
    ),
    row(
        "drop/imgur",
        "Imgur",
        "settings.drop.imgur.id.row",
        &["client id", "upload pictures"],
    ),
    row(
        "drop/sftp",
        "SFTP server",
        "settings.drop.sftp.host.row",
        &["ssh", "scp", "upload", "server"],
    ),
    row(
        "drop/s3",
        "S3 bucket",
        "settings.drop.s3.bucket.row",
        &["aws", "r2", "minio", "upload", "bucket"],
    ),
    // Passwords
    row(
        "passwords",
        "Server",
        "settings.passwords.server.row",
        &["vaultwarden", "self-hosted", "url", "account"],
    ),
    row(
        "passwords",
        "Lock after",
        "settings.passwords.lock-after.row",
        &["auto-lock", "timeout", "lock"],
    ),
    row(
        "passwords",
        "Fill passwords in apps",
        "settings.passwords.fill",
        &["autofill", "input method", "password field"],
    ),
    // About
    page("about/files", "Settings files", &["json", "config files"]),
    // System
    page(
        "about/advanced",
        "Configuration folder",
        &["flake", "repository", "nixos-config"],
    ),
    page(
        "apps/installed",
        "Installed apps",
        &["uninstall", "flatpak", "remove", "installed packages"],
    ),
    row(
        "default-apps/file-types",
        "File types",
        "settings.defaults.type-row",
        &["extension", "kind of file", "reset"],
    ),
];

/// The page and sub-page a place is on, for a result's second line.
fn where_is(place: &str) -> String {
    let (page, sub) = match place.split_once('/') {
        Some((page, sub)) => (page, Some(sub)),
        None => (place, None),
    };
    let page = crate::pages::named(page).map_or_else(|| page.to_string(), crate::page_title);
    match sub {
        Some(sub) => {
            let mut sub = sub.to_string();
            if let Some(first) = sub.get_mut(0..1) {
                first.make_ascii_uppercase();
            }
            format!("{page} › {sub}")
        }
        None => page,
    }
}

/// A result: where it goes and what it lights.
#[derive(Clone, Debug, PartialEq)]
pub struct Found {
    pub place: String,
    pub title: String,
    pub row: Option<String>,
    pub path: String,
    score: f32,
}

/// How well a title matches `query` (lowercased), 0 … 1, or `None`: the
/// launcher's fuzzy match on the title, and on another word for it at
/// half the weight, so a title always beats a synonym.
fn score(query: &str, title: &str, words: &[&str]) -> Option<f32> {
    let by_title = fuzzy(query, title);
    let by_word = words
        .iter()
        .filter_map(|word| fuzzy(query, word))
        .fold(None, |best: Option<f32>, score| {
            Some(best.map_or(score, |b| b.max(score)))
        })
        .map(|score| score * 0.5);
    match (by_title, by_word) {
        (Some(title), Some(word)) => Some(title.max(word)),
        (title, word) => title.or(word),
    }
}

/// What `query` finds, best first; nothing for an empty query.
pub fn find(query: &str, system: &[(String, String, String)]) -> Vec<Found> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return Vec::new();
    }
    let mut found: Vec<Found> = entries()
        .filter_map(|entry| {
            Some(Found {
                place: entry.place.to_string(),
                title: entry.title.to_string(),
                row: entry.row.map(String::from),
                path: where_is(entry.place),
                score: score(&query, entry.title, entry.words)?,
            })
        })
        .collect();
    // the System pages' settings, by their labels
    for (page, id, label) in system {
        if let Some(score) = score(&query, label, &[]) {
            found.push(Found {
                place: page.clone(),
                title: label.clone(),
                row: Some(format!("settings.system.row.{id}")),
                path: where_is(page),
                score,
            });
        }
    }
    found.sort_by(|a, b| b.score.total_cmp(&a.score));
    found.dedup_by(|a, b| a.title == b.title && a.place == b.place);
    found.truncate(30);
    found
}

/// The configuration's settings now, as (page, id, label).
pub fn system_entries() -> Vec<(String, String, String)> {
    Snapshot::read()
        .entries
        .iter()
        .filter(|entry| !entry.page.is_empty())
        .map(|entry| (entry.page.clone(), entry.id.clone(), entry.label.clone()))
        .collect()
}

/// How long a row stays lit after a search led to it.
pub const LIT: Duration = Duration::from_millis(1600);

thread_local! {
    /// The row a search led to, and when.
    static LIT_ROW: RefCell<Option<(String, Instant)>> = const { RefCell::new(None) };
    /// Ctrl+F asked for the field.
    static FOCUS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Opens a result: its place, the row lit, the search put away.
pub fn open(found: &Found) {
    crate::go(&found.place);
    LIT_ROW.with_borrow_mut(|lit| *lit = found.row.clone().map(|row| (row, Instant::now())));
    TextInput::set_text(FIELD, "");
    // the light goes out by itself
    emaki_kit::motion::redraw_after::<crate::Navigation>(LIT);
}

/// The row to light while drawing, if it's still lit.
pub fn lit_row() -> Option<String> {
    LIT_ROW.with_borrow_mut(|lit| {
        if lit.as_ref().is_some_and(|(_, at)| at.elapsed() >= LIT) {
            *lit = None;
        }
        lit.as_ref().map(|(row, _)| row.clone())
    })
}

/// Ctrl+F: the field takes the keys at the next drawing.
pub fn focus() {
    FOCUS.set(true);
}

pub fn take_focus() -> bool {
    FOCUS.replace(false)
}

/// Escape: clears what's typed, if anything was.
pub fn clear() -> bool {
    if TextInput::text(FIELD).is_empty() {
        return false;
    }
    TextInput::set_text(FIELD, "");
    drop(crate::Navigation::write());
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_page_and_sub_page_can_be_found() {
        for (_, def) in crate::pages::all() {
            for sub in def.subs.0 {
                let place = format!("{}/{sub}", def.name);
                assert!(
                    entries().any(|entry| entry.place == place),
                    "{place} has nothing to find"
                );
            }
        }
        for entry in entries() {
            assert!(
                crate::place_named(entry.place).is_some(),
                "{} isn't a page or sub-page",
                entry.place
            );
        }
    }

    #[test]
    fn every_row_found_is_a_row_on_its_page() {
        // the row ids written in the pages' source, whatever builds them
        let mut source = String::new();
        let mut folders = vec![std::path::PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src"
        ))];
        while let Some(folder) = folders.pop() {
            for entry in std::fs::read_dir(folder).unwrap().flatten() {
                let path = entry.path();
                if path.is_dir() {
                    folders.push(path);
                } else if path.extension().is_some_and(|ext| ext == "rs")
                    && path.file_name().is_some_and(|name| name != "search.rs")
                {
                    source.push_str(&std::fs::read_to_string(path).unwrap());
                }
            }
        }
        for entry in INDEX {
            if let Some(row) = entry.row {
                // a field's row is its id and `.row`
                let field = row.strip_suffix(".row").unwrap_or(row);
                assert!(
                    source.contains(&format!("\"{row}\""))
                        || source.contains(&format!("\"{field}\"")),
                    "{row} ({}) is no row in the pages",
                    entry.title
                );
            }
        }
    }

    #[test]
    fn a_few_letters_find_a_page() {
        // initials and letters in order, as in the launcher
        assert_eq!(find("qs", &[])[0].place, "status-bar/quick-settings");
        assert!(find("dsplys", &[]).iter().any(|f| f.place == "displays"));
    }

    #[test]
    fn titles_beat_other_words() {
        let found = find("dark", &[]);
        assert_eq!(found[0].place, "appearance");
        let seconds = find("seconds", &[]);
        assert_eq!(seconds[0].place, "status-bar/clock");
        assert_eq!(
            seconds[0].row.as_deref(),
            Some("settings.bar.clock-seconds")
        );
        assert_eq!(seconds[0].path, "Status Bar › Clock");
        assert!(
            find("wifi", &[])
                .iter()
                .any(|f| f.place == "status-bar/quick-settings")
        );
        assert!(find("  ", &[]).is_empty());
        let system = vec![(
            String::from("general"),
            String::from("hostName"),
            String::from("Computer name"),
        )];
        assert!(
            find("computer n", &system)
                .iter()
                .any(|f| f.row.as_deref() == Some("settings.system.row.hostName"))
        );
    }
}
