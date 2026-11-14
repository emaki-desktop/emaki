//! Networks:
//!
//! - **A failed try** to join a Wi‑Fi network or bring up a VPN: a
//!   notification saying why ("Wrong password"), unless Quick Settings is
//!   open, whose row already says so.
//! - **A VPN dropped** while the network under it is still up, and nobody
//!   asked: a notification with Reconnect.
//! - **A sign-in page** (a captive portal, as NetworkManager's connectivity
//!   check finds it): a notification that opens it.
//! - **An Ethernet cable** plugged in or pulled: a capsule.
//!
//! Joining a network on purpose shows nothing: the bar's icon says it.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, Once, PoisonError};
use std::time::{Duration, Instant};

use amane::{Network, Notify, Service, ServiceError};
use emaki_kit::icons::ICON_LAN;
use emaki_kit::settings::Feedback;

use super::{Echo, echo, notify};
use crate::chrome::panel::{Panel, is_open_anywhere};
use crate::services::Wake;
use crate::services::network::connections::Connections;
use crate::services::network::failure_label;

/// How long a VPN change asked for here isn't told of as a drop.
const ASKED_FOR: Duration = Duration::from_secs(15);

/// Opened to reach a captive portal's sign-in page: any plain-HTTP page is
/// sent there, and this is the one NetworkManager's own check uses.
const PORTAL_PROBE: &str = "http://nmcheck.gnome.org/";

static WAKE: Wake = Wake::new();

// VPNs turned on or off from the shell, by UUID, and when
static ASKED: Mutex<Option<HashMap<String, Instant>>> = Mutex::new(None);

/// One reading of what's told of.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Seen {
    /// Each failure's time, by what it's for.
    pub failures: BTreeMap<String, Instant>,
    /// The VPNs up, by UUID, with their names.
    pub vpns: BTreeMap<String, String>,
    pub connected: bool,
    pub portal: bool,
    /// The cable is in and its connection up.
    pub wired: Option<bool>,
}

/// Starts watching, once.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        Network::subscribe(|_| WAKE.nudge()).forever();
        std::thread::spawn(|| {
            let mut before: Option<Seen> = None;
            loop {
                let now = read();
                if let Some(before) = &before {
                    tell(before, &now);
                }
                before = Some(now);
                WAKE.wait(None);
            }
        });
    });
}

/// The VPN `uuid` is being turned on or off from the shell: not a drop.
pub fn asked(uuid: &str) {
    let mut asked = ASKED.lock().unwrap_or_else(PoisonError::into_inner);
    let asked = asked.get_or_insert_with(HashMap::new);
    asked.retain(|_, at| at.elapsed() < ASKED_FOR);
    asked.insert(uuid.to_string(), Instant::now());
}

fn was_asked(uuid: &str) -> bool {
    ASKED
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .as_ref()
        .and_then(|asked| asked.get(uuid))
        .is_some_and(|at| at.elapsed() < ASKED_FOR)
}

fn read() -> Seen {
    let network = Network::read();
    Seen {
        failures: network
            .failures()
            .map(|(key, failure)| (key.to_string(), failure.at))
            .collect(),
        vpns: network
            .vpns()
            .filter(|profile| profile.active)
            .map(|profile| (profile.uuid.clone(), profile.name.clone()))
            .collect(),
        connected: network.connected(),
        portal: network.behind_portal(),
        wired: network.wired().map(|wired| wired.connected),
    }
}

/// The failures new since `before`, by what they're for.
pub fn new_failures<'a>(before: &Seen, now: &'a Seen) -> Vec<&'a str> {
    now.failures
        .iter()
        .filter(|(key, at)| before.failures.get(*key) != Some(at))
        .map(|(key, _)| key.as_str())
        .collect()
}

/// The VPNs that went down while the network under them stayed up.
pub fn dropped<'a>(before: &'a Seen, now: &Seen) -> Vec<(&'a str, &'a str)> {
    if !now.connected {
        return Vec::new();
    }
    before
        .vpns
        .iter()
        .filter(|(uuid, _)| !now.vpns.contains_key(*uuid))
        .map(|(uuid, name)| (uuid.as_str(), name.as_str()))
        .collect()
}

