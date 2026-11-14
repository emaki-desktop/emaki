//! What the shell tells of as the system changes by itself, so a device
//! plugged in or a network lost never goes by unseen. One rule picks how:
//!
//! - **A capsule** (the on-screen display's [`echo`]) for a change that
//!   needs nothing: a device connected or gone, the charger, a display. It
//!   fades by itself, makes no sound, and never stays in the center.
//! - **A notification** ([`notify`]) for what offers something to do or
//!   needs a look: a drive to open or eject, a battery running low, a
//!   failure. Ones that are only news are sent transient, so the center
//!   doesn't keep them.
//!
//! Each kind can be turned off in Settings → Notifications
//! ([`Feedback`]). Nothing is told in the first moments after the shell
//! starts or the computer wakes, when devices come back in a burst. See
//! `docs/user/features/notifications.md`.

pub mod bluetooth;
pub mod displays;
pub mod led;
pub mod levels;
pub mod network;
pub mod power;
pub mod screenshots;
pub mod storage;
pub mod thermals;
pub mod usb;

use std::collections::VecDeque;
use std::sync::{Mutex, Once, PoisonError};
use std::time::{Duration, Instant};

use amane::{Logind, Notify, NotifyHandle, Service};
use emaki_kit::settings::{Feedback, notification_rules, sound};

/// How long after starting or waking nothing is told.
const SETTLE: Duration = Duration::from_secs(5);

/// The least time between two connection sounds.
const SOUND_GAP: Duration = Duration::from_millis(400);

// until when nothing is told
static QUIET_UNTIL: Mutex<Option<Instant>> = Mutex::new(None);

/// Starts every source, once.
pub fn start() {
    static START: Once = Once::new();
    START.call_once(|| {
        quiet_for(SETTLE);
        follow_sleep();
        bluetooth::watch();
        usb::watch();
        power::watch();
        displays::watch();
        network::watch();
        screenshots::watch();
        storage::watch();
        thermals::watch();
        led::watch();
    });
}

/// Whether the shell tells of `kind` now: turned on, and not settling.
pub fn shows(kind: Feedback) -> bool {
    !settling() && notification_rules().shows(kind)
}

/// What a capsule says: a glyph, a name, and a quieter detail after it,
/// like "Connected · 80 %".
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Echo {
    pub glyph: char,
    pub name: String,
    pub detail: String,
}

emaki_kit::marker! {
    /// An echo is waiting: the on-screen display draws again to show it.
    pub Echoes
}

/// The most echoes waiting their turn; past it the oldest is dropped.
const MOST_WAITING: usize = 3;

// echoes posted from any thread, which the on-screen display takes in turn
static WAITING: Mutex<VecDeque<Echo>> = Mutex::new(VecDeque::new());

/// A capsule for `kind`, if it shows now; the on-screen display shows it
/// once what it shows has run out ([`take_echo`]). The same one already
/// waiting isn't queued twice.
pub fn echo(kind: Feedback, echo: Echo) {
    if !shows(kind) {
        return;
    }
    {
        let mut waiting = WAITING.lock().unwrap_or_else(PoisonError::into_inner);
        if waiting.contains(&echo) {
            return;
        }
        if waiting.len() >= MOST_WAITING {
            waiting.pop_front();
        }
        waiting.push_back(echo);
    }
    Echoes::changed();
}

/// The next echo waiting, for the on-screen display: none while `busy`
/// (it shows something), and every one dropped while `drop_all` (Quick
/// Settings is open, whose rows already show what changed).
pub fn take_echo(busy: bool, drop_all: bool) -> Option<Echo> {
    let mut waiting = WAITING.lock().unwrap_or_else(PoisonError::into_inner);
    if drop_all {
        waiting.clear();
        return None;
    }
    if busy {
        return None;
    }
    waiting.pop_front()
}

/// A notification for `kind`, sent under the shell's entry for it, if it
/// shows now. A warning that matters (`always`) is sent while settling too,
/// like a battery found low on waking.
pub fn notify(kind: Feedback, notify: Notify, always: bool) -> Option<NotifyHandle> {
    let on = notification_rules().shows(kind);
    if !on || (!always && settling()) {
        return None;
    }
    Some(notify.desktop_entry(kind.entry()).icon(icon(kind)).send())
}

/// The sound for a device connecting or going, when Settings asks for it.
pub fn connection_sound(added: bool) {
    if settling() || !notification_rules().connection_sounds {
        return;
    }
    let name = if added {
        "device-added"
    } else {
        "device-removed"
    };
    emaki_kit::sounds::play(name, sound().volume, SOUND_GAP);
}

/// The theme icon a kind's notifications carry, unless one is given.
fn icon(kind: Feedback) -> &'static str {
    match kind {
        Feedback::Bluetooth => "bluetooth",
        Feedback::UsbDevices => "drive-removable-media-usb",
        Feedback::Storage => "drive-removable-media",
        Feedback::Power => "battery",
        Feedback::Network => "network-wireless",
        Feedback::Displays => "video-display",
        Feedback::Screenshots => "applets-screenshooter",
        Feedback::Thermals => "temperature-symbolic",
    }
}

fn settling() -> bool {
    QUIET_UNTIL
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .is_some_and(|until| Instant::now() < until)
}

fn quiet_for(span: Duration) {
    *QUIET_UNTIL.lock().unwrap_or_else(PoisonError::into_inner) = Some(Instant::now() + span);
}

/// Quiet while the computer goes to sleep, and for a while after it wakes,
/// as Bluetooth and USB devices come back one by one.
fn follow_sleep() {
    let wakeups = Mutex::new(Logind::read().wakeups());
    Logind::subscribe(move |logind| {
        let woke = logind.wakeups();
        let mut seen = wakeups.lock().unwrap_or_else(PoisonError::into_inner);
        if logind.preparing_for_sleep() || woke != *seen {
            *seen = woke;
            quiet_for(SETTLE);
        }
    })
    .forever();
}
