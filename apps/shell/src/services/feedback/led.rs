//! A laptop light that tells of notifications while you're away from the
//! screen: a ThinkPad's lid logo dot or ThinkVantage key pulses while a
//! notification hasn't been seen, and, when asked, stays lit while Do Not
//! Disturb is on. Off in Settings unless chosen; nothing on a laptop
//! without such a light.
//!
//! The light is set through logind, as the microphone's mute light is
//! ([`crate::services::power::mic_led`]), so no root is needed. Pulsing
//! wakes once a second only while there's something unseen; otherwise the
//! thread sleeps until a notification, Do Not Disturb, the lock or the
//! setting changes.

use std::path::Path;
use std::sync::Once;
use std::time::{Duration, SystemTime};

use amane::{Config, Logind, Notifications, Service};
use emaki_kit::settings::{Light, NotificationRules, Pulse, notification_rules};

use crate::services::Wake;
use crate::services::do_not_disturb::DoNotDisturb;
use crate::services::notification_history::unseen;
use crate::services::power::logind::set_brightness;

/// How long the light is on, then off, in each pulse.
const ON: Duration = Duration::from_secs(1);
const OFF: Duration = Duration::from_secs(2);

const LIT: u32 = 255;

static WAKE: Wake = Wake::new();

/// What the light should do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Show {
    Dark,
    Steady,
    Pulse,
}

/// The laptop lights this machine has, of those Settings offers.
pub fn present() -> Vec<Light> {
    Light::ALL
        .into_iter()
        .filter(|light| Path::new("/sys/class/leds").join(light.led()).exists())
        .collect()
}

/// Something the light follows changed: look again.
pub fn nudge() {
    WAKE.nudge();
}

/// Starts following, once; nothing without a light.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        if present().is_empty() {
            return;
        }
        Notifications::subscribe(|_| WAKE.nudge()).forever();
        DoNotDisturb::subscribe(|_| WAKE.nudge()).forever();
        Logind::subscribe(|_| WAKE.nudge()).forever();
        Config::<NotificationRules>::subscribe(|_| WAKE.nudge()).forever();
        std::thread::Builder::new()
            .name(String::from("emaki-led"))
            .spawn(run)
            .ok();
    });
}

fn run() {
    // the light lit, and whether it's lit now
    let mut lit: Option<(Light, bool)> = None;
    let mut phase_on = false;
    loop {
        let rules = notification_rules().clone();
        let show = show(
            rules.pulse,
            rules.light_for_dnd,
            unseen(false),
            unseen(true),
            DoNotDisturb::read().on(),
            Logind::read().locked_hint(),
        );
        // a light let go, or another picked, goes dark
        if let Some((light, true)) = lit
            && (light != rules.light || show == Show::Dark)
        {
            set_brightness("leds", light.led(), 0);
            lit = Some((light, false));
        }
        let next = match show {
            Show::Dark => None,
            Show::Steady => {
                phase_on = true;
                None
            }
            Show::Pulse => {
                phase_on = !phase_on;
                Some(SystemTime::now() + if phase_on { ON } else { OFF })
            }
        };
        let on = show != Show::Dark && phase_on;
        if lit != Some((rules.light, on)) {
            set_brightness("leds", rules.light.led(), if on { LIT } else { 0 });
            lit = Some((rules.light, on));
        }
        WAKE.wait(next);
    }
}

/// What the light does, from what it's set to and what there is.
pub fn show(
    pulse: Pulse,
    for_dnd: bool,
    unseen: usize,
    unseen_critical: usize,
    dnd: bool,
    locked: bool,
) -> Show {
    let waiting = match pulse {
        Pulse::Off => false,
        Pulse::All => unseen > 0,
        Pulse::Critical => unseen_critical > 0,
    };
    // locked, the light is how anything unseen is known from across the room
    if waiting && (!dnd || unseen_critical > 0) {
        return Show::Pulse;
    }
    if for_dnd && dnd && !locked {
        return Show::Steady;
    }
    Show::Dark
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pulses_for_what_it_was_asked_to() {
        assert_eq!(show(Pulse::All, false, 2, 0, false, false), Show::Pulse);
        assert_eq!(show(Pulse::Critical, false, 2, 0, false, false), Show::Dark);
        assert_eq!(
            show(Pulse::Critical, false, 2, 1, false, false),
            Show::Pulse
        );
        assert_eq!(show(Pulse::Off, false, 2, 1, false, false), Show::Dark);
        assert_eq!(show(Pulse::All, false, 0, 0, false, false), Show::Dark);
    }

    #[test]
    fn do_not_disturb_holds_it_steady_unless_something_is_critical() {
        assert_eq!(show(Pulse::All, true, 3, 0, true, false), Show::Steady);
        assert_eq!(show(Pulse::All, true, 3, 1, true, false), Show::Pulse);
        assert_eq!(show(Pulse::All, false, 3, 0, true, false), Show::Dark);
        assert_eq!(show(Pulse::Off, true, 0, 0, true, true), Show::Dark);
    }
}
