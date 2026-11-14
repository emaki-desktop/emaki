//! Notification center — a full-height sidebar at the right edge that
//! slides in when the clock is clicked, holding every notification of the
//! session until it's dismissed.
//!
//! Notifications are grouped by app, newest group first. A group of
//! several is a stack: the newest card in front, up to
//! `notification.stack.max-peek` blank plates peeking below it, each
//! smaller and fainter. Clicking the stack springs it open into a list
//! under the app's name, with "Show Less" and a clear button; a single
//! notification is a plain card. Cards that go fade and slide away.

use std::cell::RefCell;
use std::collections::HashSet;
use std::time::SystemTime;

use amane::{
    Animation, Center, Column, Full, Horizontal, Keyboard, LayerWindow, Margin, Notification,
    Notifications, Parent, Rectangle, ScrollArea, Service, Stack, Text, Vertical, Widget,
    window_size,
};

use super::card::{self, Measured, Place};
use crate::chrome::panel::{self, Panel, dismissable, is_open, overlay};
use crate::services::clock::{self, Clock};
use crate::services::do_not_disturb::{DoNotDisturb, Until};
use crate::tokens::{Styled, notification, sys};
use emaki_kit::app_bar::TopAppBar;
use emaki_kit::buttons::{IconButton, PushButton, Shape, Style};
use emaki_kit::card::Card;
use emaki_kit::icons::ICON_DND;
use emaki_kit::layout::{blank, boxed, cursor, frame, split_row};
use emaki_kit::material::{self, Level, Material};
use emaki_kit::motion::{self, Arrive, Item, Pose};

/// Layer-surface namespace; the niri `layer-rule` matches it for blur, and
/// the Do Not Disturb menu hangs under it.
pub const CENTER_NS: &str = "emaki.notifications.center";

/// Every list of cards in the center is a `motion` scope under this.
const SCOPE: &str = "center.";

thread_local! {
    // 0 hidden off the right edge, 1 in place
    static SLIDE_IN: RefCell<Animation> = RefCell::new(motion::sheet(0.0));

    // the groups opened out from a stack, by key
    static EXPANDED: RefCell<HashSet<String>> = RefCell::new(HashSet::new());

    // the groups drawn last time, so one whose last card went can finish
    // sliding it out
    static DRAWN: RefCell<Vec<(String, String)>> = const { RefCell::new(Vec::new()) };
}

// ───── runtime ───────────────────────────────────────────────────────────────

/// Tells the panels what the center does as it opens and closes. Once, at
/// startup.
pub fn register_panel() {
    panel::register(Panel::Notifications, panel::Hooks { opened, closed });
}

/// The center opened: toasts go, everything counts as seen.
fn opened() {
    super::toasts::Toasts::hide_all();
    let newest = Notifications::read()
        .list()
        .iter()
        .map(Notification::id)
        .max()
        .unwrap_or(0);
    crate::services::notification_history::mark_seen(newest);
}

/// The center closed: stacks fold back up, and the calendar comes back to
/// this month, for next time.
fn closed() {
    EXPANDED.with_borrow_mut(HashSet::clear);
    super::calendar_card::reset();
}

/// How many arrived since the center was last opened; what was restored
/// from before a restart was there already.
pub fn unseen_count() -> usize {
    crate::services::notification_history::unseen(false)
}

// ───── groups ────────────────────────────────────────────────────────────────

struct Group {
    key: String,
    name: String,
    // newest first
    cards: Vec<Notification>,
}

/// What groups a notification: its desktop entry, else its app name, the
/// key its app's rule is kept under too.
fn group_key(desktop_entry: &str, app_name: &str) -> String {
    emaki_kit::settings::notification_app_key(desktop_entry, app_name)
}

/// Groups newest first, each newest card first. `list` is oldest first,
/// as amane keeps it.
fn groups(list: &[Notification]) -> Vec<Group> {
    let mut groups: Vec<Group> = Vec::new();
    for n in list.iter().rev() {
        let key = group_key(n.desktop_entry(), n.app_name());
        match groups.iter_mut().find(|group| group.key == key) {
            Some(group) => group.cards.push(n.clone()),
            None => groups.push(Group {
                key,
                name: card::app_name(n).to_string(),
                cards: vec![n.clone()],
            }),
        }
    }
    groups
}

