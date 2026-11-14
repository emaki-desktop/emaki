//! The notification center's history, kept across restarts in the state
//! file `notifications.json`: each notification's words, pictures, sender
//! and time ([`KeptNotification`]), oldest first, at most [`MOST`] of them
//! and none older than [`LONGEST`]. Transient ones aren't kept.
//!
//! At startup [`restore`] hands the file to amane, which puts the
//! notifications back without their actions, since their senders may be
//! gone. [`watch`] then saves the list whenever it changes, and remembers
//! each sending app for Emaki Settings' Notifications page.

use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use amane::serde::{Deserialize, Serialize};
use amane::{KeptNotification, Notification, Notifications, Service};
use emaki_kit::sync::Locked as _;

use super::state::{self, State};

/// The most notifications kept.
const MOST: usize = 100;

/// How long one is kept.
const LONGEST: Duration = Duration::from_secs(14 * 24 * 60 * 60);

#[derive(Serialize, Deserialize, Default, Clone, PartialEq, Debug)]
#[serde(crate = "amane::serde", transparent)]
struct History(Vec<KeptNotification>);

impl State for History {
    const FILE: &'static str = "notifications.json";
    const VERSION: u32 = 1;
}

// what's in the file, so an unchanged list isn't written again
static SAVED: Mutex<Option<History>> = Mutex::new(None);

/// Puts back what was kept when the shell last stopped. Once, at startup,
/// before [`watch`].
// the newest notification seen in the center, for the bar's dot and the
// indicator light, read from either's thread
static SEEN: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

/// The center was opened with `newest` the newest notification in it:
/// everything up to it counts as seen.
pub fn mark_seen(newest: u32) {
    SEEN.fetch_max(newest, std::sync::atomic::Ordering::Relaxed);
    super::feedback::led::nudge();
}

/// How many arrived since the center was last opened, from any thread;
/// what was restored from before a restart was there already. Only
/// critical ones when `critical_only`.
pub fn unseen(critical_only: bool) -> usize {
    let seen = SEEN.load(std::sync::atomic::Ordering::Relaxed);
    Notifications::read()
        .list()
        .iter()
        .filter(|n| n.id() > seen && !n.restored())
        .filter(|n| !critical_only || n.urgency() == amane::Urgency::Critical)
        .count()
}

pub fn restore() {
    let Some(History(kept)) = state::load::<History>() else {
        return;
    };
    let kept = fit(kept.into_iter().map(without_lost_image).collect(), now());
    *SAVED.locked() = Some(History(kept.clone()));
    Notifications::restore(kept);
}

/// Saves the history whenever the notifications change. Once, at startup.
pub fn watch() {
    Notifications::subscribe(|notifications| {
        // another daemon has the notifications; the file is left for when
        // the shell has them again
        if !notifications.running() {
            return;
        }
        for n in notifications.list() {
            if !n.restored() {
                remember(n);
            }
        }
        let history = History(kept(notifications.list(), now()));
        let mut saved = SAVED.locked();
        if saved.as_ref() != Some(&history) {
            state::save(&history);
            *saved = Some(history);
        }
    })
    .forever();
}

fn remember(n: &Notification) {
    let key = emaki_kit::settings::notification_app_key(n.desktop_entry(), n.app_name());
    emaki_kit::settings::remember_notifier(&key, n.app_name());
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// What of `list` (oldest first) is kept at `now`.
fn kept(list: &[Notification], now: u64) -> Vec<KeptNotification> {
    let kept = list
        .iter()
        .filter(|n| !n.transient())
        .map(Notification::kept)
        .collect();
    fit(kept, now)
}

/// `kept` (oldest first) without what's too old, and only the newest
/// [`MOST`].
fn fit(mut kept: Vec<KeptNotification>, now: u64) -> Vec<KeptNotification> {
    kept.retain(|k| k.received.saturating_add(LONGEST.as_secs()) >= now);
    let extra = kept.len().saturating_sub(MOST);
    kept.drain(..extra);
    kept
}

/// `k` without its image when the file has gone, as one amane saved in the
/// runtime folder does at a reboot; the card shows the icon instead.
fn without_lost_image(mut k: KeptNotification) -> KeptNotification {
    let path = k.image.strip_prefix("file://").unwrap_or(&k.image);
    if !k.image.is_empty() && path.starts_with('/') && !Path::new(path).exists() {
        k.image.clear();
    }
    k
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::state::tests::in_folder;

    fn note(summary: &str, received: u64) -> KeptNotification {
        KeptNotification {
            app_name: String::from("Mail"),
            summary: String::from(summary),
            body: String::new(),
            icon: String::new(),
            image: String::new(),
            urgency: 1,
            category: String::new(),
            desktop_entry: String::from("org.gnome.Evolution"),
            received,
        }
    }

    #[test]
    fn the_history_round_trips() {
        in_folder("notification-history", &[], |_| {
            let history = History(vec![note("one", 10), note("two", 20)]);
            state::save(&history);
            assert_eq!(state::load::<History>(), Some(history));
        });
    }

    #[test]
    fn only_the_newest_hundred_and_none_too_old_are_kept() {
        let now = LONGEST.as_secs() + 1_000;
        let mut list = vec![note("too old", 999)];
        list.extend((0..120).map(|i| note(&i.to_string(), 1_000 + i)));
        let kept = fit(list, now);
        assert_eq!(kept.len(), MOST);
        assert_eq!(kept.first().unwrap().summary, "20");
        assert_eq!(kept.last().unwrap().summary, "119");
    }

    #[test]
    fn a_lost_image_is_dropped_and_others_stay() {
        let mut gone = note("gone", 1);
        gone.image = String::from("/nonexistent/emaki-notifications/1-0.png");
        assert!(without_lost_image(gone).image.is_empty());

        let mut here = note("here", 1);
        here.image = String::from("/");
        assert_eq!(without_lost_image(here).image, "/");

        let mut named = note("named", 1);
        named.image = String::from("avatar-default");
        assert_eq!(without_lost_image(named).image, "avatar-default");
    }
}
