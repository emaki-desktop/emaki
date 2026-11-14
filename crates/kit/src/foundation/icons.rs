//! Material Symbols codepoints (the font stack is token `sys.type.icon`),
//! and the pickers that choose one for a device or a reading, so the bar,
//! Quick Settings and the notifications all agree. Draw them with
//! [`layout::icon`](crate::layout::icon), or, where a control can also show
//! a mark the font hasn't got, as a [`Glyph`] with [`glyph`].

use amane::{
    AudioKind, BatteryState, Center, Color, Column, DeviceKind, PowerDevice, Radius, Rectangle,
    Row, Size,
};

use crate::layout::{boxed, fixed_width, frame, icon};

/// A grid of squares (`apps`): the app launcher.
pub const ICON_APPS: char = '\u{e5c3}';
/// A speaker with waves (`volume_up`): sound on, and the built-in speakers.
pub const ICON_VOLUME_UP: char = '\u{e050}';
/// A struck-out speaker (`volume_off`): muted or silent.
pub const ICON_VOLUME_OFF: char = '\u{e04f}';
/// A speaker with one wave (`volume_down`): quiet.
pub const ICON_VOLUME_DOWN: char = '\u{e04d}';
/// A padlock (`lock`).
pub const ICON_LOCK: char = '\u{e897}';
/// A fingerprint (`fingerprint`).
pub const ICON_FINGERPRINT: char = '\u{e90d}';
/// A crescent moon (`bedtime`): hibernation.
pub const ICON_HIBERNATE: char = '\u{ea40}';
/// A thermometer (`thermostat`): a temperature readout.
pub const ICON_THERMOSTAT: char = '\u{f076}';
/// A three-blade fan (`mode_fan`): fan speed.
pub const ICON_FAN: char = '\u{ec17}';
/// The gear (`settings`).
pub const ICON_SETTINGS: char = '\u{e8b8}';
/// A check mark (`check`).
pub const ICON_CHECK: char = '\u{e5ca}';
/// A warning triangle (`warning`).
pub const ICON_WARNING: char = '\u{e002}';
/// An X (`close`).
pub const ICON_CLOSE: char = '\u{e5cd}';
/// An arrow curling back (`undo`): put back what was just done.
pub const ICON_UNDO: char = '\u{e166}';
/// Two chevrons pointing right (`keyboard_double_arrow_right`): more that
/// didn't fit.
pub const ICON_MORE: char = '\u{eac9}';
/// A minus sign (`remove`).
pub const ICON_REMOVE: char = '\u{e15b}';
/// A plus sign (`add`).
pub const ICON_ADD: char = '\u{e145}';
/// Two arrows out to opposite corners (`open_in_full`): resize.
pub const ICON_RESIZE: char = '\u{f1ce}';
/// A mouse (`mouse`).
pub const ICON_MOUSE: char = '\u{e323}';
/// A keyboard (`keyboard`).
pub const ICON_KEYBOARD: char = '\u{e312}';
/// Headphones (`headphones`).
pub const ICON_HEADPHONES: char = '\u{e310}';
/// A game controller (`sports_esports`).
pub const ICON_GAMEPAD: char = '\u{ea28}';
/// A struck-out microphone (`mic_off`).
pub const ICON_MIC_OFF: char = '\u{e02b}';
/// A standing speaker (`speaker`).
pub const ICON_SPEAKER: char = '\u{e32d}';
/// A phone (`mobile`).
pub const ICON_PHONE: char = '\u{e32c}';
/// A laptop (`computer`).
pub const ICON_COMPUTER: char = '\u{e30a}';
/// A screen on its stand (`desktop_windows`).
pub const ICON_DISPLAY: char = '\u{e30c}';
/// A phone beside a screen (`devices_other`): a device of no known kind.
pub const ICON_DEVICE_OTHER: char = '\u{e337}';
/// The USB trident (`usb`): a USB device of no better-known kind.
pub const ICON_USB: char = '\u{e1e0}';
/// A printer (`print`).
pub const ICON_PRINTER: char = '\u{e8ad}';
/// A drive (`hard_drive`): a removable disk.
pub const ICON_DRIVE: char = '\u{f80e}';
/// An arrow up over a bar (`eject`): a drive let go.
pub const ICON_EJECT: char = '\u{e8fb}';
/// The Bluetooth rune (`bluetooth`).
pub const ICON_BLUETOOTH: char = '\u{e1a7}';
/// The Bluetooth rune with dots either side (`bluetooth_connected`).
pub const ICON_BLUETOOTH_CONNECTED: char = '\u{e1a8}';
/// The struck-out Bluetooth rune (`bluetooth_disabled`).
pub const ICON_BLUETOOTH_OFF: char = '\u{e1a9}';
/// A bell (`notifications`).
pub const ICON_NOTIFICATIONS: char = '\u{e7f4}';
/// A crescent moon (`bedtime`): do not disturb.
pub const ICON_DND: char = '\u{ef44}';
/// A chevron pointing left (`chevron_left`): back.
pub const ICON_CHEVRON_LEFT: char = '\u{e5cb}';
/// A chevron pointing right (`chevron_right`): more inside.
pub const ICON_CHEVRON_RIGHT: char = '\u{e5cc}';
/// A bin (`delete`): removes something for good.
pub const ICON_DELETE: char = '\u{e872}';
/// An open eye (`visibility`): shows a hidden password.
pub const ICON_VISIBILITY: char = '\u{e8f4}';
/// A struck-out eye (`visibility_off`): hides it again.
pub const ICON_VISIBILITY_OFF: char = '\u{e8f5}';
/// An i in a circle (`info`).
pub const ICON_INFO: char = '\u{e88e}';
/// A chevron pointing down (`expand_more`).
pub const ICON_EXPAND_MORE: char = '\u{e5cf}';
/// Sliders (`tune`): settings for one thing.
pub const ICON_TUNE: char = '\u{e429}';
/// The power symbol (`power_settings_new`).
pub const ICON_POWER: char = '\u{f8c7}';
/// A dashed frame (`screenshot_region`): a screenshot.
pub const ICON_SCREENSHOT: char = '\u{f7d2}';
/// A dim sun (`brightness_low`).
pub const ICON_BRIGHTNESS_LOW: char = '\u{e1ad}';
/// A bright sun (`brightness_high`).
pub const ICON_BRIGHTNESS_HIGH: char = '\u{e1ac}';
/// A sun on the horizon (`wb_twilight`): night light.
pub const ICON_NIGHT_LIGHT: char = '\u{e1c6}';
/// A half-filled circle (`contrast`): dark mode.
pub const ICON_DARK_MODE: char = '\u{eb37}';
/// An airplane (`airplanemode_active`).
pub const ICON_AIRPLANE: char = '\u{e53d}';
/// A key (`vpn_key`): a VPN.
pub const ICON_VPN: char = '\u{e0da}';
/// Waves out from a point (`wifi_tethering`): a hotspot.
pub const ICON_HOTSPOT: char = '\u{e1e2}';
/// Linked boxes (`lan`): a wired network.
pub const ICON_WIRED: char = '\u{eb2f}';
/// A lamp giving off light (`backlight_high`): the keyboard light.
pub const ICON_KEYBOARD_LIGHT: char = '\u{f7ed}';
/// An arrow up over a bar (`keyboard_capslock`): Caps Lock.
pub const ICON_CAPS_LOCK: char = '\u{e318}';
/// The digits 123 (`123`): Num Lock.
pub const ICON_NUM_LOCK: char = '\u{eb8d}';
/// A video camera (`videocam`).
pub const ICON_CAMERA: char = '\u{e04b}';
/// A screen with an arrow out of it (`screen_share`).
pub const ICON_SCREEN_SHARE: char = '\u{e0e2}';
/// A music note (`music_note`).
pub const ICON_MUSIC: char = '\u{e405}';
/// A play triangle (`play_arrow`).
pub const ICON_PLAY: char = '\u{e037}';
/// Two bars (`pause`).
pub const ICON_PAUSE: char = '\u{e034}';
/// Skip forward (`skip_next`).
pub const ICON_SKIP_NEXT: char = '\u{e044}';
/// Skip back (`skip_previous`).
pub const ICON_SKIP_PREVIOUS: char = '\u{e045}';
/// A crescent in a circle (`mode_standby`): sleep.
pub const ICON_SLEEP: char = '\u{f037}';
/// A circular arrow (`restart_alt`): restart.
pub const ICON_RESTART: char = '\u{f053}';
/// An arrow out of a door (`logout`): log out.
pub const ICON_LOG_OUT: char = '\u{e9ba}';
/// Tiles with one turned (`widgets`): apps running in the background.
pub const ICON_BACKGROUND_APPS: char = '\u{e1bd}';
/// A microphone (`mic`).
pub const ICON_MIC: char = '\u{e029}';
/// A coffee cup (`coffee`): keep the screen awake.
pub const ICON_KEEP_AWAKE: char = '\u{efef}';
/// A struck-out screen (`desktop_access_disabled`): turn the screen off.
pub const ICON_SCREEN_OFF: char = '\u{e99d}';
/// A magnifying glass (`search`).
pub const ICON_SEARCH: char = '\u{e8b6}';
/// The four sums (`calculate`): a calculator.
pub const ICON_CALCULATE: char = '\u{ea5f}';
/// A box with an arrow out of it (`open_in_new`).
pub const ICON_OPEN_IN_NEW: char = '\u{e89e}';
const ICON_MIC_EXTERNAL: char = '\u{ef5a}';
const ICON_HEADSET_MIC: char = '\u{e311}';
const ICON_TV: char = '\u{e333}';
const ICON_CAR: char = '\u{e531}';
const ICON_WEBCAM: char = '\u{e04b}';

