//! The network toggles — GNOME's Wired, Wi‑Fi and VPN, each a tile of
//! its own.
//!
//! Wi‑Fi's toggle switches the radio and opens its page: the networks found
//! (rescanned every 15 s while shown, as GNOME does) and a password field
//! for a secured one not yet saved; while the machine is a hotspot the
//! tile says so and a click stops it. Wired shows only with a cable
//! in, VPN only when NetworkManager has a VPN profile.

use std::cell::{Cell as Slot, RefCell};
use std::time::{Duration, Instant};

use amane::{
    AccessPoint, Center, Column, ErrorKind, Link, Network, NetworkOperation, Parent, Row,
    ScrollArea, Service, ServiceError, TextInput, Widget,
};

use super::super::{Page, open};
use super::{connected_count, emaki_settings_link};
use crate::services::network::connections::Connections;
use crate::services::network::connections::Vpn;
use crate::services::network::rfkill::Rfkill;
use crate::surfaces::quick_settings::QuickSettings;
use crate::surfaces::quick_settings::detail::DetailPage;
use crate::surfaces::quick_settings::detail::page_width;
use crate::surfaces::quick_settings::tile::QuickToggle;
use crate::surfaces::status_bar::Indicator;
use crate::tokens::sys;
use emaki_kit::buttons::{PushButton, Style, Switch};
use emaki_kit::icons::{
    ICON_HOTSPOT, ICON_LOCK, ICON_VPN, ICON_WIFI_NONE, ICON_WIFI_OFF, ICON_WIRED, wifi_glyph,
};
use emaki_kit::layout::{boxed, caption, fixed_width, icon, nothing, text_height};
use emaki_kit::list::{self, List, ListItem};
use emaki_kit::material::{self, Level};
use emaki_kit::motion::redraw_after;

/// How often the Wi‑Fi page looks for networks again.
const RESCAN: Duration = Duration::from_secs(15);

const NETWORK_SETTINGS: &str = "nm-connection-editor";

/// TextInput identity for the password field (keys its text store).
const PW_ID: &str = "emaki.qs.wifi.password";

/// The network a password is being typed for.
#[derive(Default)]
struct Password {
    // SSID currently in password-entry mode, if any
    for_ssid: Option<String>,
    // live contents of the password field (TextInput has no getter)
    typed: String,
}

thread_local! {
    static LAST_SCAN: Slot<Option<Instant>> = const { Slot::new(None) };

    static PASSWORD: RefCell<Password> = RefCell::new(Password::default());
}

pub(crate) fn wired_toggle() -> Option<QuickToggle> {
    let wired = Connections::read().wired().cloned()?;

    let device = wired.device.clone();
    let subtitle = if wired.connected {
        wired.name
    } else {
        String::from("Not Connected")
    };
    Some(
        QuickToggle::new("emaki.qs.wired", ICON_WIRED, "Wired")
            .subtitle(subtitle)
            .checked(wired.connected)
            .on_toggle(move |on| Connections::set_wired(&device, on)),
    )
}

/// Whether there's Wi‑Fi to show: a radio, or a hotspot running on one.
pub(crate) fn wifi_available() -> bool {
    Connections::read().hotspot().is_some() || Rfkill::read().has("wlan")
}

pub(crate) fn vpn_available() -> bool {
    !Connections::read().vpns().is_empty()
}

pub(crate) fn wifi_toggle() -> Option<QuickToggle> {
    if let Some(hotspot) = Connections::read().hotspot().cloned() {
        // the disc stops the hotspot; the tile opens the networks
        return Some(
            QuickToggle::new(
                "emaki.qs.wifi",
                ICON_HOTSPOT,
                format!("{} Hotspot", hotspot.ssid),
            )
            .subtitle("Sharing this connection")
            .checked(true)
            .on_press(move || Connections::stop_hotspot(&hotspot.uuid))
            .on_open(|| open(Page::Wifi)),
        );
    }

    // a machine with no Wi‑Fi radio has nothing to switch
    if !Rfkill::read().has("wlan") {
        return None;
    }

    let net = Network::read();
    let enabled = net.wifi_enabled();
    // a hardware switch holds the radio off, so the disc can't turn it on;
    // the page still opens
    let held = Rfkill::read().hard_blocked("wlan");
    let subtitle = if held {
        String::from("Hardware switch off")
    } else if !enabled {
        String::from("Off")
    } else if net.connecting() {
        String::from("Connecting…")
    } else if net.link() == Link::Wifi {
        net.ssid().to_string()
    } else {
        String::from("Not Connected")
    };
    let glyph = match (enabled, net.link()) {
        (false, _) => ICON_WIFI_OFF,
        (true, Link::Wifi) => wifi_glyph(net.strength()),
        (true, _) => ICON_WIFI_NONE,
    };
    drop(net);

    Some(
        QuickToggle::new("emaki.qs.wifi", glyph, "Wi‑Fi")
            .subtitle(subtitle)
            .checked(enabled)
            .on_toggle(move |on| {
                if !held {
                    Network::set_wifi(on);
                }
            })
            .on_open(|| open(Page::Wifi)),
    )
}

