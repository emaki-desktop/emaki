//! Connections — the wired link, VPNs and a Wi-Fi hotspot, as the bar and Quick Settings show
//! them. amane's `Network` follows NetworkManager's signals and lists the profiles; this keeps
//! the shell's view of them (and the VPN used last, for the toggle) and changes them through
//! amane, which leaves secrets and VPN plugins to NetworkManager. A VPN is any `vpn` or
//! `wireguard` profile, like GNOME's; a tunnel NetworkManager doesn't manage (tailscale) isn't
//! one. Without NetworkManager there's nothing, and the tiles stay hidden.

use amane::{Network, NetworkProfile, Service};
use emaki_kit::service::ServiceExt as _;
use emaki_kit::sync::Locked as _;

use crate::services::state::{self, State};

pub use amane::Wired;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Vpn {
    pub uuid: String,
    pub name: String,
    pub active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hotspot {
    pub uuid: String,
    pub ssid: String,
}

#[derive(Default, PartialEq)]
pub struct Connections {
    vpns: Vec<Vpn>,
    wired: Option<Wired>,
    hotspot: Option<Hotspot>,
}

impl Service for Connections {
    fn new() -> Self {
        from_network(&Network::read())
    }

    fn listen() {
        Network::subscribe(|network| {
            Connections::replace(from_network(network));
        })
        .forever();
    }
}

fn from_network(network: &Network) -> Connections {
    assemble(network.profiles(), network.wired().cloned())
}

fn assemble(profiles: &[NetworkProfile], wired: Option<Wired>) -> Connections {
    Connections {
        vpns: profiles
            .iter()
            .filter(|profile| profile.is_vpn())
            .map(|profile| Vpn {
                uuid: profile.uuid.clone(),
                name: profile.name.clone(),
                active: profile.active,
            })
            .collect(),
        wired,
        hotspot: profiles
            .iter()
            .rfind(|profile| profile.is_hotspot())
            .map(|profile| Hotspot {
                uuid: profile.uuid.clone(),
                ssid: profile.name.clone(),
            }),
    }
}

impl Connections {
    pub fn vpns(&self) -> &[Vpn] {
        &self.vpns
    }

    pub fn wired(&self) -> Option<&Wired> {
        self.wired.as_ref()
    }

    pub fn hotspot(&self) -> Option<&Hotspot> {
        self.hotspot.as_ref()
    }

    pub fn set_vpn(uuid: &str, on: bool) {
        // asked for here, so its going down isn't told of as a drop
        crate::services::feedback::network::asked(uuid);
        if on {
            remember_vpn(uuid);
            Network::activate(uuid);
        } else {
            Network::deactivate(uuid);
        }
    }

    /// The VPN the toggle turns on: the last one used, else the first.
    pub fn preferred_vpn(&self) -> Option<&Vpn> {
        let last = last_vpn();
        self.vpns
            .iter()
            .find(|vpn| Some(&vpn.uuid) == last.as_ref())
            .or(self.vpns.first())
    }

    pub fn stop_hotspot(uuid: &str) {
        Network::deactivate(uuid);
    }

    pub fn set_wired(device: &str, on: bool) {
        Network::set_device(device, on);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn profile(
        name: &str,
        uuid: &str,
        kind: &str,
        mode: Option<&str>,
        active: bool,
    ) -> NetworkProfile {
        NetworkProfile {
            uuid: uuid.into(),
            name: name.into(),
            kind: kind.into(),
            wifi_mode: mode.map(String::from),
            active,
            ..NetworkProfile::default()
        }
    }

    // what tests/fixtures/nmcli/connection.txt listed, as amane hands it over
    fn recorded(home_mode: &str) -> Vec<NetworkProfile> {
        vec![
            profile(
                "Home Wi-Fi",
                "1111",
                "802-11-wireless",
                Some(home_mode),
                true,
            ),
            profile("tailscale0", "2222", "tun", None, true),
            profile("lo", "3333", "loopback", None, true),
            profile(
                "Office: 5 GHz",
                "4444",
                "802-11-wireless",
                Some("ap"),
                false,
            ),
            profile("Work VPN", "5555", "vpn", None, false),
            profile("wg-home", "6666", "wireguard", None, true),
            profile("Wired connection 1", "7777", "802-3-ethernet", None, true),
        ]
    }

    #[test]
    fn the_recorded_profiles_give_the_vpns_and_no_hotspot() {
        let wired = Wired {
            device: "enp2s0".into(),
            connected: true,
            name: "Wired connection 1".into(),
        };
        let c = assemble(&recorded("infrastructure"), Some(wired.clone()));
        let vpns: Vec<(&str, bool)> = c.vpns.iter().map(|v| (v.name.as_str(), v.active)).collect();
        assert_eq!(vpns, [("Work VPN", false), ("wg-home", true)]);
        assert_eq!(c.wired, Some(wired));
        // an access-point profile that isn't up isn't a hotspot
        assert_eq!(c.hotspot, None);
    }

    #[test]
    fn an_active_wifi_profile_in_ap_mode_is_the_hotspot() {
        let c = assemble(&recorded("ap"), None);
        assert_eq!(
            c.hotspot,
            Some(Hotspot {
                uuid: "1111".into(),
                ssid: "Home Wi-Fi".into(),
            })
        );
        assert_eq!(c.wired, None);
    }
}

/// The VPN last turned on, kept in the state file `vpn.json` (the older
/// `vpn`, its id alone, is read when that isn't there).
#[derive(amane::serde::Serialize, amane::serde::Deserialize)]
#[serde(crate = "amane::serde")]
struct LastVpn {
    uuid: String,
}

impl State for LastVpn {
    const FILE: &'static str = "vpn.json";
    const VERSION: u32 = 1;

    fn legacy() -> Option<Self> {
        let uuid = state::legacy_text("vpn")?.trim().to_string();
        (!uuid.is_empty()).then_some(Self { uuid })
    }
}

// read once, since Quick Settings asks on every frame it shows the VPN
static LAST_VPN: std::sync::Mutex<Option<Option<String>>> = std::sync::Mutex::new(None);

fn last_vpn() -> Option<String> {
    let mut last = LAST_VPN.locked();
    last.get_or_insert_with(|| state::load::<LastVpn>().map(|vpn| vpn.uuid))
        .clone()
}

fn remember_vpn(uuid: &str) {
    *LAST_VPN.locked() = Some(Some(uuid.to_string()));
    state::save(&LastVpn {
        uuid: uuid.to_string(),
    });
}

#[cfg(test)]
mod last_vpn_tests {
    use super::*;

    #[test]
    fn the_old_vpn_file_migrates() {
        use crate::services::state::tests::in_folder;
        in_folder("vpn", &[("vpn", "0d6f-uuid\n")], |_| {
            assert_eq!(
                state::load::<LastVpn>().map(|vpn| vpn.uuid).as_deref(),
                Some("0d6f-uuid")
            );
        });
        in_folder("vpn-none", &[], |_| {
            assert!(state::load::<LastVpn>().is_none())
        });
    }
}