/// The glyph for what a sound goes to (or, with `input`, comes from), so the
/// bar, the Sound card and page and its device rows all agree. Built-in
/// devices show the plain speaker / microphone.
pub fn audio_glyph(kind: AudioKind, input: bool) -> char {
    match kind {
        AudioKind::Internal if input => ICON_MIC,
        AudioKind::Internal => ICON_VOLUME_UP,
        AudioKind::Speaker => ICON_SPEAKER,
        AudioKind::Headphones => ICON_HEADPHONES,
        AudioKind::Headset => ICON_HEADSET_MIC,
        AudioKind::Display => ICON_TV,
        AudioKind::Car => ICON_CAR,
        AudioKind::Phone => ICON_PHONE,
        AudioKind::Webcam => ICON_WEBCAM,
        AudioKind::Microphone => ICON_MIC_EXTERNAL,
    }
}

/// The glyph for an output's level: off while muted or silent, down while
/// quiet on the built-in speakers, else the device's own (`audio_glyph`).
pub fn volume_glyph(muted: bool, volume: u8, kind: AudioKind) -> char {
    let loud = audio_glyph(kind, false);
    if muted || volume == 0 {
        ICON_VOLUME_OFF
    } else if volume < 50 && kind == AudioKind::Internal {
        ICON_VOLUME_DOWN
    } else {
        loud
    }
}

