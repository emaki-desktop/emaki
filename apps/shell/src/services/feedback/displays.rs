//! Displays plugged in and out: a capsule with the display's own name and
//! the mode it shows ("Dell U2723QE · 3840×2160"). The laptop's built-in
//! panel is left out, since it goes and comes back as the lid closes on a
//! dock.

use std::collections::BTreeMap;
use std::sync::Once;

use amane::{Outputs, Service, niri};
use emaki_kit::icons::ICON_DISPLAY;
use emaki_kit::settings::Feedback;

use super::{Echo, echo};
use crate::services::Wake;

static WAKE: Wake = Wake::new();

/// Starts watching, once.
pub fn watch() {
    static WATCH: Once = Once::new();
    WATCH.call_once(|| {
        Outputs::subscribe(|_| WAKE.nudge()).forever();
        std::thread::spawn(|| {
            let mut before: Option<BTreeMap<String, String>> = None;
            loop {
                let now = external(Outputs::read().all());
                if let Some(before) = &before {
                    for (name, connected) in changes(before, &now) {
                        tell(&name, connected, now.get(&name).or(before.get(&name)));
                    }
                }
                before = Some(now);
                WAKE.wait(None);
            }
        });
    });
}

/// The displays plugged in that aren't built in, by connector, each with
/// what to call it.
fn external(outputs: &[niri::Output]) -> BTreeMap<String, String> {
    outputs
        .iter()
        .filter(|output| !built_in(&output.name))
        .map(|output| (output.name.clone(), title(output)))
        .collect()
}

/// A laptop's own panel, by its connector.
pub fn built_in(connector: &str) -> bool {
    ["eDP", "LVDS", "DSI"]
        .iter()
        .any(|prefix| connector.starts_with(prefix))
}

/// "Dell U2723QE", or the connector when the display names nothing.
fn title(output: &niri::Output) -> String {
    let named = format!("{} {}", output.make, output.model);
    let named = named.trim();
    if named.is_empty() {
        output.name.clone()
    } else {
        named.to_string()
    }
}

/// Which connectors came (true) and went (false).
pub fn changes(
    before: &BTreeMap<String, String>,
    now: &BTreeMap<String, String>,
) -> Vec<(String, bool)> {
    let went = before
        .keys()
        .filter(|name| !now.contains_key(*name))
        .map(|name| (name.clone(), false));
    let came = now
        .keys()
        .filter(|name| !before.contains_key(*name))
        .map(|name| (name.clone(), true));
    went.chain(came).collect()
}

fn tell(connector: &str, connected: bool, title: Option<&String>) {
    let mode = connected
        .then(|| {
            let outputs = Outputs::read();
            let output = outputs.get(connector)?;
            let mode = output.modes.get(output.current_mode?)?;
            Some(format!("{}×{}", mode.width, mode.height))
        })
        .flatten();
    echo(
        Feedback::Displays,
        Echo {
            glyph: ICON_DISPLAY,
            name: title.cloned().unwrap_or_else(|| connector.to_string()),
            detail: match (connected, mode) {
                (true, Some(mode)) => mode,
                (true, None) => String::from("Connected"),
                (false, _) => String::from("Disconnected"),
            },
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_built_in_panel_is_left_out() {
        assert!(built_in("eDP-1"));
        assert!(!built_in("DP-3"));
        assert!(!built_in("HDMI-A-1"));
    }

    #[test]
    fn tells_of_displays_that_came_and_went() {
        let before = BTreeMap::from([(String::from("DP-1"), String::from("Dell"))]);
        let now = BTreeMap::from([(String::from("HDMI-A-1"), String::from("LG"))]);
        assert_eq!(
            changes(&before, &now),
            vec![
                (String::from("DP-1"), false),
                (String::from("HDMI-A-1"), true),
            ]
        );
        assert!(changes(&now, &now).is_empty());
    }
}
