//! The shell's own services: system state Quick Settings shows that the
//! amane library has no service for. Each is an amane [`Service`] like the
//! library's — live state a view reads, redrawing only when it changes, and
//! a sane "not present" when its daemon or device is missing — so a tile
//! reads it the same way it reads `Network` or `Bluetooth`.
//!
//! Power (`power/`):
//!
//! - [`power::session`] — lock, suspend, restart, power off and log out (logind).
//! - [`power::keep_awake`] — the screen held on and the lid ignored (inhibitors).
//! - [`power::keyboard_backlight`] — the keyboard's light (sysfs and logind).
//! - [`power::power_refresh`] — each display's refresh rate on battery.
//! - [`display::wallpaper_dim`] — the wallpaper dimmed under the Black appearance.
//!
//! Network and radios (`network/`):
//!
//! - [`network::connections`] — wired, VPN and hotspot (amane's `Network`).
//! - [`network::rfkill`] — airplane mode: every radio blocked.
//! - [`network::tailscale`] — the tailnet, its exit nodes and settings (the CLI).
//! - [`feedback`] — capsules and notifications as devices, power and networks change.
//!
//! Display (`display/`):
//!
//! - [`display::night_light`] — a warm screen, by running `wlsunset`.
//! - [`display::dark_style`] — the system's light/dark preference (dconf).
//!
//! The desktop (`desktop/`):
//!
//! - [`desktop::privacy`] — what's using the microphone, the camera or the screen.
//! - [`desktop::background_apps`] — apps running without a window (the portal).
//! - [`desktop::updates`] — what the last NixOS update check found.
//! - [`desktop::system_stats`] — processor, memory, network, temperature and
//!   disks, and their recent history, for the desktop's widgets.
//!
//! The launcher and the bar (`launcher/`):
//!
//! - [`clock`] — the wall-clock time and date the bar shows, and times written on the local clock.
//! - [`launcher::timer`] — the launcher's countdown, which the bar shows while it runs.
//! - [`launcher::clipboard_history`] — what's been copied, for the launcher's
//!   Clipboard (wl-clipboard), once it's turned on.
//! - [`launcher::file_search`] — files by name and content for the launcher's
//!   Files (LocalSearch's index, or `fd`).
//!
//! Notifications:
//!
//! - [`notification_history`] — the notification center's history, kept across restarts.
//!
//! Passwords (`vault/`):
//!
//! - [`vault`] — Bitwarden through rbw: locked or not, its entries without their
//!   secrets, the shell's pinentry, and which app owns a field (`docs/user/features/passwords.md`).
//!
//! Shared helpers: [`state`] (the state files),
//! [`run`], [`run_now`], [`installed`] and [`Latest`] (`process.rs`), and [`Wake`] (`wake.rs`).
//!
//! See `docs/user/features/quick-settings.md` and `docs/user/features/widgets.md`.
//!
//! [`Service`]: amane::Service

pub mod assistant;
pub mod child_stream;
pub mod clock;
pub mod desktop;
pub mod display;
pub mod do_not_disturb;
pub mod feedback;
pub mod launcher;
pub mod media_art;
pub mod network;
pub mod notification_history;
pub mod phone;
pub mod power;
pub mod status_apps;
pub mod vault;

pub mod state;
mod wake;

pub use emaki_kit::process::{Latest, installed, run, run_now};
pub use wake::Wake;

/// Starts `argv` (a program and its arguments, as they are) without waiting, in a scope of
/// its own; one that can't start is logged.
pub fn launch<S: AsRef<str>>(argv: &[S]) {
    if let Err(error) = amane::Launch::argv(argv).spawn() {
        log::warn!("{error}");
    }
}
