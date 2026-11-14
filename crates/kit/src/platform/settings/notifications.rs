//! `notifications.json`: what each app's notifications may do.

use std::collections::BTreeSet;

use super::*;

/// What an app's notifications may do.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(crate = "amane::serde", rename_all = "kebab-case")]
pub enum AppRule {
    /// A banner, then the list in the notification center.
    #[default]
    Allow,
    /// Straight into the notification center, with no banner.
    CenterOnly,
    /// Nowhere; a critical one still shows, as through Do Not Disturb.
    Off,
}

impl AppRule {
    /// Every rule, in the order a menu lists them.
    pub const ALL: [Self; 3] = [Self::Allow, Self::CenterOnly, Self::Off];

    /// What a menu calls it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Allow => "Banners and List",
            Self::CenterOnly => "List Only",
            Self::Off => "Off",
        }
    }
}

/// What the shell itself tells of as the system changes: a capsule for a
/// device coming or going, a notification for what needs a hand.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(crate = "amane::serde", rename_all = "kebab-case")]
pub enum Feedback {
    /// Bluetooth devices connecting, leaving, running low.
    Bluetooth,
    /// USB devices plugged in and out.
    UsbDevices,
    /// Removable drives: ready to open, ejected, pulled out too soon.
    Storage,
    /// The charger, the battery running low, a slow charger.
    Power,
    /// A network that couldn't be joined, a VPN dropped, a sign-in page.
    Network,
    /// Displays connected and disconnected.
    Displays,
    /// A screenshot taken.
    Screenshots,
    /// The computer running hot.
    Thermals,
}

impl Feedback {
    /// Every kind, in the order Settings lists them.
    pub const ALL: [Self; 8] = [
        Self::Bluetooth,
        Self::UsbDevices,
        Self::Storage,
        Self::Power,
        Self::Network,
        Self::Displays,
        Self::Screenshots,
        Self::Thermals,
    ];

    /// What Settings calls it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Bluetooth => "Bluetooth",
            Self::UsbDevices => "USB Devices",
            Self::Storage => "Drives",
            Self::Power => "Power",
            Self::Network => "Network",
            Self::Displays => "Displays",
            Self::Screenshots => "Screenshots",
            Self::Thermals => "Temperature",
        }
    }

    /// The desktop entry its notifications are sent under, so the center
    /// groups them and Settings knows them as the shell's own.
    pub fn entry(self) -> &'static str {
        match self {
            Self::Bluetooth => "emaki-bluetooth",
            Self::UsbDevices => "emaki-usb",
            Self::Storage => "emaki-drives",
            Self::Power => "emaki-power",
            Self::Network => "emaki-network",
            Self::Displays => "emaki-displays",
            Self::Screenshots => "emaki-screenshots",
            Self::Thermals => "emaki-thermals",
        }
    }

    /// The kind whose notifications are sent under `key`, if the shell's.
    pub fn of_entry(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.entry() == key)
    }
}

/// The laptop light that pulses for notifications.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(crate = "amane::serde", rename_all = "kebab-case")]
pub enum Light {
    /// The dot over the "i" of the logo on a ThinkPad's lid.
    #[default]
    LidLogo,
    /// The red dot by a ThinkPad's ThinkVantage key.
    ThinkVantage,
}

impl Light {
    /// Every light, in the order a menu lists them.
    pub const ALL: [Self; 2] = [Self::LidLogo, Self::ThinkVantage];

    /// What a menu calls it.
    pub fn label(self) -> &'static str {
        match self {
            Self::LidLogo => "Lid Logo",
            Self::ThinkVantage => "ThinkVantage Key",
        }
    }

    /// Its LED under `/sys/class/leds`.
    pub fn led(self) -> &'static str {
        match self {
            Self::LidLogo => "tpacpi::lid_logo_dot",
            Self::ThinkVantage => "tpacpi::thinkvantage",
        }
    }
}

/// What the light pulses for.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(crate = "amane::serde", rename_all = "kebab-case")]
pub enum Pulse {
    /// Nothing.
    #[default]
    Off,
    /// Any notification not seen yet.
    All,
    /// Only a critical one not seen yet, like a battery running out.
    Critical,
}

impl Pulse {
    /// Every choice, in the order a menu lists them.
    pub const ALL: [Self; 3] = [Self::All, Self::Critical, Self::Off];

    /// What a menu calls it.
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::All => "All Notifications",
            Self::Critical => "Critical Only",
        }
    }
}

/// The notification rules, by app key (see [`notification_app_key`]); an
/// app that isn't here is allowed.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(crate = "amane::serde", default)]
pub struct NotificationRules {
    /// Each app's rule, by its key.
    pub apps: BTreeMap<String, AppRule>,
    /// The kinds of system feedback turned off; any other shows.
    pub system_off: BTreeSet<Feedback>,
    /// A sound as a device connects or disconnects.
    pub connection_sounds: bool,
    /// Leave a drive unmounted as it's plugged in, rather than mounting it
    /// to open.
    pub keep_drives_unmounted: bool,
    /// Which laptop light shows notifications.
    pub light: Light,
    /// What the light pulses for.
    pub pulse: Pulse,
    /// The light stays on while Do Not Disturb is.
    pub light_for_dnd: bool,
}

impl NotificationRules {
    /// The rule for the app keyed `key`.
    pub fn rule(&self, key: &str) -> AppRule {
        self.apps.get(key).copied().unwrap_or_default()
    }

    /// Whether the shell tells of `kind` at all.
    pub fn shows(&self, kind: Feedback) -> bool {
        !self.system_off.contains(&kind)
    }
}

/// What a notification's app is known by: its desktop entry, else its app
/// name, lowercased. The center groups by it, and the rules are keyed by it.
pub fn notification_app_key(desktop_entry: &str, app_name: &str) -> String {
    let key = if desktop_entry.is_empty() {
        app_name
    } else {
        desktop_entry
    };
    key.to_lowercase()
}

settings_file!(
    NotificationRules,
    crate::app_ids::SHELL,
    "notifications",
    notification_rules,
    change_notification_rules
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_app_is_known_by_its_entry_else_its_name() {
        assert_eq!(
            notification_app_key("firefox", "Firefox Nightly"),
            "firefox"
        );
        assert_eq!(notification_app_key("", "Slack"), "slack");
        assert_eq!(
            notification_app_key("Org.Gnome.Nautilus", ""),
            "org.gnome.nautilus"
        );
    }

    #[test]
    fn rules_read_as_kebab_case_and_default_to_allow() {
        let rules: NotificationRules =
            serde_json::from_str(r#"{"apps":{"slack":"center-only","spam":"off"}}"#).unwrap();
        assert_eq!(rules.rule("slack"), AppRule::CenterOnly);
        assert_eq!(rules.rule("spam"), AppRule::Off);
        assert_eq!(rules.rule("firefox"), AppRule::Allow);
    }

    #[test]
    fn system_feedback_shows_unless_turned_off() {
        let rules: NotificationRules =
            serde_json::from_str(r#"{"system_off":["usb-devices"]}"#).unwrap();
        assert!(!rules.shows(Feedback::UsbDevices));
        assert!(rules.shows(Feedback::Bluetooth));
        assert!(!rules.connection_sounds);
        assert_eq!(Feedback::of_entry("emaki-usb"), Some(Feedback::UsbDevices));
        assert_eq!(Feedback::of_entry("firefox"), None);
    }
}
