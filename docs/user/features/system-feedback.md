# System Feedback

What the shell tells you as the system changes by itself: a device plugged in, a Bluetooth
headset connecting, the charger, a drive to eject, a network lost, a screenshot taken, the
computer running hot (`apps/shell/src/services/feedback/`). Each kind can be turned off in
Emaki Settings → **Notifications** → **System Feedback**.

## One rule: a capsule or a notification

- **A capsule** for a change that needs nothing from you. It's the
  [on-screen display](osd.md)'s capsule, with a glyph, the device's name and a quieter detail
  ("AirPods Pro · Connected · 80 %"). It fades after 2.2 s, makes no sound (unless you turn on
  connection sounds), never stays in the notification center, and shows through Do Not
  Disturb, as the volume does.
- **A notification** for something you might act on, or should know about later: a drive to
  open or eject, a battery running low, a failure. Ones that are only news (a drive ready, a
  screenshot) are sent `transient`, so they leave once their toast ends. Warnings and failures
  stay in the [notification center](notifications.md). They follow Do Not Disturb like any
  other, and critical ones show through it.

The shell's own notifications are sent under their own names (`emaki-bluetooth`, `emaki-usb`,
`emaki-drives`, `emaki-power`, `emaki-network`, `emaki-displays`, `emaki-screenshots`,
`emaki-thermals`). The center groups them by those names. Settings lists them under System
Feedback, not with the apps.

## What's told of

| Kind | Capsule | Notification |
|---|---|---|
| **Bluetooth** | a device connecting ("Connected · 80 %") or disconnecting | a device or a phone (KDE Connect) at 20 % and again at 10 % (critical), once each per connection |
| **USB devices** | a device plugged in or out, by its own name ("MX Keys"), with a glyph for its kind (keyboard, mouse, headset, camera, printer, phone, controller); several at once, like a dock's, are one capsule ("4 USB Devices") | — |
| **Drives** | "Safe to Remove" once ejected | ready, with its size, **Open** and **Eject**; "Ejecting…"; "Couldn't Eject" when a program has a file open, with **Try Again**; "Wasn't Ejected" when pulled out while mounted; "Couldn't Open" when it can't be mounted |
| **Power** | the charger in ("Charging · 100 W", with what a USB‑C charger agreed to give), out ("On Battery · 3 h 20 min left"), and charging stopping at full or the charge limit | "Battery Low" at UPower's low level, with **Turn On Power Saver**; "Battery Very Low" (critical) past it; "Slow Charger" for a USB‑C charger under 30 W, naming the port; "Plugged In, Not Charging" when the battery still runs down 90 s after plugging in |
| **Network** | an Ethernet cable connecting or disconnecting | "Couldn't Join" a Wi‑Fi network or "Couldn't Connect" a VPN, with why and **Try Again** (unless Quick Settings is open, whose row already says so); a VPN that went down while the network stayed up and you didn't turn it off, with **Reconnect**; "Sign In to …" for a captive portal, which opens the sign-in page |
| **Displays** | a display plugged in, with its name and mode ("Dell U2723QE · 3840×2160"), or out; the laptop's own panel is left out | — |
| **Screenshots** | "Screenshot Copied" for one only copied | "Screenshot Saved" with the picture, **Copy**, **Open** and **Show in Files**; clicking it opens it |
| **Temperature** | — | "Running Hot" when the processor or graphics stays at 95 °C or more, or the SSD near its own limit, for two readings running (at most once every 15 minutes), with **Show Temperatures**; "Very Hot" (critical) at the limit |

Nothing is told in the first 5 s after the shell starts or the computer wakes, when devices come
back one after another. A battery warning is sent even then.

While Quick Settings is open, capsules for what it shows are dropped: its rows are the feedback.
A capsule never cuts short a level being set: it waits until the volume or brightness capsule has
gone.

## Settings

Emaki Settings → **Notifications**:

- **System Feedback**: a menu for each kind, **Show** or **Off**. Off silences both its capsules
  and its notifications.
- **Open drives as they're plugged in** (on): a drive is mounted as it appears, so its
  notification can open it. Off, it's left alone until you open it.
- **Play a sound as devices connect** (off): the sound theme's `device-added` and
  `device-removed`, at the notification sound's volume.
- **Indicator Light**: only on a laptop with one (a ThinkPad's lid logo dot or ThinkVantage key).
  - **Pulse for**: **All Notifications**, **Critical Only** or **Off** (the default). The light
    pulses (1 s on, 2 s off) while a notification hasn't been seen, and stops once the center is
    opened. During Do Not Disturb only a critical one makes it pulse.
  - **Light**: which one, when there are both.
  - **Stay lit during Do Not Disturb**: a steady light while Do Not Disturb is on and the screen
    isn't locked.

The light is set through logind, as the microphone's mute light is, so no root is needed.

## What it needs

- **Drives** need UDisks2. On NixOS, `programs.emaki.removableDrives.enable = true` in the
  system module turns on `services.udisks2`. Its polkit rules let the active session mount and
  eject removable drives without a password.
- **USB devices** and **the charger's power** come from the kernel's device events (a netlink
  socket, no udev needed), and the names from sysfs. A device on one of the laptop's own fixed
  ports (its camera, fingerprint reader, Bluetooth radio) is left out. A hub is left out too.
- **The charger's power** comes from the USB‑C ports' UCSI power supplies
  (`/sys/class/power_supply/ucsi-source-psy-*`). On a laptop without them, the capsule shows the
  battery's level.
- **A captive portal** is found by NetworkManager's connectivity check, which must be on (a
  `[connectivity]` URI in its config). Without it, there's no sign-in notification.
- **Screenshots** come from niri's `ScreenshotCaptured` event.
- **Temperatures** come from `/sys/class/hwmon`, as Settings' About page reads them.

## How it works

Each kind is a small service in `services/feedback/` that waits on its source and compares what
it reads with the reading before. Nothing polls except the temperature, which is read every 20 s
and only while the session is unlocked and in use.

The feedback services hand capsules to a queue (`feedback::echo`), which the on-screen display
takes from in turn. Notifications go through amane's `Notify`, which, with the shell as the
notification server, puts them straight into the list and runs a button's closure when it's
pressed.

Temperature can't follow the kernel's trip points: on AMD and recent Intel laptops the firmware
slows the processor by itself, well before any trip the kernel knows (a ThinkPad T14's only
trips are at 113 °C). So "Running Hot" reads the sensors and says the computer *may* slow down,
not that it has.

## Not yet

- The indicator light can be left lit if the shell stops while it's on; it goes dark the next
  time the shell starts and finds nothing unseen.
- Tailscale going down isn't told of yet, only NetworkManager's VPNs.
- A wrong Wi‑Fi password has no **Try Again**, since only Quick Settings asks for a new one.
- Thunderbolt devices that need approving (boltd), firmware and NixOS updates, and the shell
  recovering from a crash aren't told of yet.