/// Horizontal batteries (`battery_android_0` … `_6`, `_full`, `_bolt`), the
/// laptop shape; the fill shows the charge.
const ICON_BATTERY_LEVELS: [char; 7] = [
    '\u{f30d}', '\u{f30c}', '\u{f30b}', '\u{f30a}', '\u{f309}', '\u{f308}', '\u{f307}',
];
/// A full horizontal battery (`battery_android_full`).
pub const ICON_BATTERY_FULL: char = '\u{f304}';
const ICON_BATTERY_BOLT: char = '\u{f305}';

/// The battery glyph for a charge: a bolt while charging, otherwise filled to
/// the nearest of the glyph's six bars (full from 95 %).
pub fn battery_glyph(percent: u8, charging: bool) -> char {
    if charging {
        ICON_BATTERY_BOLT
    } else if percent >= 95 {
        ICON_BATTERY_FULL
    } else {
        let bars = (usize::from(percent) * 6 + 50) / 100;
        ICON_BATTERY_LEVELS[bars.min(6)]
    }
}

/// Signal wedges (`network_wifi_1_bar` … `_3_bar`, `network_wifi`, then the
/// filled `signal_wifi_4_bar`), drawn like the battery glyphs: the filled part
/// shows the signal.
const ICON_WIFI_LEVELS: [char; 5] = ['\u{ebe4}', '\u{ebd6}', '\u{ebe1}', '\u{e1ba}', '\u{e1d8}'];
/// The filled wedge (`signal_wifi_4_bar`): full signal, and Wi-Fi in general.
pub const ICON_WIFI: char = ICON_WIFI_LEVELS[4];
/// The empty wedge (`signal_wifi_0_bar`): the radio is on but nothing is joined.
pub const ICON_WIFI_NONE: char = '\u{f0b0}';
/// The struck-out wedge (`signal_wifi_off`): the radio is off.
pub const ICON_WIFI_OFF: char = '\u{e1da}';
/// A plug on a cable (`cable`).
pub const ICON_CABLE: char = '\u{efe6}';
/// Three linked boxes (`lan`): a link that is neither Wi‑Fi nor a cable.
pub const ICON_LAN: char = '\u{eb2f}';
/// A folder (`folder`).
pub const ICON_FOLDER: char = '\u{e2c7}';
/// Stacked sheets (`layers`).
pub const ICON_LAYERS: char = '\u{e53b}';
/// A clipboard (`content_paste`): paste.
pub const ICON_CONTENT_PASTE: char = '\u{e14f}';
/// Two sheets (`content_copy`): copy.
pub const ICON_CONTENT_COPY: char = '\u{e14d}';
/// Three dots in a row (`more_horiz`): more.
pub const ICON_MORE_HORIZ: char = '\u{e5d3}';
/// Three dots in a column (`more_vert`): more.
pub const ICON_MORE_VERT: char = '\u{e5d4}';
/// The two bars a reorderable list item is dragged by (`drag_handle`).
pub const ICON_DRAG_HANDLE: char = '\u{e25d}';
/// A head and shoulders (`person`).
pub const ICON_PERSON: char = '\u{e7fd}';
/// A picture (`image`).
pub const ICON_IMAGE: char = '\u{e3f4}';
/// A star (`star`).
pub const ICON_STAR: char = '\u{e838}';
/// An envelope (`mail`).
pub const ICON_MAIL: char = '\u{e158}';
/// Two arrows, left and right (`swap_horiz`): swap.
pub const ICON_SWAP: char = '\u{e8d4}';
/// A lightning bolt (`bolt`).
pub const ICON_BOLT: char = '\u{ea0b}';
/// A clock with an arrow around it (`history`).
pub const ICON_HISTORY: char = '\u{e889}';
/// Stacked drives (`storage`).
pub const ICON_STORAGE: char = '\u{e1db}';
/// A clock with a circling arrow (`update`): updates.
pub const ICON_UPDATE: char = '\u{e923}';
/// `dns`: a server, for services.
pub const ICON_DNS: char = '\u{e875}';
/// `deployed_code`: a package.
pub const ICON_PACKAGE: char = '\u{f720}';
/// `terminal`: a command-line tool.
pub const ICON_TERMINAL: char = '\u{eb8e}';
/// A stopwatch (`timer`).
pub const ICON_TIMER: char = '\u{e425}';
/// `schedule`: a clock face.
pub const ICON_SCHEDULE: char = '\u{e8b5}';
/// `partly_cloudy_day`: the weather.
pub const ICON_PARTLY_CLOUDY: char = '\u{f172}';
/// A sheet with lines (`description`): a document.
pub const ICON_DESCRIPTION: char = '\u{e873}';
/// An arrow up onto a bar (`upload`).
pub const ICON_UPLOAD: char = '\u{e2c6}';
/// An arrow down onto a bar (`download`).
pub const ICON_DOWNLOAD: char = '\u{e2c4}';
/// A cloud with an arrow up (`cloud_upload`).
pub const ICON_CLOUD_UPLOAD: char = '\u{e2c3}';
/// A tray with an arrow into it (`move_to_inbox`): somewhere to drop things.
pub const ICON_TRAY: char = '\u{e168}';
/// Three linked dots (`share`).
pub const ICON_SHARE: char = '\u{e80d}';
/// A paper plane (`send`).
pub const ICON_SEND: char = '\u{e163}';
/// A filled dot (`fiber_manual_record`): recording, or a mark with no picture.
pub const ICON_RECORD: char = '\u{e061}';
/// An exclamation mark in a circle (`error`).
pub const ICON_ERROR: char = '\u{e000}';
/// A check mark in a circle (`check_circle`).
pub const ICON_CHECK_CIRCLE: char = '\u{e86c}';

