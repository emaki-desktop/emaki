//! The on-screen display: a small glass capsule above the bottom edge that
//! shows a level or a switch as it changes (volume, the microphone muted,
//! brightness, the keyboard light, a lock key, airplane mode, the power
//! mode, the sound moving to another device), then fades out, as GNOME
//! shows the media keys.
//!
//! The keys come to the shell over IPC ([`keys`]), which changes the level
//! and shows it at once; changes made anywhere else (a script, a headset's
//! buttons, the keyboard light's own key) show too, noticed by the services
//! they change ([`watch`]). See `docs/user/features/osd.md`.
//!
//! One surface, placed by niri on the focused output, as big as the
//! capsule and taking no input. Hidden, it's `visible(false)` and holds
//! nothing on the GPU; it draws only rectangles, text and glyphs.

mod keys;
mod watch;

use std::cell::{Cell, RefCell};
use std::time::{Duration, Instant};

use amane::{
    AudioKind, Center, Horizontal, Keyboard, LayerWindow, Margin, Rectangle, Row, Start, Text,
    Vertical, Widget,
};

use crate::chrome::panel::{self, overlay};
use crate::services::feedback;
use crate::tokens::{Styled, osd, sys};
use emaki_kit::glass_region;
use emaki_kit::icons::{
    ICON_BRIGHTNESS_HIGH, ICON_BRIGHTNESS_LOW, ICON_KEYBOARD_LIGHT, ICON_MIC, ICON_MIC_OFF,
    volume_glyph,
};
use emaki_kit::layout::{boxed, icon, pad, slot};
use emaki_kit::material::{self, FillLevel, Level, Material};
use emaki_kit::motion;
use emaki_kit::slider::Slider;

use emaki_kit::reveal::{self, Swell};
pub use keys::{run, set_brightness, set_volume, step_brightness, step_volume, toggle_mute};

emaki_kit::marker! {
    /// The display showed, changed or ran out: only its window draws again.
    pub Osd
}

/// Layer-surface namespace; the niri `layer-rule` matches it.
pub const OSD_NS: &str = "emaki.osd";

/// How long a reading stays before it fades.
const LINGER: Duration = Duration::from_millis(1500);

/// A device's name is longer to read than a level.
const DEVICE_LINGER: Duration = Duration::from_millis(2200);

/// Keyboard lights with this many steps or fewer show them as pips.
const MAX_PIPS: u32 = 4;

/// What the display shows.
#[derive(Clone, Debug, PartialEq)]
pub enum Readout {
    /// The output's level, 0 … 100.
    Volume {
        level: u8,
        muted: bool,
        kind: AudioKind,
    },
    /// The microphone's level and whether it's muted.
    Mic { level: u8, muted: bool },
    /// The screen's brightness, 0 … 100.
    Brightness(u8),
    /// The keyboard light, in its steps.
    KeyboardLight { level: u32, max: u32 },
    /// Something switched or picked: a glyph and what it now is.
    Label { glyph: char, text: String },
    /// A device came or went, posted by [`feedback::echo`]: its glyph, its name, and a
    /// quieter detail after it, like "Connected · 80 %".
    Device {
        glyph: char,
        name: String,
        detail: String,
    },
}

impl Readout {
    fn linger(&self) -> Duration {
        match self {
            Self::Label { .. } | Self::Device { .. } => DEVICE_LINGER,
            _ => LINGER,
        }
    }
}

thread_local! {
    // what shows, kept while it fades out
    static SHOWN: RefCell<Option<Readout>> = const { RefCell::new(None) };
    // when it starts to fade
    static UNTIL: Cell<Option<Instant>> = const { Cell::new(None) };

    // how far it has faded in
    static FADE: RefCell<amane::Animation> = RefCell::new(motion::fade(0.0, motion::duration::fade()));
    static OPENING: Cell<bool> = const { Cell::new(false) };
}

/// The next device echo from [`feedback`], when nothing shows. While
/// Quick Settings is open its rows already show what changed, so the
/// echoes go unshown.
fn next_echo(open: bool) -> Option<Readout> {
    feedback::Echoes::watch();
    let controls = panel::is_open(panel::Panel::QuickSettings);
    feedback::take_echo(open, controls).map(|echo| Readout::Device {
        glyph: echo.glyph,
        name: echo.name,
        detail: echo.detail,
    })
}

/// Shows `readout`, in place of whatever shows, for its linger from now.
/// Showing again while it's up changes the capsule in place.
pub fn show(readout: Readout) {
    let linger = readout.linger();
    SHOWN.set(Some(readout));
    UNTIL.set(Some(Instant::now() + linger));
    motion::redraw_after::<Osd>(linger);
    Osd::changed();
}

/// Steps the fade towards `open` and gives how far in it is.
fn faded_in(open: bool) -> f32 {
    FADE.with_borrow_mut(|anim| {
        if OPENING.replace(open) != open {
            anim.retime(if open {
                motion::duration::fade()
            } else {
                motion::duration::exit()
            });
            anim.to(if open { 1.0 } else { 0.0 });
        }
        anim.value()
    })
}

