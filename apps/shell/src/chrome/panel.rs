//! The one-open-at-a-time state Quick Settings, the notification center
//! and the launcher share, and the overlay-surface helpers they and the
//! toasts are built on.
//!
//! Opening one closes the others; Escape or toggling again closes it.

use std::cell::{Cell, RefCell};

use amane::{Horizontal, Key, Keyboard, Layer, LayerWindow, Margin, Vertical, Zone, mark_all};

use super::bar;
use crate::tokens::sys;
use emaki_kit::material;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Panel {
    /// The notification center, a sidebar at the right edge.
    Notifications,
    /// Quick Settings, a card at the bar's right-hand corner.
    QuickSettings,
    /// The app launcher, a panel in the middle.
    Launcher,
}

/// What a panel's surface does as it opens and closes, given once at startup
/// ([`register`]) so this module knows no surface.
#[derive(Clone, Copy)]
pub struct Hooks {
    pub opened: fn(),
    pub closed: fn(),
}

/// What happens whenever the open panel changes away from one, as closing the
/// menu that belonged to it.
pub type OnSwitch = fn();

thread_local! {
    static OPEN: Cell<Option<Panel>> = const { Cell::new(None) };

    // the monitor the open panel shows on: the one the pointer was over
    static ON: RefCell<Option<String>> = const { RefCell::new(None) };

    static HOOKS: RefCell<Vec<(Panel, Hooks)>> = const { RefCell::new(Vec::new()) };
    static ON_SWITCH: RefCell<Vec<OnSwitch>> = const { RefCell::new(Vec::new()) };
}

/// Runs `hooks` as `panel` opens and closes.
pub fn register(panel: Panel, hooks: Hooks) {
    HOOKS.with_borrow_mut(|all| all.push((panel, hooks)));
}

/// Runs `f` whenever the open panel changes away from one.
pub fn on_switch(f: OnSwitch) {
    ON_SWITCH.with_borrow_mut(|all| all.push(f));
}

fn hooks(panel: Option<Panel>) -> Vec<Hooks> {
    HOOKS.with_borrow(|all| {
        all.iter()
            .filter(|(p, _)| Some(*p) == panel)
            .map(|(_, hooks)| *hooks)
            .collect()
    })
}

/// The monitor the panels show on, by name: the one the pointer was over as
/// the last one opened, so a bar's panels open on its own display.
pub fn on_output() -> Option<String> {
    ON.with_borrow(Clone::clone)
}

/// The open panel moves to the monitor the pointer is over now, as when
/// another bar's icon changes what Quick Settings shows.
pub fn follow_pointer() {
    if let Some(here) = amane::pointer_monitor() {
        ON.set(Some(here));
    }
}

// ───── runtime ───────────────────────────────────────────────────────────────

/// Opens `panel`, closing whichever other one is open, or closes it if it is.
pub fn toggle(panel: Panel) {
    set(if is_open(panel) { None } else { Some(panel) });
}

pub fn close() {
    set(None);
}

pub fn is_open(panel: Panel) -> bool {
    OPEN.with(Cell::get) == Some(panel)
}

// which panel is open, for threads other than the UI's: 0 for none
static OPEN_ANYWHERE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

fn code(panel: Option<Panel>) -> u8 {
    match panel {
        None => 0,
        Some(Panel::Notifications) => 1,
        Some(Panel::QuickSettings) => 2,
        Some(Panel::Launcher) => 3,
    }
}

/// [`is_open`] from any thread, as a service's own thread asks before
/// telling of something the panel already shows.
pub fn is_open_anywhere(panel: Panel) -> bool {
    OPEN_ANYWHERE.load(std::sync::atomic::Ordering::Relaxed) == code(Some(panel))
}

fn set(next: Option<Panel>) {
    if next.is_some() {
        follow_pointer();
    }
    let previous = OPEN.with(|open| open.replace(next));
    if previous == next {
        return;
    }
    OPEN_ANYWHERE.store(code(next), std::sync::atomic::Ordering::Relaxed);

    for hooks in hooks(previous) {
        (hooks.closed)();
    }
    for hooks in hooks(next) {
        (hooks.opened)();
    }
    if previous.is_some() {
        for f in ON_SWITCH.with_borrow(Clone::clone) {
            f();
        }
    }

    mark_all();
}

// ───── view ──────────────────────────────────────────────────────────────────

/// An overlay-layer surface anchored to `vertical` and `horizontal` edges,
/// clear of what other surfaces reserve, its labels vibrant where vibrancy
/// is on (`material::vibrant`). Callers size it first; panels put it on
/// [`on_output`].
pub(crate) fn overlay(
    base: LayerWindow,
    namespace: &'static str,
    vertical: Vertical,
    horizontal: Horizontal,
) -> LayerWindow {
    base.anchor_vertical(vertical)
        .anchor_horizontal(horizontal)
        .layer(Layer::Overlay)
        .space(Zone::Respect)
        .namespace(namespace)
        .vibrant(material::vibrant())
}

/// `window` with nothing to show this frame: hidden, with no frost.
pub(crate) fn empty(window: LayerWindow) -> LayerWindow {
    window
        .visible(false)
        .blur_shapes(Vec::new())
        .child(emaki_kit::layout::blank())
}

/// Takes keys as `keyboard` says, and closes the open panel on Escape.
pub(crate) fn dismissable(base: LayerWindow, keyboard: Keyboard) -> LayerWindow {
    base.keyboard(keyboard).on_key(|key| {
        if matches!(key, Key::Escape) {
            close();
        }
    })
}

/// Where a card in the bar's right-hand corner sits: `gap.m` in from the
/// right edge and clear of the bar, above it or below it.
pub(crate) fn popover_margin() -> Margin {
    let clear = (bar::height() + sys::space::gap::m()) as i32;
    let top = bar::edge() == Vertical::Top;

    Margin {
        top: if top { clear } else { 0 },
        right: sys::space::gap::m() as i32,
        bottom: if top { 0 } else { clear },
        left: 0,
    }
}