/// The symbols other programs may ask for by name, for their status items
/// and activities (`ItemIcon::Glyph`): a short list, kebab-case.
pub const NAMED: [(&str, char); 32] = [
    ("dot", ICON_RECORD),
    ("record", ICON_RECORD),
    ("timer", ICON_TIMER),
    ("music", ICON_MUSIC),
    ("play", ICON_PLAY),
    ("pause", ICON_PAUSE),
    ("skip-next", ICON_SKIP_NEXT),
    ("skip-previous", ICON_SKIP_PREVIOUS),
    ("screen-share", ICON_SCREEN_SHARE),
    ("camera", ICON_CAMERA),
    ("mic", ICON_MIC),
    ("upload", ICON_UPLOAD),
    ("download", ICON_DOWNLOAD),
    ("cloud-upload", ICON_CLOUD_UPLOAD),
    ("tray", ICON_TRAY),
    ("share", ICON_SHARE),
    ("send", ICON_SEND),
    ("phone", ICON_PHONE),
    ("folder", ICON_FOLDER),
    ("image", ICON_IMAGE),
    ("mail", ICON_MAIL),
    ("terminal", ICON_TERMINAL),
    ("update", ICON_UPDATE),
    ("history", ICON_HISTORY),
    ("check", ICON_CHECK),
    ("check-circle", ICON_CHECK_CIRCLE),
    ("close", ICON_CLOSE),
    ("warning", ICON_WARNING),
    ("error", ICON_ERROR),
    ("copy", ICON_CONTENT_COPY),
    ("package", ICON_PACKAGE),
    ("settings", ICON_SETTINGS),
];