fn tell(before: &Seen, now: &Seen) {
    if !is_open_anywhere(Panel::QuickSettings) {
        for key in new_failures(before, now) {
            failed(key);
        }
    }
    for (uuid, name) in dropped(before, now) {
        if !was_asked(uuid) {
            vpn_dropped(uuid, name);
        }
    }
    if now.portal && !before.portal {
        let network = Network::read();
        let place = network.ssid().to_string();
        drop(network);
        let title = if place.is_empty() {
            String::from("Sign In to the Network")
        } else {
            format!("Sign In to {place}")
        };
        notify(
            Feedback::Network,
            Notify::new(&title)
                .body("This network needs you to sign in before it lets you online.")
                .category("network")
                .action("open", "Open Sign-In Page", || {
                    emaki_kit::launch::open(PORTAL_PROBE);
                })
                .on_click(|| emaki_kit::launch::open(PORTAL_PROBE)),
            false,
        );
    }
    if let (Some(was), Some(is)) = (before.wired, now.wired)
        && was != is
    {
        echo(
            Feedback::Network,
            Echo {
                glyph: ICON_LAN,
                name: String::from("Ethernet"),
                detail: String::from(if is { "Connected" } else { "Disconnected" }),
            },
        );
    }
}

/// A failed try, named for what it was: a VPN by its name, a network by
/// its own.
fn failed(key: &str) {
    let network = Network::read();
    let Some(failure) = network.failure(key).cloned() else {
        return;
    };
    let vpn = network
        .vpns()
        .find(|profile| profile.uuid == key)
        .map(|profile| profile.name.clone());
    let wired = network.wired().is_some_and(|wired| wired.device == key);
    drop(network);

    let reason = reason(&failure);
    let (title, again): (String, Option<Box<dyn Fn() + Send + Sync>>) = match (vpn, wired) {
        (Some(name), _) => {
            let uuid = key.to_string();
            (
                format!("Couldn't Connect to {name}"),
                Some(Box::new(move || Connections::set_vpn(&uuid, true))),
            )
        }
        (None, true) => (String::from("Couldn't Connect Ethernet"), None),
        (None, false) => {
            let ssid = key.to_string();
            // a wrong password needs a new one, which only Quick Settings asks for
            let retry = failure.kind != amane::ErrorKind::AuthenticationFailed;
            (
                format!("Couldn't Join {key}"),
                retry.then(|| {
                    Box::new(move || Network::connect(&ssid, None)) as Box<dyn Fn() + Send + Sync>
                }),
            )
        }
    };
    let mut toast = Notify::new(&title)
        .body(&reason)
        .icon("network-error")
        .category("network.error");
    if let Some(again) = again {
        toast = toast.action("again", "Try Again", again);
    }
    notify(Feedback::Network, toast, false);
}

fn reason(failure: &ServiceError) -> String {
    let label = failure_label(failure);
    if failure.message.is_empty() || failure.kind != amane::ErrorKind::Failed {
        format!("{label}.")
    } else {
        failure.message.clone()
    }
}

fn vpn_dropped(uuid: &str, name: &str) {
    let uuid = uuid.to_string();
    notify(
        Feedback::Network,
        Notify::new(&format!("{name} Disconnected"))
            .body("The VPN went down. Traffic now goes over your network directly.")
            .icon("network-vpn-disconnected")
            .category("network.disconnected")
            .action("reconnect", "Reconnect", move || {
                asked(&uuid);
                Connections::set_vpn(&uuid, true);
            }),
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vpn(uuid: &str) -> (String, String) {
        (uuid.to_string(), format!("VPN {uuid}"))
    }

    #[test]
    fn a_failure_is_new_once_its_time_changes() {
        let at = Instant::now();
        let before = Seen {
            failures: BTreeMap::from([(String::from("Home"), at)]),
            ..Seen::default()
        };
        let same = before.clone();
        assert!(new_failures(&before, &same).is_empty());
        let again = Seen {
            failures: BTreeMap::from([(String::from("Home"), at + Duration::from_secs(1))]),
            ..Seen::default()
        };
        assert_eq!(new_failures(&before, &again), vec!["Home"]);
    }

    #[test]
    fn a_vpn_going_down_with_its_network_is_no_drop() {
        let before = Seen {
            vpns: BTreeMap::from([vpn("a")]),
            connected: true,
            ..Seen::default()
        };
        let offline = Seen::default();
        assert!(dropped(&before, &offline).is_empty());
        let online = Seen {
            connected: true,
            ..Seen::default()
        };
        assert_eq!(dropped(&before, &online), vec![("a", "VPN a")]);
    }
}
