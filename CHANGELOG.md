# Changelog

All notable changes to emaki (amane-shell until 2026-11) are recorded here.

The format is [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/), and the project
follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

<!-- one line per change, newest last -->

- **BREAKING: amane-shell is emaki.** The amane library underneath keeps its name, and its
  own `AMANE_*` variables (`AMANE_DEBUG`, `AMANE_FRAMES`, `AMANE_VELLO_*`, …). Renamed:
  - binaries: `amane-shell` is `emaki`; `amane-settings`, `amane-messages`, `amane-mirror`,
    `amane-shader-lab`, `amane-drop`, `amane-assistant`, `amane-pinentry` and
    `amane-train-watch` are `emaki-…`
  - packages and crates: `amane-shell` is `emaki`, `amane-kit` is `emaki-kit`, and the other
    `amane-*` crates are `emaki-*` (`emaki_kit::` in Rust); the flake's package is
    `packages.<system>.emaki` (and `default`)
  - app ids and desktop files: `amane-settings.desktop` and the others are `emaki-*.desktop`,
    the icons `emaki-settings` and `emaki-messages`
  - environment variables the shell reads: `AMANE_THEME`, `AMANE_DENSITY`, `AMANE_LOG`,
    `AMANE_RBW`, `AMANE_ASSISTANT_MODELS` and the rest are `EMAKI_*`
  - sockets: `$XDG_RUNTIME_DIR/amane.sock`, `amane-drop.sock`, `amane-assistant.sock` and
    `amane-drop/` are `emaki.sock`, `emaki-drop.sock`, `emaki-assistant.sock` and `emaki-drop/`
  - D-Bus: `org.amane.StatusBar1` at `/org/amane/StatusBar` is `org.emaki.StatusBar1` at
    `/org/emaki/StatusBar`, with no alias; item ids `org.amane.*` are `org.emaki.*`
  - layer namespaces: `amane.dock`, `amane.status-bar`, `amane.launcher` and the others are
    `emaki.*`; the niri config's markers are `// >>> emaki layer-rules` and
    `// <<< emaki layer-rules`
  - folders: `~/.config/amane-shell`, `~/.local/state/amane-shell` and `~/.cache/amane-shell`
    are `…/emaki`, the `amane-*` ones `emaki-*`, `~/.cache/amane/messages` is
    `~/.cache/emaki/phone/messages`, `~/.local/share/amane-assistant` is
    `~/.local/share/emaki-assistant`
  - the settings app: Shell Settings is **Emaki Settings**
  - NixOS: the flake input `amane-shell` is `emaki`, the options `amane.*` are `emaki.*`, the
    flake output `amaneSettings` is `emakiSettings`, `amane-nixos-activate` and its polkit
    action `org.amane.nixos-activate` are `emaki-nixos-activate` and `org.emaki.nixos-activate`,
    the units `amane-shell.service`, `amane-drop.service` and `amane-update-check` are
    `emaki.service`, `emaki-drop.service` and `emaki-update-check`
- Launcher: it opens from a drop that stretches into the pill, the mode buttons budding off its
  end one after another and the text coming in last; it closes the other way, quicker. Opened
  on a mode, it holds as the pill a moment before growing into the panel. Results and the app
  grid rise in row by row when they first show. The mode buttons lift on a spring, the mode's
  glyph crossfades into the back button, the empty field's words crossfade, the clear button
  swells in and the highlight glides. With Reduce Motion on, each part only fades.

### Security

- Emaki Settings' activation helper ships in this flake (`nixosModules.default`,
  `programs.emaki.nixosActivate.enable`): it activates only a valid, registered NixOS system's
  store path, logs each activation to the journal, and asks for the password every time
  (`auth_admin`, no longer `auth_admin_keep`). **Remove the helper from `modules/managed.nix`**
  when you enable it.
- The polkit dialog shows the command line that will run, store path included; only polkitd may
  call the agent, and an answer with a line break is refused.
- A new Bitwarden server, asked for over the IPC socket, is confirmed in the shell's own dialog
  (`passwords configure` answers `confirm`), and the unlock dialog names the server.
- The shell, emaki-pinentry and the assistant's daemon trust each other only from the same
  build, and rbw-agent only from the rbw the shell runs; being in the Nix store no longer counts.
  After a rebuild, until the shell restarts, unlocking uses pinentry-gnome3.
- The programs emaki runs (nix, git, pkexec, curl, secret-tool, wl-copy, xdg-open, adb, …) come
  from the system or user profile, or pinned store paths, never from `PATH`.
- Drop's passwords no longer go to a plain-text file without asking: Emaki Settings asks when no
  keyring answers. Secrets are wiped from memory after use; Settings and Drop turn off their core
  dumps, and Settings and the assistant keep other processes from reading their memory.
