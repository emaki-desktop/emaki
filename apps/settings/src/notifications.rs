//! Notifications: the sound one plays ([`crate::sound::alerts`]), what the
//! shell itself tells of as devices, power and networks change, and what
//! each app that has sent one may do. Each choice changes
//! `notifications.json`, which the shell follows at once.

use amane::Column;
use emaki_kit::app_icon;
use emaki_kit::dropdown::Dropdown;
use emaki_kit::form::FormRow;
use emaki_kit::settings::{
    AppRule, Feedback, Light, Pulse, change_notification_rules, notification_rules, notifiers_seen,
};

use crate::system::apps::icon_side;
use crate::{group, groups, switch_row};

/// What a system feedback menu offers.
const SHOW: [(&str, bool); 2] = [("Show", true), ("Off", false)];

pub fn page() -> Column {
    let rules = notification_rules().clone();

    let rows: Vec<FormRow> = notifiers_seen()
        .into_iter()
        // the shell's own are chosen under System Feedback
        .filter(|(key, _)| Feedback::of_entry(key).is_none())
        .map(|(key, name)| {
            let label = if name.is_empty() { key.clone() } else { name };
            let changed = key.clone();
            let menu = Dropdown::choice(
                format!("settings.notifications.app.{key}.rule"),
                AppRule::ALL.map(|rule| (rule.label(), rule)),
                rules.rule(&key),
                move |rule| {
                    change_notification_rules(|now| {
                        if rule == AppRule::Allow {
                            now.apps.remove(&changed);
                        } else {
                            now.apps.insert(changed.clone(), rule);
                        }
                    });
                },
            )
            .width(crate::MENU_W)
            .build();
            FormRow::new(format!("settings.notifications.app.{key}"), &label)
                .leading(app_icon::tile(&key, &label, icon_side()), icon_side())
                .trailing(menu)
        })
        .collect();

    let apps = if rows.is_empty() {
        group().header("Apps").row(crate::empty_row(
            "settings.notifications.none",
            "Apps show here once they've sent a notification, each with its own choice.",
        ))
    } else {
        group().header("Apps").rows(rows)
    };
    let mut shown = vec![crate::sound::alerts(), system(&rules)];
    shown.extend(light(&rules));
    shown.push(apps.footer(
        "List Only skips the banner. Off hides them altogether. Critical notifications, \
         like a low battery, show whatever is chosen.",
    ));
    groups(shown)
}

/// One menu for each kind of system feedback, and the connection sound.
fn system(rules: &emaki_kit::settings::NotificationRules) -> emaki_kit::form::Group {
    let rows: Vec<FormRow> = Feedback::ALL
        .into_iter()
        .map(|kind| {
            let id = format!("settings.notifications.system.{}", kind.entry());
            let menu = Dropdown::choice(format!("{id}.menu"), SHOW, rules.shows(kind), move |on| {
                change_notification_rules(|now| {
                    if on {
                        now.system_off.remove(&kind);
                    } else {
                        now.system_off.insert(kind);
                    }
                });
            })
            .width(crate::MENU_W)
            .build();
            FormRow::new(id, kind.label()).trailing(menu)
        })
        .collect();
    group()
        .header("System Feedback")
        .rows(rows)
        .row(switch_row(
            "settings.notifications.system.mount",
            "Open drives as they're plugged in",
            !rules.keep_drives_unmounted,
            |on| change_notification_rules(|now| now.keep_drives_unmounted = !on),
        ))
        .row(switch_row(
            "settings.notifications.system.sounds",
            "Play a sound as devices connect",
            rules.connection_sounds,
            |on| change_notification_rules(|now| now.connection_sounds = on),
        ))
        .footer(
            "A device connecting or a display plugged in shows briefly at the bottom of the \
         screen. Something to act on, like a drive to eject or a battery running low, \
         comes as a notification.",
        )
}

/// The laptop's own lights this machine has: a ThinkPad's lid logo dot and
/// ThinkVantage key.
fn lights() -> Vec<Light> {
    Light::ALL
        .into_iter()
        .filter(|light| {
            std::path::Path::new("/sys/class/leds")
                .join(light.led())
                .exists()
        })
        .collect()
}

/// Which light shows notifications, and what for; none on a laptop
/// without one.
fn light(rules: &emaki_kit::settings::NotificationRules) -> Option<emaki_kit::form::Group> {
    let present = lights();
    let first = *present.first()?;
    let chosen = if present.contains(&rules.light) {
        rules.light
    } else {
        first
    };
    let pulse = Dropdown::choice(
        "settings.notifications.light.pulse.menu",
        Pulse::ALL.map(|pulse| (pulse.label(), pulse)),
        rules.pulse,
        |pulse| change_notification_rules(|now| now.pulse = pulse),
    )
    .width(crate::MENU_W)
    .build();
    let mut group = group()
        .header("Indicator Light")
        .row(FormRow::new("settings.notifications.light.pulse", "Pulse for").trailing(pulse));
    if present.len() > 1 {
        let which = Dropdown::choice(
            "settings.notifications.light.which.menu",
            present.iter().map(|light| (light.label(), *light)),
            chosen,
            |light| change_notification_rules(|now| now.light = light),
        )
        .width(crate::MENU_W)
        .build();
        group =
            group.row(FormRow::new("settings.notifications.light.which", "Light").trailing(which));
    }
    Some(
        group
            .row(switch_row(
                "settings.notifications.light.dnd",
                "Stay lit during Do Not Disturb",
                rules.light_for_dnd,
                |on| change_notification_rules(|now| now.light_for_dnd = on),
            ))
            .footer(
                "The light pulses while a notification hasn't been seen, and stops once the \
             notification center is opened.",
            ),
    )
}
