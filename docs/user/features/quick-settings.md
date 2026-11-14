# Quick Settings

GNOME's quick settings menu, drawn as a grid of glass modules
(`apps/shell/src/surfaces/quick_settings/`). The card itself has no background: it's a
clear surface above the bar's right end holding frosted glass modules — Wi‑Fi and
Bluetooth tiles beside Now Playing, the Focus tile and round buttons, a
Display card and a Sound card with their sliders, then the battery and more round buttons.
Each control can open a page of its own, and an edit mode moves, resizes, removes and adds them.

It's opened from the bar's Quick Settings button (the slider glyph, between the battery and the
notification bell), and closed by the same button, by Escape, or by opening the notification
center, whose one-open-at-a-time state it shares (`Panel::QuickSettings`). It's the only place
for sound, Wi‑Fi, Bluetooth and power: the bar's volume, Bluetooth, network and battery icons open
their page **on its own**: no grid behind it and no back chevron, only that icon lit (the Quick
Settings button lights only for the whole card), Escape closes it, and the icon again closes it; the
Quick Settings button over such a page brings the whole card. Each of those pages has a **Show in
Status Bar** switch for its icon (see [status-bar.md](status-bar.md#status-icons)). A page inside a
page (a Bluetooth device's) keeps its chevron back to the page it's in.

While the card shows, its surface covers the output (clear of the bar), so a click anywhere outside
the card closes it, as the launcher does; the bar stays clickable, so another status icon swaps
the page. In edit mode clicks outside the card and the Customize window go through instead.

## What's in it

In the default order (edit mode changes it). Every control shows only
when the machine has what it drives, as in GNOME, and the grid packs around what's missing:

| Control | Shows | Click | Page |
|---|---|---|---|
| Wi‑Fi | the network joined, "Connecting…", "Not Connected", "Off" or "Hardware switch off"; "<ssid> Hotspot" while the machine is one | the disc switches the radio (or stops the hotspot); the tile opens the page | the networks (rescanned every 15 s while shown; "Searching…" or "Wi‑Fi is off" while there are none), joining with a password; the page stays open and the row says "Connecting…" |
| Bluetooth | the device connected, how many are, "Turning On…" or "Off" | the disc switches the adapter (lifting rfkill's soft block first); the tile opens the page | My Devices and Other Devices (named ones, the strongest first), searching and discoverable as this computer's name while shown; a row says "Connecting…", "Pairing…" or why it failed ("Not reachable"), else the battery and a check; a paired device's ⓘ or right click opens its own page (Connect, Battery, Connect Automatically, Name, kind and address, Forget This Device… with an alert); a pairing code to confirm, type or show sits under the list. Off, it says why: off, airplane mode or a hardware switch |
| Phone | a tile, waiting in the window's Suggestions by default; the phone connected and its charge, "Not Connected", "No Phone" or "Off"; hidden without KDE Connect | opens the page | My Devices and Other Devices, a phone's request to pair with its code; a phone's own page with Battery, Ring, Send Files…, Messages and Phone Input, the clipboard, the phone's remote for this computer and Forget ([phone.md](phone.md)) |
| Airplane Mode | a round button (or a tile) | switches every radio | — |
| Tailscale | a tile with Tailscale's mark (drawn, as the font hasn't got it), waiting in the window's Suggestions by default; "via <exit node>", this device's address, "Connecting…", "Not logged in", "Disconnected" or "Daemon not running"; hidden without the `tailscale` binary | the disc connects or disconnects (`tailscale up` / `down`), or logs in when logged out; the tile opens the page | what's wrong (no daemon; **Log In…**, which opens the browser once the control plane hands out the URL; no operator grant, with the command to copy; a failed change; tailscaled's own warnings), This Device (a click copies the address), Exit Node (None or a peer that offers itself; offline ones can't be picked), Accept Routes, Accept DNS and Block Incoming switches, and the tailnet's Devices, online first (a click copies the address); Show in Status Bar; Admin Console… |
| VPN, Wired | tiles, waiting in the window's Suggestions by default; VPN says "Off", the tunnel that's up or how many are; Wired the connection or "Not Connected" | the disc switches; the VPN tile opens its page | VPN: a switch in the header for the preferred tunnel, and one per profile |
| Now Playing | the active player's art (from a file or the web: only from a public address, with no redirects, at most 10 MB, the newest 200 kept), title and artist, how far through the track, or "Not Playing" | previous, play/pause, next | — |
| Focus | Do Not Disturb: what's on, until when, or off | the disc switches it; the tile opens the page | Focus: For 1 Hour, Until This Evening, Until Tomorrow, the one that's on checked |
| Keep Awake | a tile with a cup, waiting in the window's Suggestions by default; "Off", "No limit", the time left ("1 h 24 min"), "Lid ignored" or "Lid · 1 h 24 min"; lit while anything is held | the disc holds the screen on with no limit, or lets go of everything, the lid too; the tile opens the page | how long (No limit, 15 min, 30 min, 1 h, 2 h; picking one starts it now), Keep screen on and Ignore lid close (hidden without logind; the last one off turns it all off), Turn the screen off now (the monitors go off a second later, nothing is let go, and input brings them back); Show in Status Bar |
| Dark Style, Night Light | round buttons, white with the glyph in the accent while on | switch; a right click on Night Light opens its page | Night Light: how warm, or "Scheduled" when it follows a schedule set on the [Displays page](displays.md#night-light) |
| Display | only with a backlight; the brightness slider between a dim and a bright glyph | drag or scroll | — |
| Sound | the default output's level; the small glyph says how loud (off, down, up) | the small glyph mutes; drag or scroll sets the level; the chevron or a right click opens the page | Sound: the output's level and percentage (its glyph mutes it) and its devices, then a level for each app playing (its icon, name and what's playing, a slider whose glyph mutes that app), then the microphone's level and its devices (with a microphone); the default device checked, a click makes another the default |
| Battery | only with a battery; the charge and how long it lasts, the disc in the critical color when it's low | opens its page | Battery (time left, power mode, peripherals) |
| Round buttons | Power Mode (on while not Balanced), Keyboard (the backlight), Background Apps (how many, in a tooltip), screenshot, lock, power off | switch; a right click opens the page; screenshot closes the card, then opens niri's screenshot UI; lock locks through logind | Power Mode: the modes; Keyboard: Off/Low/High (a slider for keyboards with many steps); Background Apps: the apps, each with a close button; Power Off: suspend, restart, power off, log out |

A control is a module. A click on the round disc switches the
setting, and a click anywhere else on a tile or a module row opens its page. A control with no
page switches from anywhere. A right click opens the page too; a round button has no label, so a
tooltip names it, and a tile whose lines are too long for it names itself the same way. The
Power Off page's Restart…, Power Off… and Log Out… close Quick Settings and ask first, in the
shell's alert ("Are you sure you want to restart your computer now?"; see
[the alert](#the-alert)). Log Out's answer is destructive, in red. Switch
User is left out because greetd has no user switching.

A page is at least five rows tall, however short the grid is, and its lists scroll in the room
they have. A page whose setting goes away while it's open (the adapter unplugged) gives way to
the grid, and `quick-settings open` answers with an error for a page the machine can't show.

Now Playing is always there, saying "Not Playing" when nothing is; everything else
follows GNOME's rule and hides without its hardware.

A page's last row opens the rest of its settings: Emaki Settings' [Network page](network.md) for
Wi‑Fi and VPN, and its [Sound page](sound.md) for Sound. Without Emaki Settings, they open
`nm-connection-editor` and `pavucontrol` when installed. Bluetooth has none: its
page is the whole manager.

A device that asks to pair while the Bluetooth page isn't showing asks through the shell's alert
(`surfaces/alert/bluetooth.rs`): Pair or Cancel for a code to confirm, the code to type on a
keyboard, and **Enter Code…**, which opens the page, for a code to type here. The shell is
BlueZ's pairing agent (`Bluetooth::serve_agent`). A device connecting or disconnecting shows a
capsule, and its battery warns once at 20 % and once at 10 % per connection, and so does a
phone through KDE Connect while it isn't charging (`services/feedback/bluetooth.rs`; see
[system-feedback.md](system-feedback.md)). A phone asking to pair through KDE Connect asks
the same way (`surfaces/alert/phone.rs`) while the Phone page isn't showing.

The bar also shows GNOME's status icons, each only while it applies:

- **Privacy icons**, first: the screen being shared, the camera in use, and the microphone in use.
  The microphone's tooltip names the app.
- **Glyphs for settings that are on**: night light, a VPN, a power mode other than Balanced, and
  airplane mode.
- **Keep Awake's cup**, in the accent color while it holds anything (the time left in its
  tooltip), unless its page's Show in Status Bar is off.

Clicking any of them opens Quick Settings.

## Edit Controls

"Edit Controls" under the grid enters edit mode, as does a right click on the canvas's empty
space and `quick-settings edit` over IPC. Editing opens the **Customize
window** beside the card (`customize/`): a frosted sheet in the middle of the screen, moved left
of the card when the screen has the room and no taller than the screen allows, with a sidebar and
a pane. The card stays open at the bottom-right in edit mode: it grows to the whole canvas, the
controls go inert, every free cell shows as a faint disc, and each control gets a remove badge at
its top-left and, when it comes in more than one size, a resize handle at its bottom-right. The
chrome fades in, a control added grows into its cells and one removed shrinks away, and when the
grid gains or loses rows at rest the controls slide to their new place rather than jump.

The window:

- **The sidebar**: a search field ("Search Controls") over the categories, each a row with an
  accent tile — All Controls, Battery, Connectivity, Display & Brightness, Focus, Keyboards, Lock
  Screen, Now Playing, Power, Screenshot, Sounds and Utilities (`customize/catalog.rs` says which
  control is under which). The picked row is filled.
- **The pane**: every control of the picked category at its real size — a round button, a tile,
  the square Now Playing, the slider cards — on a plain tint rather than glass (`tile::plain`),
  its name centred under it on up to two lines, under a header per section with a hairline between
  sections. All Controls starts with **Suggestions**, the controls not on the card, then every
  category. A control this machine hasn't got shows dimmed, as it does on the canvas. Typing
  anywhere searches by name or category; the pane shows the results. The pane scrolls with
  momentum, a scrollbar showing while it moves.
- **The footer**: the hint, then **Undo Remove** (while there's something to undo), **Reset
  Layout** and **Done**, the one main action, in the accent.

Using it:

- **Add**: drag a control from the pane onto the card. It lifts under the pointer at
  `quick.drag.scale` and, once it's over the canvas, the cell its top-left is nearest takes it and
  the others make way live; an accent outline shows the cells it will land in. Let go there and it
  stays; let go anywhere else and it goes back to the pane. A control already on the card, dragged
  from the pane, lifts off its cells and follows the pointer the same way.
- **Move**: drag a control on the card; the cell its top-left is nearest takes it, and the others
  make way live: one control of the same size under it changes places with it, anything else goes
  to the first free cells. Where nothing fits, the arrangement stays as it last fit. Every preview
  is worked out from the layout as it was when the drag began, and the held control sits exactly
  where the pointer took it (the pointer less where it was grabbed), so the cell under it never
  shifts because of what the preview moved (the same cell always gives the same arrangement, and
  moving back restores the start). A control keeps its cell until it's clearly in the next (half
  a pitch and a tenth more), so a hand resting on a boundary doesn't make the grid flap. Escape
  lets go of it where it started.
- **Remove**: drag a control off the card (more than half a cell past its edge) and let go,
  or click its badge. Dragged off, the others close up
  behind it as it goes.
- **Resize**: drag the handle; the pull picks the nearest size the control comes in, by width
  and height (a round button becomes a tile, a tile three or four cells wide, Now Playing a
  full row). An outline follows the hand, the control snaps between its sizes (holding the one
  it has until another is clearly nearer), and the others make way the same way.
- **Undo Remove** puts the last removed control back where it was; **Reset Layout** restores
  the default order (as `quick-settings reset-layout` does).
- **Done** (in the window's footer, or the card's capsule, or Escape) leaves edit mode and closes
  the window. Escape clears the search first when there is one. The card's capsule is then
  glass tinted with the accent rather than filled with it: its frost carries the accent as its
  tint, which niri's material lays on like stained glass (a compositor without the material gets
  the accent as a fill).

The grid is a canvas of `quick.grid.cols` × `quick.grid.rows` (4 × 8) cells of
`quick.grid.cell` (74 px) with `quick.grid.gap` (16 px) between them, so a row is 344 px. A
control spans whole cells: a round button 1×1, a tile 2×1 (164 × 74), Now Playing and the
Now Playing 2×2 (164 × 164), a slider 3×1 (254 × 74, room for a button beside it) or
4×1 (344 × 74). Inside, a tile is padded `quick.tile.pad` (17) with `quick.tile.gap` (8) between
its disc (`quick.tile.disc`, 40) and its two lines (`quick.type.title` 14 px and `.caption` 12 px
on a `quick.type.line` 17 px line), so a 1×1 is exactly the disc and its padding. Out of edit
mode the card shows the rows in use; holes inside stay empty.

### One surface for the card and the window

amane can't carry a drag from one layer surface to another: each surface keeps its own hit areas
and the compositor only sends a drag's motion to the surface it began on. So while editing the
Quick Settings surface covers the output (as the launcher's does), and holds both the card, drawn
where it always is at the bottom-right, and the window. Its input region is the two of them, so
clicks anywhere else go through to what's under them. One still layer under both
(`customize::drag_layer`) is what every drag is measured on: a press on a control in the pane
takes hold of it (`edit::lift`), a press on the canvas picks up or resizes what's there, and the
control in hand is drawn by the surface over card and window alike (`customize::ghost`), its grid
copy clear, so it's never cut off at the card's edge. The surface grows to the output as edit
mode begins and shrinks back to the card once the window has faded out, both at rest. The card's
place on the surface is worked out from the surface's size in both modes, so the one frame amane
draws at the old size as the mode changes draws the card where it is.

The layout is kept in the state file `quick-settings.json`
([services.md](../../developer/integration/services.md#settings-and-state)); the older `quick-settings`, the
same shape with `"version": 2` inside, is read when that isn't there:

```json
{ "version": 1, "data": { "controls": [ { "id": "focus", "cols": 2, "rows": 1, "col": 0, "row": 2 }, … ] } }
```

The ids are `wifi`, `bluetooth`, `now-playing`, `vpn`, `wired`, `focus`, `dark-style`,
`night-light`, `display`, `sound`, `battery`, `power-mode`, `keyboard`, `airplane`,
`background-apps`, `screenshot`, `lock`, `power-off`, `tailscale`, `keep-awake`,
`software-update` and `phone`. A control
without `col`/`row` is hidden (it waits in the window's Suggestions). A file from another
version is left alone and the default shown. One that places something off the canvas or over another control moves it to
the first free cells (or hides it); a control the file doesn't name is added hidden, so a control
that arrives with an update or new hardware waits in Suggestions rather than tearing a hole in
the canvas. The default layout packs the default order onto the canvas for the controls this machine
has, with VPN, Wired, Tailscale, Keep Awake and Software Update hidden. `quick-settings
reset-layout` over IPC restores it. Software Update (Utilities; with Emaki Settings installed)
draws only while the NixOS configuration's scheduled update check found newer inputs
(`$XDG_STATE_HOME/emaki/nixos/check.json`), naming them; a press opens Emaki Settings on
Software Update ([nixos.md](../../developer/integration/nixos.md#the-scheduled-check)).

## Motion

The card has one height, the grid's, for the grid and every page. A surface resized every frame
costs a compositor configure round-trip and a second draw each time, which is what made the old
unfolding card stutter. So nothing resizes while anything moves. The card swells in from the
bar's corner and rises a little out of it on a spring (`motion::pop`, from `quick.open.scale`),
its modules arriving in a quick wave out from that corner, the Edit Controls capsule last. It goes
quicker than it came (`motion::CLOSE`), at the size it had, without the wave. A tile's page grows
out of the tile (`motion::zoom`): one glass sheet that starts as the tile's glass, its rect and
corners, and grows to fill the card, while the rest of the grid stays put, sinks back from the
tile (`quick.page.recede`) and fades. What's on the page comes in once the sheet has room for
it: the header, then the rows one after another, rising a little (`detail.rs`); going back,
they go first and the sheet shrinks into the tile empty. A page opened from the bar or over IPC
grows out of the card's bottom-right corner, by the bar; opened while the card is closed, it's
simply there. Every spring keeps its speed when it's turned around (amane's `Animation::to`), so
going back halfway through, or closing while the card is still opening, never stalls.

Every piece of frost fades with what's on it: the card's as it opens and closes, the grid's as it
sinks under a page, a module's as it arrives, a control's as it's added or removed in edit mode
(`glass_region::faded`, alpha on each blur shape). A page's sheet is the exception: its frost is
at full strength from the first frame, taking over from its tile's, so the card never shows
through half-frosted mid-zoom.

A compositor that can't fade frost (no niri-blur-shapes, `amane::blur_fades()` false) only shows
or hides it, and frost at full strength under modules still fading in reads as blurred patches
with nothing on them; there the card arrives whole, frost and modules in the same frame, closing
drops the frost in the first frame, and amane leaves out the frost of anything less than half
visible; the modules don't arrive in a wave there either, as their frost would pop in one by
one. The surface is resized only at rest: when
the grid gains or loses a row, and when edit mode begins or ends — never while a control is
held. A surface resized just before it hides
still has the compositor's answers to the resize on their way, and amane draws on those after
the hide, which niri rejects as a protocol error; closing never resizes.

Every control's hover layer fades (`motion::HOVER`), and a press shrinks a tile or disc to
`quick.press.scale` and springs it back (`motion::PRESS`): amane reports a press as the start of a
drag and not its release, so the pulse is timed rather than held. In edit mode the controls
spring between cells (`motion::animate`, keyed by control), and the one in hand draws over them
at `quick.drag.scale`, springing back to size when let go. The cursor a drag started with stays
until the drop (amane keeps the dragged target's cursor): a resize handle doesn't follow the
pointer, and a cursor flipping on every motion made niri redraw every output each time.

A drag is measured on one still layer over the whole surface (`customize::drag_layer`), not on
the control. amane measures a drag against the hit areas of the last frame drawn, and a control on
the move is drawn somewhere new each frame; a drag measured on it counted the pointer's motion
against where the control *was*, several times a frame with a mouse that reports faster than the
screen refreshes, so the control overshot and snapped back every frame. amane also keeps the
target being dragged by its position in the frame's hit areas, so the layer comes first on the
surface, before anything that changes under a drag. Buttons and rows keep their own presses
(amane hands a press to the innermost hit area that listens for a drag, and every button does, for
its press pulse), so only a press on an inert control or the canvas reaches the layer. The frost
behind a moving control is the control as drawn, scaled about its centre, the same shape its glass
edge is lit on.

## Frost

niri frosts a layer surface where the surface asks, or its whole geometry when it doesn't ask.
The Quick Settings surface asks with one rounded shape per piece of glass
(`LayerWindow::blur_shapes`, `glass_region.rs`): the grid's modules, the Edit Controls capsule,
the Customize window's sheet and the control in hand, and a page's sheet. The shapes go over niri-blur-shapes, a protocol of our niri
fork ([`effects/blur-shapes.md`](https://github.com/emaki-desktop/niri/blob/emaki/docs/fork/effects/blur-shapes.md); amane's side is [`graphics/blur.md`](https://github.com/emaki-desktop/amane/blob/emaki/docs/fork/graphics/blur.md)): each is a rounded rectangle in fractional surface coordinates with its own alpha, which
niri draws through its corner-rounding shader, half a pixel inside the module's own antialiased
edge, so no frost strays past a corner. While editing, the Customize window's sheet and the
control in hand are shapes of their own; the controls in the window's pane ask for none, which is
why they're drawn on a plain tint. Each is a `Material::Control` shape (a page's sheet is a
`Popover`, the Done capsule `Prominent`), which names niri's `glass` material and carries the
tint. When niri's config defines a `glass` material (`materials {}`, see niri's
[`effects/materials.md`](https://github.com/emaki-desktop/niri/blob/emaki/docs/fork/effects/materials.md)), niri draws the modules' tint and lit edge itself, from the real
backdrop and on the frost's own edge: the modules then leave out their tint fill and the grid
its rim pass (`Material::compositor_draws`), and a page's sheet its `glass::apply`. Without
the material the shell draws both as before. On a compositor without the protocol, amane
sends the shapes as an `ext-background-effect-v1` region instead: strips that step in at the
corners, rounded inwards. Its layer rule therefore carries no corner radius, blur or shadow, only
`xray false` (niri's blur shows the wallpaper alone otherwise); see
[layer-rules.md](../../developer/integration/layer-rules.md).

A module is the panels' tint, the panels' glass edge and a soft drop shadow (`tile::module`,
`quick.shadow`); vello draws the shadow analytically in the module's own pass, so sixteen of
them cost well under a millisecond. The card itself sits `quick.grid.margin` in from the
surface's edge, room for the shadows. The glass (`glass.wgsl`: the lit rim, the far catch and the
bloom; the panels' inner glow is left out, as across a whole module it read as a band inside the
edge) is a GPU pass whatever it covers, and a pass per module (`glass::apply`, as
the panels wear it) cost 13 ms more a frame on an integrated GPU, so the grid lights every
module in one pass over them all (`glass::rims`, up to thirty pieces a pass, their geometry in
the shader's values; amane's fork gives a shader 64 rows for that): +2 ms a frame, measured. A
page's sheet, one piece, wears `glass::apply` itself.

## IPC and the keyboard

`quick-settings [toggle [<page>] | open [<page>] | close | edit | customize | reset-layout]` opens
the card, a page of it on its own (as its status icon does; `toggle <page>` closes it again), or
edit mode from a script or a keybind ([ipc.md](../../developer/integration/ipc.md#quick-settings),
with the page names).

The card takes the keyboard while it's open (`Keyboard::Exclusive`: it's modal, as Control
Center is), so Escape works from the first moment however the card was opened, and the Wi‑Fi
password field types without a click first. A click outside closes it.

## Components

The parts are in `apps/shell/src/surfaces/quick_settings/`:

- **`tile.rs` (`QuickToggle`, `module`)**: GNOME's QuickToggle in the module shapes
  (`Style`): a `Card` (a glass capsule two, three or four cells wide) or a `Round` button (the disc
  alone in a glass circle a cell big, named by a tooltip). `module` is the glass piece every control sits in; `sheet` the page's, with the full
  glass edge; `capsule_button` the labelled capsules; `plain` builds modules on a flat tint for the window.
  - A round disc holding the glyph. It springs to the accent when the setting is on, like the
    `Switch` knob.
  - A title and an elided subtitle beside the disc.
  - Like the other controls, it shows a click before the service agrees, for up to 1.5 s.
  - `inert` draws it without any handler, for edit mode.
- **`grid.rs`**: the canvas's geometry, which is the shared grid's (`components::grid`, which the
  desktop widgets are laid out on too): a `Span` (cells across and down), a `Place`, a `Rect`,
  the `Canvas`, `Metrics` (cell and gap in pixels, a rect's `Slot`, the nearest place to a
  point) and `Occupancy` (which control each cell holds; the first free place for a span, what a
  rect covers, the rows in use). What's Quick Settings' own here is the control sizes
  (`Span::S` … `FULL`) and the canvas and metrics from its tokens. amane has no grid widget, on
  purpose.
- **`controls.rs` (`ControlId`)**: every control, the sizes it comes in, and how to build it.
- **`layout.rs` (`Layout`)**: the shared grid's layout of `ControlId`s (which is its
  `GridKey`); `moved`, `placed_at` (a control from the window too) and `resized` give the layout
  a drag would leave (swap or push), `show` the first free cells. Its own are the default
  order, the file, and that every control has an entry.
- **`edit.rs`**: edit mode, its drag a shared `grid::drag::Session` (the base layout, its
  preview and what the pointer holds).
  Here too: the resize ghost, the badge and handle, the ghost discs, undo and reset.
- **`customize/`**: the Customize window: `mod.rs` the sheet, the sidebar, the pane (every
  control on a plain tint, named), the footer, the still drag layer and the control in hand;
  `catalog.rs` the categories, what each shows and the search (pure, tested).
- **`glass_region`** (the kit's, `crates/kit/src/look/glass_region.rs`): the frost behind a piece of glass as a blur shape, and its transforms.
- **`tiles/media.rs`**: the square Now Playing module; its art is `services/media_art.rs`'.
- **`slider_card.rs`**: the Display and Sound cards: a title over each thin slider, a small
  glyph before the track and a large faint one after it, and a right click (`Card::on_secondary`)
  for a card with a page. The track's fill is the panel's `fg` (white in dark mode),
  a documented departure from the one-accent rule (§11.2). (A microphone slider
  would need a taller module; the grid's cells are one height, so it's on the Sound page.)
- **`detail.rs` (`DetailPage`)**: a page, as tall as the grid.
  - A back chevron (`TopAppBar::back`), the title and subtitle (elided to the room the
    chevron and switch leave), and an optional `Switch`.
  - A hairline, then a body built for the room it has (`List` rows are the page's width,
    `list::with_row_width`), at least `MIN_PAGE_ROWS` tall.
  - Pinned at the bottom: an optional **Show in Status Bar** switch (`DetailPage::status_bar`, for the
    bar's status icon) and an optional link to a settings app.
- **`system_row.rs`, `tiles/`, `pages.rs`**: the controls and pages themselves. `tiles/` has one
  file per setting, holding its toggle and its page.

It reuses the shared components:

- **`SearchField`**, **`Flow`**, **`List`** (its `leading_tile` and `bare` rows),
  **`PushButton`**, **`ScrollView`** and **`Kinetic`** in the Customize window, and
  **`text_edit::Field`** for its search.
- **`Slider`**: the thin slider, with `fill` and `thickness` for the cards and `inert` for
  edit mode; `Slider::large` (a control-tall track with the glyph as its knob, which can mute)
  on the pages.
- **`buttons::optimistic` / `expect`**: the click shown before the service agrees, for the
  mute and play/pause glyphs as `Pending` is for the tiles.
- **`dnd::choices`**: the timed choices, which the Do Not Disturb menu uses too.

### SearchField and text_edit

The kit's `search_field` and `text_edit`, which the launcher's field shares.

`SearchField::new(id, &field, focused)` is a `surface.raised` capsule a control tall: a magnifier,
the query (or a `.placeholder(…)`, "Search" by default) in `sys.type.label` with its caret (accent,
while `focused`) and selection (a `state.selected` wash), and a clear (×) button while there's a
query and an `.on_clear(…)`. `.width(…)` sets its width. It draws the text itself rather than
focusing a `TextInput`, so every key reaches the surface's `on_key`.

The text is a `text_edit::Field`: the characters, the caret and the selection's other end, with
`insert`, `backspace`, `delete_selection`, `left`/`right`/`move_to` (extending the selection with
Shift) and `select_all`, and `handle(key, modifiers)`, which applies the keys a one-line field
takes — characters, Space, Backspace, Left/Right/Home/End with or without Shift, Ctrl+A/C/X/V
through wl-clipboard (`text_edit::copy`, `paste`) — and says whether the key was the field's.
Line breaks never go in. It's pure and tested; the launcher types into one with its own key
routing, the Customize window through `handle`.

Tokens: `quick.grid.{cell,cols,rows,gap,margin}`, `quick.tile.{pad,gap,disc}`,
`quick.card.radius`, `quick.media.{art,art-radius,play}`, `quick.slider.track`, `quick.shadow`,
`quick.type.{title,caption,line}`, `quick.page.recede`, `quick.open.scale`, `quick.press.scale`
and `quick.drag.scale`. The motion is `quick.open.{lift,cascade,cascade-scale,cascade-rise}`,
`quick.page.{tile-fade,grid-fade}`, `quick.page.reveal.*` and `quick.drag.band`; a page is at least
`quick.page.min-rows` tall, and a placeholder is `quick.placeholder.opacity` faint. The Customize
window is `quick.customize.*`. Everything else is `sys.*`.

## The alert

The Power Off page's Restart…, Power Off… and Log Out… ask first in the shell's alert
(`apps/shell/src/surfaces/alert/mod.rs`): an alert in the middle of the screen, a kit
[`Dialog`](../../developer/kit/components.md#dialog) on thick glass. `alert::show(dialog)` puts it on the
`emaki.alert` surface, as big as its card, which frosts itself and holds the keyboard until
answered; its layer rule rounds it (`dialog.radius`) and gives it niri's shadow (`0 18 48`). In
debug builds the `alert-test` IPC call shows a sample ([ipc.md](../../developer/integration/ipc.md#alert-test)).

## Backends

What amane has no service for (logind's session actions, night light, dark style, airplane mode,
the keyboard light, wired/VPN/hotspot, privacy, background apps, Keep Awake, Tailscale) is a shell
service in `apps/shell/src/services/`; see [services.md](../../developer/integration/services.md), which also has the
Tailscale operator grant, why night light owns `wlsunset`, and how Keep Awake and locking behave.

## Related

- The bar and its panels: [status-bar.md](status-bar.md).
- Its frosted surface's niri layer rule: [layer-rules.md](../../developer/integration/layer-rules.md).
- The desktop widgets, on the same grid: [widgets.md](widgets.md).
- The shared components it's built from: [components.md](../../developer/kit/components.md) and
  [controls.md](../../developer/kit/controls.md).