pub(crate) fn wifi_page() -> DetailPage {
    let net = Network::read();
    let enabled = net.wifi_enabled();
    let subtitle = (net.link() == Link::Wifi).then(|| net.ssid().to_string());
    drop(net);

    // GNOME looks again every 15 s while its menu is open
    if enabled && LAST_SCAN.get().is_none_or(|at| at.elapsed() >= RESCAN) {
        LAST_SCAN.set(Some(Instant::now()));
        Network::scan();
        redraw_after::<QuickSettings>(RESCAN);
    }

    let page = DetailPage::new("emaki.qs.wifi.page", "Wi‑Fi")
        .subtitle(
            subtitle.unwrap_or_else(|| String::from(if enabled { "Not Connected" } else { "Off" })),
        )
        .switch(enabled, Network::set_wifi)
        .status_bar(Indicator::Network)
        .body(networks);
    emaki_settings_link(page, "Network Settings…", "network", NETWORK_SETTINGS)
}

/// The Wi‑Fi page was left: the next one scans straight away, and a
/// half-typed password is forgotten.
pub(crate) fn left_wifi() {
    LAST_SCAN.set(None);
    PASSWORD.set(Password::default());
    TextInput::set_text(PW_ID, "");
}

fn begin_password(ssid: String) {
    PASSWORD.set(Password {
        for_ssid: Some(ssid),
        typed: String::new(),
    });
    TextInput::set_text(PW_ID, "");
    QuickSettings::changed();
}

/// Joins `ssid` with the password typed for it. The page stays to show the
/// network connecting.
fn join_with_password(ssid: &str, password: &str) {
    Network::connect(ssid, Some(password));
    left_wifi();
}

