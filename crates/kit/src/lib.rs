//! The shell's look, for the shell and for apps that run beside it: the
//! design tokens from Penpot (`design-tokens/tokens.json`), the theme they
//! resolve against, and the controls built from them.
//!
//! The source is grouped into folders by kind, and every module is
//! re-exported here, so its path is `emaki_kit::<module>` whatever folder
//! it lives in.
//!
//! Controls (`controls/`):
//!
//! - [`buttons`] — push, icon and segmented buttons, a switch, a checkbox
//!   and radio buttons.
//! - [`slider`] — a 0–100 level along a track.
//! - [`dropdown`] — one choice out of a list, behind a button that shows it.
//! - [`color_picker`] — a colour, from a square, a hue strip or hex.
//! - [`control_size`] — the control sizes, and the one in use.
//! - [`tag`] — a small fact about something, as a capsule label.
//! - [`menu`] — a pop-up menu of items, under a control or the pointer.
//! - [`leading_icon`] — the glyph on a coloured tile that leads a row.
//! - [`stepper`] — a number nudged up and down.
//! - [`thumbnail_picker`] — one choice out of pictures.
//! - [`nav_buttons`] — the back and forward pair of a window's toolbar.
//! - [`pop_up_button`] — a small round button that opens a menu.
//! - [`calendar`] — a month at a glance, today marked.
//!
//! The `controls` example shows them all.
//!
//! Containers (`containers/`):
//!
//! - [`card`] — a filled or outlined surface, optionally frosted.
//! - [`app_bar`] — a panel's or sheet's title row.
//! - [`list`] — one- or two-line items with leading and trailing slots.
//! - [`divider`] — a hairline between groups.
//! - [`dialog`] — the shell's alert, an app's modal.
//! - [`status_page`] — an empty or unreachable list, said once and plainly.
//! - [`toast`] — a short word that something was done, with an Undo.
//! - [`tooltip`] — a hovered control's label.
//! - [`scroll_view`] — a clipped viewport scrolled along one axis, with a
//!   scrollbar.
//! - [`flow`] — fixed-size items flowing into rows, wrapping at a width.
//! - [`form`] — settings laid out in rounded groups of rows,
//!   each a coloured tile, a title and its control.
//! - [`arrangement`] — screens dragged into place, as Displays arranges them.
//!
//! The look of glass (`look/`):
//!
//! - [`material`] — the materials: glass by role, its thickness and
//!   the vibrant labels on it.
//! - [`glass`] — the lit edge and glow every translucent surface wears.
//! - [`glass_region`] — where the compositor frosts behind clear glass.
//! - [`lab`] — Shader Lab's saved looks, and putting one on a rectangle.
//! - [`style`] — the colours, sizes and fonts the form controls share.
//!
//! Motion (`animation/`):
//!
//! - [`motion`] — the shared springs and fades, and keyed list motion.
//! - [`kinetic`] — following fingers, and the momentum a flick leaves.
//! - [`pager`] — turning pages with the fingers, a flick or a wheel.
//! - [`reveal`] — a surface opening on a spring and closing quicker.
//! - [`stagger`] — motion in parts, one after another, on one clock.
//! - [`edge_reveal`] — a surface hiding at a screen edge until the pointer
//!   touches it.
//!
//! Primitives (`foundation/`), with no look of their own:
//!
//! - [`layout`] — layout primitives (`boxed`, `pad`, `icon`, `split_row` …).
//! - [`icons`] — the Material Symbols glyphs and their pickers.
//! - [`color`] — colour arithmetic (`over`, `faded`, `with_alpha`).
//! - [`symbol`] — the few drawn symbols the controls use (chevrons, checks).
//! - [`fuzzy`] — how well typed letters match a name, for searches.
//! - [`paths`] — the XDG base folders: config, state, cache, data, runtime.
//!
//! The system around an app (`platform/`):
//!
//! - [`app`] — an app's window, text fields, dialogs and sidebar.
//! - [`app_icon`] — an app's icon from its desktop entry, by app_id.
//! - [`settings`] — the shell's settings file, which a settings app changes.
//! - [`sounds`] — the sound theme's sounds, and playing one.
//! - [`mod@marker`] — scoped redraws for state kept outside a service.
//!
//! And at the root, [`tokens`] and [`theme`]: the design tokens and the
//! mode, density, accent and text scale they resolve with.

#![deny(missing_docs)]
// no unsafe code: what needs it (syscalls, FFI) lives in emaki-base or amane
#![forbid(unsafe_code)]

mod animation;
mod containers;
mod controls;
mod foundation;
mod look;
mod platform;

pub mod theme;
pub mod tokens;

pub use animation::{edge_reveal, kinetic, motion, pager, reveal, stagger};
pub use containers::{
    app_bar, arrangement, card, dialog, divider, flow, form, grid, list, scroll_view, status_page,
    toast, tooltip,
};
pub use controls::{
    avatar, buttons, calendar, color_picker, control_size, dropdown, leading_icon, menu,
    nav_buttons, pop_up_button, search_field, slider, spinner, stepper, tag, text_edit,
    thumbnail_picker,
};
pub use foundation::{area, color, fuzzy, icons, layout, symbol};
pub use look::{glass, glass_region, lab, material, style};
pub use platform::{
    app, app_icon, launch, marker, process, secrets, sensors, service, settings, shell_ipc, sounds,
    usb_c,
};

// what's below the UI, from emaki-base, so an app needs only the kit
pub use emaki_base::{
    app_ids, args, format, fs, harden, intern, migrate, mime, nix, paths, peer, percent, sync,
    time, tools,
};