/// A card was clicked: its default action, or for one restored from
/// before a restart, whose sender can't hear it, its app opens.
fn click(id: u32) {
    let restored = Notifications::read()
        .list()
        .iter()
        .find(|n| n.id() == id)
        .filter(|n| n.restored())
        .map(|n| String::from(n.desktop_entry()));
    match restored {
        Some(entry) => {
            emaki_kit::app_icon::with_entry(&entry, amane::DesktopApp::launch);
            Notifications::dismiss(id);
        }
        None => Notifications::click(id),
    }
}

// ───── view ──────────────────────────────────────────────────────────────────

/// The center's surface: a sheet sliding in from the right edge.
pub fn side_sheet() -> LayerWindow {
    let open = is_open(Panel::Notifications);
    let progress = SLIDE_IN.with_borrow_mut(|slide| {
        slide.to(if open { 1.0 } else { 0.0 });
        slide.value()
    });
    let shown = open || progress > 0.001;

    let width = notification::center::width();
    let gap = sys::space::gap::m();
    // slides from the right edge to `gap` in from it, or fades in there where
    // motion is reduced. A pixel of it stays on the output: niri sends no
    // frame callbacks to a layer surface it doesn't draw, so a sheet parked
    // wholly past the edge would wait up to a second to start moving.
    let travel = motion::travel(width + gap - 1.0);
    let right = gap - (1.0 - progress) * travel;
    let opacity = if travel == 0.0 {
        progress.clamp(0.0, 1.0)
    } else {
        1.0
    };

    let base = LayerWindow::new()
        .width(width)
        .height(Full)
        .margin(Margin {
            top: gap as i32,
            right: right.round() as i32,
            bottom: gap as i32,
            left: 0,
        })
        .visible(shown);
    // niri focuses an on-demand surface as it maps, which turns the focused
    // window inactive (its sidebar dims); the sheet takes keys once it's in
    // place and clicked into
    let keyboard = if open && progress < 0.999 {
        Keyboard::None
    } else {
        Keyboard::OnDemand
    };
    let base = dismissable(
        overlay(base, CENTER_NS, Vertical::Top, Horizontal::Right).output(panel::on_output()),
        keyboard,
    );

    if !shown {
        motion::forget(SCOPE);
        DRAWN.with_borrow_mut(Vec::clear);
        return base.child(blank());
    }

    // the whole surface is the sheet's glass; it slides with the surface
    let frost = Material::Sheet.shape(
        0.0,
        0.0,
        width,
        window_size().1,
        notification::center::radius(),
        opacity,
    );
    base.blur_shapes(vec![frost])
        .child(sheet().opacity(opacity))
}