- Limits on what other programs send: images (file size, pixels, no outside files in an SVG),
  notification images and text, a rate per sender, the IPC socket (2 s, 64 KiB, the same user
  only, read off the UI thread), the status bar API (8 items and 8 activities a program, 64 KiB
  calls, `org.emaki.*` ids for emaki's own programs), tray icons, drops and pastes, Phone
  Screen's packets and clipboard, base64 thumbnails, and album art (public addresses only, no
  redirects, 10 MB).
- A dropped file's name goes to the app as one argument, never through `sh -c`; `emaki-drop
  run` on an upload or phone tile holds the files instead of sending them.
- Phone Screen takes only this user's connections on its localhost port; its decoding threads
  are confined with Landlock. The assistant's daemon takes connections only from the shell, runs
  with no network, a system-call filter and no new privileges, and confines its workers with
  Landlock.
- Nix input is quoted: a package's attribute path and the configuration's host can't splice
  code into an expression; options end before the user's search text.
- Runtime files never fall back to a shared `/tmp`; the clipboard history and the shell's state
  are written 0600, and no write follows a link left in its place.
- CI checks the dependencies (`cargo deny`, `deny.toml`), fuzzes the parsers in `fuzz/`, pins
  actions by commit and gives the token read access only.

### Added

- A home-manager module (`homeManagerModules.default`, `programs.emaki`) with user units for the
  shell, Drop and the assistant's socket-activated, sandboxed daemon.

- The move from amane-shell, once, on each program's first start: the old folders move to the
  new names, the ids the settings keep (`org.amane.*`, `amane-*.desktop`) are renamed, and
  `~/.config/emaki/.migrated-from-amane` marks it done; keyring passwords under the old names
  are still found. `scripts/migrate-from-amane` does the moves by hand and renames the niri
  config's markers and sockets ([troubleshooting](docs/user/troubleshooting.md#migrating-from-amane-shell)).
- Emaki Settings reads back a `displays.kdl` that Shell Settings wrote.
- Status bar: an item dragged off the bar leaves an undo button for 5 s; the items it passes
  slide aside, and one about to be hidden shows a red badge.
- Status bar: an item stays lit while the menu it opened is open (keyboard layout, Do Not
  Disturb, tray apps, », privacy, programs' items).
- Status bar: scrolling over the keyboard layout goes round the layouts.
- Status bar: fuller tooltips: the battery's time left or to full, the sound output's name,
  the Wi‑Fi signal, Bluetooth devices' charge, how many notifications are new, and when Do Not
  Disturb ends.
- Emaki Settings' Weather page says when the place can't be found or Open-Meteo can't be
  reached (`~/.cache/emaki/weather-status.json`); its Items list has programs' items too.
- Widgets: Edit Widgets has a size picture under each kind in the gallery, a gallery that
  scrolls, the keyboard (Tab, the arrows, Shift and an arrow to resize, Delete, Enter, Ctrl+Z), a
  card's menu on a click, and an Undo Remove that goes back one removal at a time.
- Widgets: a graph of the last 15 minutes (CPU, memory, temperature, network), °F, bits per
  second, the clock's AM/PM and **Show Date**; sensors and interfaces by names a person knows
  ("CPU Tctl", "Wi-Fi · wlp194s0").
- Widgets: `widgets list --json`, `kinds`, `set`, `resize`, `move` and `undo` over IPC; Emaki
  Settings' Desktop page lists each monitor's widgets with Remove and Remove All.
- Status bar: items slide aside as others come, go or change width (a chip turning compact, the
  date dropping for room), new ones fade in, a dropped item settles into its place, and an
  activity's expanded view fades in as the menus do.
- Emaki Settings: the Status Bar's position and size are pictures of the bar; search finds the
  weather's place, the tooltip's date, scrolling over the workspaces and more words.
- Launcher: **Wi‑Fi Networks** and **Bluetooth Devices** open a list to join a network or
  connect or pair a device. A network's password is typed hidden in the field and never kept.
  From the pill, saved networks and paired devices are found by name.
- System feedback (`services/feedback/`, [system-feedback.md](docs/user/features/system-feedback.md)):
  a capsule as a Bluetooth or USB device, an Ethernet cable, a display or the charger
  (with a USB‑C charger's watts) comes or goes; notifications for a drive to open or eject, low
  and critical laptop battery, a slow USB‑C charger, a failed Wi‑Fi or VPN connection, a VPN
  dropped, a captive portal, a screenshot saved (Copy, Open, Show in Files) and the computer
  running hot. Each kind can be turned off in Emaki Settings → Notifications, which also has
  connection sounds and, on a ThinkPad, a lid logo or ThinkVantage light that pulses for
  unseen notifications. The NixOS module's `programs.emaki.removableDrives.enable` turns on
  UDisks2 for the drives.

### Fixed

- Wi‑Fi: a network joined with a wrong password asks for it again, in Quick Settings and the
  launcher, and the new password replaces the one saved. Before, the network had to be
  forgotten first.
- Status bar: a full battery on AC no longer reads "charging".
- Status bar: a link that is neither Wi‑Fi nor a cable no longer shows a Wi‑Fi glyph.
- Status bar: when Open-Meteo can't be reached, the weather keeps its last reading, dimmed,
  and tries again after 5 minutes, rather than going.
- Status bar: Night Light, VPN and power mode open their own Quick Settings pages.
- Status bar: the weather opens the notification center, like the clock beside it.
- Status bar: a tray app wanting attention shows an accent dot rather than a ring too big for
  the Compact bar.
- Status bar: the camera and microphone colours are the palette's; Comfortable spaces its items
  wider; the analog clock is drawn finer at 16 px.
- Status bar: the hover tile fills the Compact bar's height rather than being shorter than its
  glyph; items with text all pad and space their parts alike, programs' titles are in the
  primary label colour as the clock's are, and text caps grow at Comfortable.
- Emaki Settings: searching "Allow Activities" or "Allow in the Status Bar" opens their page
  rather than lighting an unrelated row.
- Emaki Settings: moving a bar item with its arrows no longer loses where programs' items were.
- Widgets: a `widgets.json` that can't be read is kept as `widgets.json.bad` rather than
  overwritten; one bad widget in it no longer loses the rest.
- Widgets: an interface that comes up (a VPN, a tether) no longer spikes the network graph; one
  sensor that doesn't answer no longer blanks every temperature; two drives' sensors of one name
  are told apart; the default route follows the lowest metric, and IPv6-only networks have one.
- Widgets: Large Memory's swap no longer draws over its graph; a sensor, interface or disk a
  widget was set to that's gone shows the automatic one and says so.
- Widgets: Escape during a drag no longer lets the next motion pick up another card, and Undo
  Remove no longer loses what was placed since.

## [0.1.0] - 2026-11-08

The first tagged release: the desktop as it runs today.

### Added

- **Status bar**: one bar a monitor, top or bottom, in three sizes (Compact, Regular,
  Comfortable) and four styles (Frosted, Glass, Solid, Clear), with the workspace's shape,
  status icons, the tray, programs' items, Activities, a clock and weather; auto-hide modes.
- **Dock**: every workspace's windows and pinned apps, rising from the bottom edge and with
  niri's overview; drag to reorder or move, drop files to open them; a reveal strip when hidden.
- **Launcher**: a search pill with Applications, Files, Actions, a clipboard history and
  Passwords (Bitwarden through rbw), and the on-device **assistant** (Cactus: Needle 3 for
  commands, Whistle for speech) in the socket-activated `amane-assistant` daemon.
- **Notifications**: the shell is the notification daemon: toasts, a notification center with a
  month calendar and a history kept across restarts, per-app rules and Do Not Disturb.
- **Quick Settings**: a grid of glass modules with a page per control (sound, Wi-Fi, VPN,
  Bluetooth with pairing, power, Night Light, keyboard, session, Tailscale, Keep Awake, the
  phone) and Edit Controls.
- **Desktop widgets** on niri's desktop layer: a stroke-drawn clock, batteries and the system's
  readings, on a shared grid with Edit Widgets.
- **On-screen display** for volume, brightness, the keyboard light, lock keys and switches.
- **Drop** (`amane-drop`): a drop shelf in the bar: folders, apps, the phone and uploads
  (0x0.st, Imgur, SFTP, S3), with a Drop Bar to hold files.
- **Shell Settings** (`amane-settings`): appearance, status bar, dock, Quick Settings,
  notifications, launcher, passwords, Drop, **Displays** (resolution, refresh, scale, rotation,
  arrangement, HDR, DDC/CI brightness, Night Light) and the **System** pages that manage the
  machine's NixOS configuration (General, Apps, Services, Power, Storage, Updates) through
  polkit; the shell is the session's polkit agent.
- **Messages** (`amane-messages`) and **Phone Screen** (`amane-mirror`): KDE Connect texts,
  attachments and remote input, and the phone's screen through scrcpy.
- **Shader Lab** (`amane-shader-lab`): shader effects tried live, with saved looks the kit
  can put on any rectangle.
- **amane-kit**: the shell's look as a library: the Penpot design tokens, theme (mode, accent,
  density, text scale, glass), controls, containers, motion, an app window, settings files.
- **Status apps**: `org.amane.StatusBar1` on the session bus, with the `amane-status` crate,
  so any program can put an item and Activities in the bar.
- The shell's IPC socket (`$XDG_RUNTIME_DIR/amane.sock`), niri layer rules generated from the
  tokens, and a Nix flake that builds every program with its runtime wiring.

[Unreleased]: https://github.com/emaki-desktop/emaki/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/emaki-desktop/emaki/releases/tag/v0.1.0