/// The surface: hidden while nothing shows, else the capsule.
pub fn surface() -> LayerWindow {
    // reading the services here wakes this view when one changes
    watch::changes();
    Osd::watch();

    let mut open = UNTIL.get().is_some_and(|until| Instant::now() < until);
    if let Some(readout) = next_echo(open) {
        show(readout);
        open = true;
    }
    let shown = faded_in(open);

    let readout = SHOWN.with_borrow(Clone::clone);
    let Some(readout) = readout.filter(|_| reveal::showing(open, shown)) else {
        if !open {
            SHOWN.set(None);
            UNTIL.set(None);
        }
        return panel::empty(window());
    };

    let (width, height, radius) = (osd::width(), osd::height(), osd::radius());

    // it fades in where it is, without swelling
    let swell = Swell::new(open, shown, 1.0);

    // small glass, light or dark with its backdrop
    let glass = glass_region::shape_id(OSD_NS);
    let capsule = Material::Control
        .adapted(glass, || {
            let face = Rectangle::new()
                .width(width)
                .height(height)
                .radius(radius)
                .fill(Material::Control.fill())
                .padding(pad(osd::pad(), 0.0))
                .align_child(Start, Center)
                .child(content(&readout, width - osd::pad() * 2.0));
            Material::Control.lit(face, radius)
        })
        .opacity(swell.opacity);

    let frost = if swell.frost {
        let shape = Material::Control.adapted(glass, || {
            Material::Control.shape(0.0, 0.0, width, height, radius, swell.opacity)
        });
        vec![shape.id(glass)]
    } else {
        Vec::new()
    };

    window().visible(true).blur_shapes(frost).child(capsule)
}

/// What's on the capsule, `width` wide: a glyph, then a level or a label.
fn content(readout: &Readout, width: f32) -> Row {
    let gap = sys::space::gap::m();
    let side = sys::size::control::height();
    let rest = (width - side - gap).max(1.0);
    let label = material::label(Level::Primary);

    let (glyph, after): (char, Box<dyn Widget>) = match readout {
        Readout::Volume { level, muted, kind } => (
            volume_glyph(*muted, *level, *kind),
            boxed(meter(*level, *muted, rest)),
        ),
        Readout::Mic { level, muted } => (
            if *muted { ICON_MIC_OFF } else { ICON_MIC },
            boxed(meter(*level, *muted, rest)),
        ),
        Readout::Brightness(level) => (
            if *level < 50 {
                ICON_BRIGHTNESS_LOW
            } else {
                ICON_BRIGHTNESS_HIGH
            },
            boxed(meter(*level, false, rest)),
        ),
        Readout::KeyboardLight { level, max } if *max <= MAX_PIPS => {
            (ICON_KEYBOARD_LIGHT, boxed(pips(*level, *max, rest)))
        }
        Readout::KeyboardLight { level, max } => {
            let percent = (level * 100 / (*max).max(1)).min(100) as u8;
            (ICON_KEYBOARD_LIGHT, boxed(meter(percent, false, rest)))
        }
        Readout::Label { glyph, text } => (
            *glyph,
            boxed(
                Text::new(text.clone())
                    .style(osd::label())
                    .color(label)
                    .elide()
                    .max_lines(1),
            ),
        ),
        Readout::Device {
            glyph,
            name,
            detail,
        } => (*glyph, boxed(device_line(name, detail, rest))),
    };

    let glyph = slot(side).child(icon(glyph, sys::size::icon::m(), label));
    Row::new(vec![boxed(glyph), after])
        .width(width)
        .gap(gap)
        .align(Center)
}

/// A device's name, cut short before its detail is, then the detail quieter.
fn device_line(name: &str, detail: &str, width: f32) -> Row {
    let name = Text::new(name.to_string())
        .style(osd::label())
        .color(material::label(Level::Primary))
        .elide()
        .max_lines(1);
    if detail.is_empty() {
        return Row::new(vec![boxed(name)]).width(width).align(Center);
    }
    let detail = Text::new(detail.to_string())
        .style(osd::label())
        .color(material::label(Level::Secondary))
        .max_lines(1);
    Row::new(vec![boxed(name), boxed(detail)])
        .width(width)
        .gap(sys::space::gap::s())
        .align(Center)
}

/// A level as Quick Settings' thin track draws it, greyed while muted.
fn meter(level: u8, muted: bool, width: f32) -> Rectangle {
    Slider::new(level, width)
        .inert(true)
        .dimmed(muted)
        .fill(material::label(Level::Primary))
        .thickness(osd::track())
        .build()
}

/// A light with few steps, one pip a step, the lit ones filled.
fn pips(level: u32, max: u32, width: f32) -> Row {
    let gap = sys::space::gap::s();
    let track = osd::track();
    let each = ((width - gap * (max - 1) as f32) / max as f32).max(1.0);
    let pips = (0..max)
        .map(|step| {
            let fill = if step < level {
                material::label(Level::Primary)
            } else {
                material::fill_role(FillLevel::Secondary)
            };
            boxed(
                Rectangle::new()
                    .width(each)
                    .height(track)
                    .radius(track / 2.0)
                    .fill(fill),
            )
        })
        .collect();
    Row::new(pips).gap(gap).align(Center)
}

/// The surface, as big as the capsule, `osd.offset` above the bottom edge
/// (clear of the bar there, which reserves its zone), in the middle, taking
/// neither the keyboard nor the pointer.
fn window() -> LayerWindow {
    let base = LayerWindow::new()
        .width(osd::width())
        .height(osd::height())
        .margin(Margin {
            top: 0,
            right: 0,
            bottom: osd::offset().round() as i32,
            left: 0,
        })
        .keyboard(Keyboard::None)
        .click_through();
    overlay(base, OSD_NS, Vertical::Bottom, Horizontal::Middle)
}