/// The symbol by its name in [`NAMED`].
pub fn named(name: &str) -> Option<char> {
    NAMED
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, glyph)| *glyph)
}

/// The wedge for a signal strength (0–100): the nearest of its five steps,
/// never fewer than one, since a network you can see has some signal.
pub fn wifi_glyph(strength: u8) -> char {
    let steps = (usize::from(strength) * 5 + 50) / 100;
    ICON_WIFI_LEVELS[steps.clamp(1, 5) - 1]
}

/// The glyph for BlueZ's freedesktop icon name (`audio-headset`, `input-mouse` …).
pub fn device_glyph(icon: &str) -> char {
    match icon {
        i if i.starts_with("audio-head") => ICON_HEADPHONES,
        i if i.starts_with("audio") => ICON_SPEAKER,
        "input-mouse" | "input-tablet" => ICON_MOUSE,
        "input-keyboard" => ICON_KEYBOARD,
        "input-gaming" => ICON_GAMEPAD,
        "phone" => ICON_PHONE,
        "computer" => ICON_COMPUTER,
        _ => ICON_DEVICE_OTHER,
    }
}

const ICON_POWER_SAVER: char = '\u{ea35}';
/// A balance scale (`balance`): the balanced power profile, and the fallback for an unknown
/// one.
pub const ICON_BALANCED: char = '\u{eaf6}';
const ICON_PERFORMANCE: char = '\u{e9e4}';

/// The glyph for a power-profiles-daemon profile.
pub fn power_profile_glyph(profile: &str) -> char {
    match profile {
        "power-saver" => ICON_POWER_SAVER,
        "performance" => ICON_PERFORMANCE,
        _ => ICON_BALANCED,
    }
}

/// The glyph for a UPower peripheral: what it is, else its battery.
pub fn peripheral_glyph(device: &PowerDevice) -> char {
    match device.kind() {
        DeviceKind::Mouse => ICON_MOUSE,
        DeviceKind::Keyboard => ICON_KEYBOARD,
        DeviceKind::Headset => ICON_HEADPHONES,
        DeviceKind::GamingInput => ICON_GAMEPAD,
        _ => battery_glyph(device.percent(), device.state() == BatteryState::Charging),
    }
}

/// The glyph for a Tailscale peer's `OS`: a computer or a phone.
pub fn os_glyph(os: &str) -> char {
    match os.to_ascii_lowercase().as_str() {
        "linux" | "windows" | "macos" | "freebsd" | "openbsd" => ICON_COMPUTER,
        "ios" | "android" => ICON_PHONE,
        _ => ICON_DEVICE_OTHER,
    }
}

/// What a control shows: a font glyph, or a mark drawn from shapes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    /// A glyph from the icon font.
    Font(char),
    /// Tailscale's nine dots.
    Tailscale,
}

impl From<char> for Glyph {
    fn from(glyph: char) -> Self {
        Self::Font(glyph)
    }
}