/// How tall the password field is with the gap over it, which the list
/// above makes room for.
fn password_height() -> f32 {
    text_height(sys::r#type::caption()) + sys::space::gap::s() * 2.0 + sys::size::control::height()
}

/// The networks found, scrolling in `height`, and the password field when
/// one is picked that needs it.
fn networks(height: f32) -> Vec<Box<dyn Widget>> {
    let asking = PASSWORD.with_borrow(|p| p.for_ssid.is_some());
    let list_h = if asking {
        (height - password_height()).max(1.0)
    } else {
        height.max(1.0)
    };
    vec![boxed(ap_list(list_h)), password_section()]
}

fn ap_list(height: f32) -> ScrollArea {
    let net = Network::read();
    let enabled = net.wifi_enabled();
    let items = net
        .access_points()
        .iter()
        .map(|ap| {
            let state = Joining {
                connecting: net.operation(ap.ssid()) == Some(NetworkOperation::Connecting),
                failed: super::failure::recent(net.failure(ap.ssid())).map(failure_label),
                wrong_password: refused(net.failure(ap.ssid())),
            };
            ap_item(ap, &state)
        })
        .collect();
    drop(net);

    let note = if enabled {
        "Searching…"
    } else {
        "Wi‑Fi is off"
    };
    ScrollArea::new("emaki.qs.wifi.aps", List::new(items).empty(note).build()).height(height)
}

/// Where joining a network stands, as its row shows it.
struct Joining {
    connecting: bool,
    /// The last try failed lately: a few words why.
    failed: Option<&'static str>,
    /// The last try failed on its password, however long ago.
    wrong_password: bool,
}

/// Whether joining `ap` asks for a password first: a secured network with
/// none saved, or one whose saved password was refused, as NetworkManager
/// keeps a new network's profile even when its password is wrong.
pub(crate) fn needs_password(ap: &AccessPoint, wrong_password: bool) -> bool {
    ap.secured() && (!ap.saved() || wrong_password)
}

/// Whether `failure` is a refused password.
pub(crate) fn refused(failure: Option<&ServiceError>) -> bool {
    failure.is_some_and(|failure| failure.kind == ErrorKind::AuthenticationFailed)
}

pub(crate) use crate::services::network::failure_label;

fn ap_item(ap: &AccessPoint, joining: &Joining) -> ListItem {
    let ssid = ap.ssid().to_string();
    let secured = ap.secured();
    let active = ap.active();

    let mut item = ListItem::new(format!("emaki.qs.wifi.ap.{ssid}"), ssid.clone())
        .leading(wifi_glyph(ap.strength()))
        .selected(active);
    if joining.connecting && !active {
        item = item.trailing(caption("Connecting…"));
    } else if let Some(failed) = joining.failed {
        item = item.trailing(list::trailing_text(
            failed,
            sys::color::status::critical::text(),
        ));
    }
    if secured {
        item = item.trailing(icon(
            ICON_LOCK,
            sys::size::icon::s(),
            material::label(Level::Secondary),
        ));
    }
    if active {
        item = item.trailing(list::check());
    }

    let asks = needs_password(ap, joining.wrong_password);
    item.on_press(move || {
        if asks {
            begin_password(ssid.clone());
        } else {
            Network::connect(&ssid, None);
        }
    })
}

fn password_section() -> Box<dyn Widget> {
    let Some(ssid) = PASSWORD.with_borrow(|p| p.for_ssid.clone()) else {
        return nothing();
    };

    let connect_ssid = ssid.clone();
    let connect = PushButton::new("emaki.qs.wifi.connect", "Connect")
        .style(Style::Filled)
        .on_press(move || {
            let typed = PASSWORD.with_borrow(|p| p.typed.clone());
            join_with_password(&connect_ssid, &typed);
        })
        .build();
    let input_w = (page_width() - fixed_width(&connect) - sys::space::gap::s()).max(1.0);

    let submit_ssid = ssid.clone();
    let input = TextInput::new(PW_ID)
        .password()
        .focused()
        .placeholder("Password")
        .size(sys::r#type::label().size)
        .color(material::label(Level::Primary))
        .width(input_w)
        .on_change(|text| PASSWORD.with_borrow_mut(|p| p.typed = text))
        .on_submit(move |pw| join_with_password(&submit_ssid, &pw));

    let field = Row::new(vec![boxed(input), boxed(connect)])
        .gap(sys::space::gap::s())
        .width(Parent)
        .align(Center);

    boxed(
        Column::new(vec![
            boxed(caption(format!("Password for {ssid}"))),
            boxed(field),
        ])
        .gap(sys::space::gap::s())
        .width(Parent),
    )
}

pub(crate) fn vpn_toggle() -> Option<QuickToggle> {
    let connections = Connections::read();
    if connections.vpns().is_empty() {
        return None;
    }
    let active: Vec<_> = connections
        .vpns()
        .iter()
        .filter(|vpn| vpn.active)
        .cloned()
        .collect();
    let preferred = connections.preferred_vpn().cloned();
    drop(connections);

    Some(
        QuickToggle::new("emaki.qs.vpn", ICON_VPN, "VPN")
            .subtitle(vpn_status(&active))
            .checked(!active.is_empty())
            .on_toggle(move |on| set_vpns(on, &preferred, &active))
            .on_open(|| open(Page::Vpn)),
    )
}

/// "Off", the tunnel that's up, or how many are.
fn vpn_status(active: &[Vpn]) -> String {
    match active {
        [] => String::from("Off"),
        [one] => one.name.clone(),
        many => connected_count(many.len()),
    }
}

/// On: the preferred tunnel comes up. Off: every active one goes down.
fn set_vpns(on: bool, preferred: &Option<Vpn>, active: &[Vpn]) {
    if on {
        if let Some(vpn) = preferred {
            Connections::set_vpn(&vpn.uuid, true);
        }
    } else {
        for vpn in active {
            Connections::set_vpn(&vpn.uuid, false);
        }
    }
}

pub(crate) fn vpn_page() -> DetailPage {
    let connections = Connections::read();
    let vpns = connections.vpns().to_vec();
    let preferred = connections.preferred_vpn().cloned();
    drop(connections);
    let active: Vec<Vpn> = vpns.iter().filter(|vpn| vpn.active).cloned().collect();
    let any = !active.is_empty();

    let page = DetailPage::new("emaki.qs.vpn.page", "VPN")
        .subtitle(vpn_status(&active))
        .switch(any, move |on| set_vpns(on, &preferred, &active))
        .body(move |height| {
            let items: Vec<ListItem> = vpns
                .iter()
                .map(|vpn| {
                    let uuid = vpn.uuid.clone();
                    ListItem::new(format!("emaki.qs.vpn.{}", vpn.uuid), vpn.name.clone())
                        .leading(ICON_VPN)
                        .selected(vpn.active)
                        .trailing(
                            Switch::new(format!("emaki.qs.vpn.{}.switch", vpn.uuid), vpn.active)
                                .on_toggle(move |on| Connections::set_vpn(&uuid, on))
                                .build(),
                        )
                })
                .collect();
            vec![boxed(
                ScrollArea::new(
                    "emaki.qs.vpn.list",
                    List::new(items)
                        .header("VPN Connections")
                        .empty("No VPN connections")
                        .build(),
                )
                .height(height),
            )]
        });
    emaki_settings_link(page, "VPN Settings…", "network", NETWORK_SETTINGS)
}