fn sheet() -> Rectangle {
    // a clone, so no lock is held while handlers are built
    let list: Vec<Notification> = Notifications::read().list().to_vec();
    // read for its minute tick, which keeps "5m ago" current
    drop(Clock::read());
    let now = SystemTime::now();

    // the month stays put over the list, which scrolls under it
    let mut rows: Vec<Box<dyn Widget>> = vec![
        boxed(header(!list.is_empty())),
        boxed(super::calendar_card::card()),
    ];

    let groups = groups(&list);
    let mut drawn = Vec::new();
    let mut views: Vec<Box<dyn Widget>> = Vec::new();
    for group in &groups {
        drawn.push((group.key.clone(), group.name.clone()));
        if let Some(view) = group_view(&group.key, &group.name, &group.cards, now) {
            views.push(view);
        }
    }
    // a group that just emptied keeps drawing while its cards slide out
    for (key, name) in DRAWN.with_borrow(Vec::clone) {
        if !groups.iter().any(|group| group.key == key)
            && let Some(view) = group_view(&key, &name, &[], now)
        {
            drawn.push((key, name));
            views.push(view);
        }
    }
    DRAWN.with_borrow_mut(|old| *old = drawn);

    if views.is_empty() {
        rows.push(boxed(
            blank().align_child(Center, Center).child(
                Text::new("No Notifications")
                    .style(sys::r#type::label())
                    .color(material::label(Level::Secondary)),
            ),
        ));
    } else {
        rows.push(boxed(ScrollArea::new(
            "emaki.notifications.center.list",
            Column::new(views).gap(sys::space::gap::l()).width(Parent),
        )));
    }

    // nested about its cards: their corners plus the inset around them
    Card::new(rows)
        .material(Material::Sheet)
        .radius(notification::center::radius())
        .fill(Material::Sheet.fill())
        .glass(true)
        .gap(sys::space::gap::l())
        .build()
}

/// "Notifications", what Do Not Disturb is doing, and the sheet's buttons.
fn header(any: bool) -> amane::Row {
    let until = DoNotDisturb::read().until();

    let open_menu = || super::dnd_menu(CENTER_NS, cursor());
    let dnd = IconButton::new("emaki.notifications.dnd", ICON_DND)
        .style(Style::Tonal)
        .color(material::label(Level::Secondary))
        .selected(until.is_some())
        .on_press(open_menu)
        .on_secondary(open_menu)
        .build();

    let mut trailing: Vec<Box<dyn Widget>> = Vec::new();
    if any {
        trailing.push(boxed(text_button(
            "emaki.notifications.clear-all",
            "Clear All",
            Notifications::clear,
        )));
    }
    trailing.push(boxed(dnd));

    let mut bar = TopAppBar::new("Notifications").actions(trailing);
    if let Some(until) = until {
        bar = bar.subtitle(dnd_status(until));
    }
    bar.build()
}

fn dnd_status(until: Until) -> String {
    match until {
        Until::Forever => String::from("Do Not Disturb"),
        Until::At(end) => format!("Do Not Disturb until {}", clock::local(end, "%H:%M")),
    }
}

/// A small capsule text button on the sheet.
fn text_button(
    id: impl Into<String>,
    label: &'static str,
    press: impl Fn() + 'static,
) -> Rectangle {
    PushButton::new(id, label)
        .shape(Shape::Round)
        .color(material::label(Level::Primary))
        .on_press(press)
        .build()
}

/// What a stack draws: its title row, or a card.
#[derive(Clone)]
enum Entry {
    Header,
    Card(Box<Measured>),
}

const HEADER_KEY: u64 = u64::MAX;

/// One app's notifications: a stack, a list, or a lone card. `None` once
/// nothing in it is left to draw.
fn group_view(
    key: &str,
    name: &str,
    cards: &[Notification],
    now: SystemTime,
) -> Option<Box<dyn Widget>> {
    let width = Place::Center.width();
    let many = cards.len() > 1;
    let expanded = many && EXPANDED.with_borrow(|open| open.contains(key));
    let header_h = sys::size::control::height();

    let measured: Vec<Measured> = cards
        .iter()
        .map(|n| Measured::new(n.clone(), Place::Center))
        .collect();
    let heights: Vec<f32> = measured.iter().map(|m| m.layout.height).collect();
    let poses = stack_poses(&heights, expanded, header_h + sys::space::gap::s());

    let mut items: Vec<Item<Entry>> = Vec::new();
    if expanded {
        let pose = Pose::at(0.0);
        items.push(Item {
            key: HEADER_KEY,
            value: Entry::Header,
            height: header_h,
            pose,
            enter: pose.faded(),
            exit: pose.faded(),
        });
    }
    for (m, pose) in measured.into_iter().zip(&poses) {
        items.push(Item {
            key: u64::from(m.notification.id()),
            height: m.layout.height,
            value: Entry::Card(Box::new(m)),
            pose: *pose,
            enter: Pose {
                scale: motion::scale(motion::enter_scale()),
                ..pose.faded()
            },
            // where motion is reduced it fades where it is
            exit: pose.moved(motion::travel(width)).faded(),
        });
    }

    let scope = format!("{SCOPE}{key}");
    let shown = motion::animate(&scope, items, Arrive::Rest);
    if shown.is_empty() {
        return None;
    }

    let height = shown
        .iter()
        .filter(|s| s.pose.opacity > 0.01)
        .map(|s| s.pose.bottom(s.height))
        .fold(0.0, f32::max);

    // drawn bottom up: cards on their way out, the plates from the back,
    // the front card, then the title
    let rank = |key: u64| {
        if key == HEADER_KEY {
            return usize::MAX;
        }
        cards
            .iter()
            .position(|n| u64::from(n.id()) == key)
            .map_or(0, |i| cards.len() - i)
    };
    let mut order: Vec<_> = shown.into_iter().collect();
    order.sort_by_key(|s| (!s.leaving, rank(s.key)));

    let group = key.to_string();
    let ids: Vec<u32> = cards.iter().map(Notification::id).collect();
    let layers: Vec<Box<dyn Widget>> = order
        .into_iter()
        .map(|s| {
            let pose = s.pose;
            let view = match s.value {
                Entry::Header => group_header(name, &group, ids.clone(), width, header_h),
                Entry::Card(m) => {
                    let id = m.notification.id();
                    let group = group.clone();
                    let view = card::card(&m, Place::Center, now, pose.content);
                    card::interactive(
                        view,
                        id,
                        |_| {},
                        move || {
                            if many && !expanded {
                                EXPANDED.with_borrow_mut(|open| open.insert(group.clone()));
                            } else {
                                click(id);
                            }
                        },
                    )
                }
            };
            boxed(
                view.translate(pose.x, pose.y)
                    .scale(pose.scale)
                    .opacity(pose.opacity.clamp(0.0, 1.0)),
            )
        })
        .collect();

    Some(boxed(
        frame(width, height.max(1.0))
            .clip()
            .child(Stack::new(layers)),
    ))
}

/// Where each card of a group goes, newest first. Opened out, they're a
/// list under the group's title (`top` tall); stacked, the newest is in
/// front and the next few are blank plates peeking out below it, each a
/// step smaller and fainter, and the rest hide behind.
fn stack_poses(heights: &[f32], expanded: bool, top: f32) -> Vec<Pose> {
    let gap = sys::space::gap::s();
    let peek = notification::stack::peek();
    let max_peek = notification::stack::max_peek() as usize;
    let scale_step = notification::stack::scale_step();
    let fade_step = notification::stack::fade_step();
    let front = heights.first().copied().unwrap_or(0.0);

    let mut y = if expanded && heights.len() > 1 {
        top
    } else {
        0.0
    };
    heights
        .iter()
        .enumerate()
        .map(|(i, &height)| {
            if expanded || i == 0 {
                let pose = Pose::at(y);
                y += height + gap;
                return pose;
            }
            let step = i.min(max_peek) as f32;
            let scale = 1.0 - scale_step * step;
            // its bottom edge shows `peek` below the one in front of it
            let bottom = front + peek * step;
            Pose {
                x: 0.0,
                y: bottom - height * (1.0 + scale) / 2.0,
                scale,
                opacity: if i > max_peek {
                    0.0
                } else {
                    1.0 - fade_step * step
                },
                content: 0.0,
            }
        })
        .collect()
}

/// An opened group's title: the app's name, "Show Less", and clear.
fn group_header(name: &str, group: &str, ids: Vec<u32>, width: f32, height: f32) -> Rectangle {
    let collapse = group.to_string();
    let clear = card::close_button(
        format!("emaki.notifications.group.{group}.clear"),
        height,
        material::label(Level::Primary),
        move || {
            for id in &ids {
                Notifications::dismiss(*id);
            }
        },
    );
    let less = text_button(
        format!("emaki.notifications.group.{group}.collapse"),
        "Show Less",
        move || {
            EXPANDED.with_borrow_mut(|open| open.remove(&collapse));
        },
    );

    frame(width, height).child(
        split_row(
            Text::new(name.to_string())
                .style(sys::r#type::title())
                .color(material::label(Level::Primary)),
            vec![boxed(less), boxed(clear)],
        )
        .height(Parent),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_stack_peeks_and_an_open_group_lists() {
        let heights = [60.0, 80.0, 60.0, 60.0];

        let stacked = stack_poses(&heights, false, 30.0);
        assert_eq!(stacked[0], Pose::at(0.0));
        // each plate behind ends lower, smaller and fainter
        assert!(stacked[1].bottom(80.0) > stacked[0].bottom(60.0));
        assert!(stacked[2].bottom(60.0) > stacked[1].bottom(80.0));
        assert!(stacked[1].scale > stacked[2].scale);
        assert!(stacked[1].opacity > stacked[2].opacity);
        assert_eq!(stacked[1].content, 0.0);
        // past the peeking ones, the rest hide behind
        assert_eq!(stacked[3].opacity, 0.0);

        let open = stack_poses(&heights, true, 30.0);
        assert_eq!(open[0].y, 30.0);
        assert!(
            open.iter()
                .all(|pose| pose.scale == 1.0 && pose.content == 1.0)
        );
        assert!(open[1].y > open[0].y + 60.0);
    }
}