/// `glyph` at `size` in `color`, as a box exactly its size.
pub fn glyph(glyph: Glyph, size: f32, color: Color) -> Rectangle {
    match glyph {
        Glyph::Font(ch) => {
            let text = icon(ch, size, color);
            let height = match amane::Widget::height(&text) {
                Size::Fixed(height) => height,
                Size::Parent => size,
            };
            frame(fixed_width(&text), height).child(text)
        }
        Glyph::Tailscale => tailscale_mark(size, color),
    }
}

/// Tailscale's mark, rebuilt from its official geometry on a 16-unit grid:
/// dots of radius 5/3 five units apart (upstream's 3:1 spacing to radius),
/// the middle row and the bottom centre solid, the other five at 40 %.
fn tailscale_mark(size: f32, color: Color) -> Rectangle {
    const SOLID: [[bool; 3]; 3] = [
        [false, false, false],
        [true, true, true],
        [false, true, false],
    ];
    let unit = size / 16.0;
    let dot = unit * 10.0 / 3.0;
    let gap = unit * 5.0 - dot;
    let rows = SOLID
        .iter()
        .map(|row| {
            let dots = row
                .iter()
                .map(|solid| {
                    boxed(
                        Rectangle::new()
                            .width(dot)
                            .height(dot)
                            .radius(Radius::Full)
                            .fill(color)
                            .opacity(if *solid { 1.0 } else { 0.4 }),
                    )
                })
                .collect();
            boxed(Row::new(dots).gap(gap))
        })
        .collect();
    frame(size, size)
        .align_child(Center, Center)
        .child(Column::new(rows).gap(gap))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_glyph_follows_the_charge() {
        assert_eq!(battery_glyph(0, false), ICON_BATTERY_LEVELS[0]);
        assert_eq!(battery_glyph(18, false), ICON_BATTERY_LEVELS[1]);
        assert_eq!(battery_glyph(50, false), ICON_BATTERY_LEVELS[3]);
        assert_eq!(battery_glyph(94, false), ICON_BATTERY_LEVELS[6]);
        assert_eq!(battery_glyph(95, false), ICON_BATTERY_FULL);
        assert_eq!(battery_glyph(100, false), ICON_BATTERY_FULL);
        assert_eq!(battery_glyph(18, true), ICON_BATTERY_BOLT);
    }

    #[test]
    fn wifi_glyph_follows_the_signal() {
        assert_eq!(wifi_glyph(0), ICON_WIFI_LEVELS[0]);
        assert_eq!(wifi_glyph(29), ICON_WIFI_LEVELS[0]);
        assert_eq!(wifi_glyph(30), ICON_WIFI_LEVELS[1]);
        assert_eq!(wifi_glyph(50), ICON_WIFI_LEVELS[2]);
        assert_eq!(wifi_glyph(70), ICON_WIFI_LEVELS[3]);
        assert_eq!(wifi_glyph(89), ICON_WIFI_LEVELS[3]);
        assert_eq!(wifi_glyph(90), ICON_WIFI);
        assert_eq!(wifi_glyph(100), ICON_WIFI);
    }

    #[test]
    fn audio_glyph_shows_what_sound_goes_through() {
        assert_eq!(audio_glyph(AudioKind::Internal, false), ICON_VOLUME_UP);
        assert_eq!(audio_glyph(AudioKind::Internal, true), ICON_MIC);
        assert_eq!(audio_glyph(AudioKind::Headphones, false), ICON_HEADPHONES);
        assert_eq!(audio_glyph(AudioKind::Headset, true), ICON_HEADSET_MIC);
        assert_eq!(audio_glyph(AudioKind::Display, false), ICON_TV);
        assert_eq!(audio_glyph(AudioKind::Speaker, false), ICON_SPEAKER);
    }

    #[test]
    fn device_glyph_follows_the_bluez_icon() {
        assert_eq!(device_glyph("audio-headset"), ICON_HEADPHONES);
        assert_eq!(device_glyph("audio-headphones"), ICON_HEADPHONES);
        assert_eq!(device_glyph("audio-card"), ICON_SPEAKER);
        assert_eq!(device_glyph("input-mouse"), ICON_MOUSE);
        assert_eq!(device_glyph("phone"), ICON_PHONE);
        assert_eq!(device_glyph(""), ICON_DEVICE_OTHER);
    }
}
