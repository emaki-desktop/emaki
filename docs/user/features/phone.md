# Phone (KDE Connect)

The phone through KDE Connect, drawn as part of the shell. `kdeconnectd` runs on
its own, and nothing of KDE's ever shows: no Plasma, no Kirigami, no
`kdeconnect-sms`. Beside it, the phone's **screen, sound and camera** come through
scrcpy's server over adb (see [Phone Screen](#phone-screen-scrcpy)). There are
four surfaces:

- **Quick Settings' Phone page:** pairing, battery, Ring, Send Files…, the
  phone's screen and camera, the clipboard, and the phone's remote for this
  computer.
- **The bar's phone icon:** the phone's charge, beside the battery.
- **Messages** (`emaki-messages`): the phone's text messages and Phone Input.
- **Phone Screen** (`emaki-mirror`): the phone's screen and sound, driven from
  here, and its apps in windows of their own.

All three read the fork's `KdeConnect` and `PhoneMessages` services
([emaki-phone](../../../crates/phone/README.md)).
They come from the Quickshell version's "KDE Connect, made native".

The phone's **notifications** and **media** need no code. The shell is the
notification server, so they show as toasts and in the center. Now Playing reads
every MPRIS player, and the phone's playback is one of them. Files the phone
shares land in `~/Downloads`, and KDE Connect sends its own notification.

## Setting up

- `kdeconnect` on in Emaki Settings → Services (`programs.kdeconnect`) installs
  it and opens its ports, UDP and TCP 1714–1764.
- The daemon starts at login from its own autostart entry.
- On the phone, install KDE Connect (F-Droid or Play), on the same network, and
  allow the plugins you want: SMS, Contacts, Remote Input and so on.

## Quick Settings: Phone

The **Phone** control waits in Edit Controls' Suggestions (it's under
Connectivity). It shows "<phone> · 54%", "Not Connected", "No Phone" or "Off",
and it's lit while a phone is connected. The bar's phone icon opens the page
too.

The page:

- **My Devices:** each paired phone with its charge, or "Not Connected". A click
  or the ⓘ opens its own page.
- **Other Devices:** the phones on the network that could pair; a click asks
  one. While it waits, the row shows the code both screens show, and a click
  stops waiting. Opening the page announces this computer again, so phones
  answer at once.
- **A phone asking to pair** appears under the list with its code and
  **Decline** / **Pair**. When the page isn't showing, it asks in the shell's
  alert instead (`surfaces/alert/phone.rs`, from `services/phone/pairing.rs`).
  Any device on the network can ask, under any name it likes: with no code to
  compare, the alert says so, and Return declines (**Pair** is a click).
- Without the daemon, the page says so and offers **Turn On**.

A paired phone's page, as far as the phone offers each:

| Row | Does |
|---|---|
| Battery | the charge, and "charging" |
| Ring | rings the phone, even on silent |
| Send Files… | the portal's file chooser, then each file to the phone |
| Messages, Phone Input | opens `emaki-messages` on that phone and view (when it's installed) |
| Clipboard: Share Automatically, Send Clipboard to Phone | auto-sync both ways, or this computer's clipboard now |
| Phone Remote: Control This Computer | adds Lock Screen, Suspend, Lock & Suspend and Screenshot to the phone's Run Command |
| Forget This Phone… | asks, then unpairs |

An action that fails shows why in red for 10 seconds.

## The bar's phone icon

`Indicator::Phone` (`phone` in `status-bar.json` and the IPC). It shows while a
paired phone is connected and has sent its charge. It's accent while charging
and red when low. The tooltip has the name and the charge. A phone running low
warns at 20 % and 10 % like a Bluetooth device does (`services/feedback/bluetooth.rs`,
[system-feedback.md](system-feedback.md)), but not while it charges.

## Messages

`emaki-messages` is an app beside Emaki Settings (`apps/messages`), with its
own window rule that frosts behind it.

- **The inbox** (the sidebar): the phone's conversations, newest first.
  - Each row has the contact's photo from the phone's contacts, or their
    initials in a color kept for the name (grey for a bare number). A group
    has the group mark and is named for its people: "Kari, Per and Ola".
  - The row shows the name, the latest message ("You: …" for this side; "Photo"
    or "Video" for an attachment alone) and when it was sent (the time today,
    "Yesterday", a weekday, then a date).
  - An unread one is bold, with the accent's dot, and the header counts them.
    KDE Connect can't mark a message read on the phone, so opening a
    conversation here clears it on this computer
    (`~/.cache/emaki/messages/read.json`); a newer message brings it
    back.
  - The search field (Ctrl+F) filters by name, number or message.
  - ⟳ asks the phone again, since a sleeping phone sends nothing until asked.
    After 20 s with nothing, it says the phone didn't answer and offers Try
    Again.
  - ✎ (Ctrl+N) starts a **new message**: the To field suggests contacts by
    name or part of a number (Enter takes the first) or takes a typed number.
    Picking someone you already have a conversation with opens it. Otherwise
    the message starts a conversation that shows at once and becomes the
    phone's thread when it sends it back.
- **A conversation:** bubbles drawn after Messages' (`apps/messages/src/bubble.rs`).
  - Grey on the left; on the right, a gradient from the accent lightened to the accent.
  - Each bubble is a pill, mirrored between the two sides, sized to its text up to 70 % of the
    width. Its size is measured once per message and width.
  - The last bubble of a run has the curled tail at its bottom corner.
  - The outline is one vector path, made for each size, that amane fills, with the gradient, on
    the quad pipeline.
  - One to three emoji alone are drawn large, without a bubble. Emoji are in
    color everywhere.
  - In a group, the sender's name is over each run they wrote, and their small
    avatar beside its last bubble.
  - Photos and videos show as their thumbnails (a play mark on a video). Other
    files are a chip. A click fetches the whole file from the phone and opens
    it.
  - Each link in a message gets a chip under it that opens it.
  - A right click on a message offers Copy, Copy Code (a 4–8 digit code in a
    message about a code), Open and Copy Link, the attachments, and on a failed
    one Send Again and Delete.

  Each day is named over its messages, and the time sits under each run (one side, under 10
  minutes apart). It opens on the newest messages, and follows new ones down while
  the newest is in view; scrolled up, a **New Message** pill says one came.
  **Load Earlier Messages** asks for 50 more, and what's shown stays put as they
  come in above. A reply shows at once, dimmed with "Sending…", until the phone
  sends it back. One that isn't back in 2 minutes, or that the phone says
  failed, is red, "Not Delivered · Send Again".
- **The reply field** grows to five lines. Enter sends, and Shift+Enter starts a
  new line. 📎 attaches files, as does dropping them on the conversation. They
  show as chips over the field, marked when they're over 1 MB, which carriers
  may refuse. Each conversation keeps its half-written reply while another is
  shown.
- **Show on Phone** in a conversation's header opens the phone's own messaging
  app in a window (`emaki-mirror --app`), once its screen is set up and its
  apps are listed.
- **Phone Input:** a trackpad (drag moves the phone's pointer, a tap clicks, a
  right click long-presses, scrolling scrolls) with Left/Middle/Right buttons.
  Keys typed while it shows go to the field focused on the phone; the hint says
  whether one is focused. Ctrl and Super stay this computer's shortcuts.

Alt+Up and Alt+Down (or Ctrl+Tab and Ctrl+Shift+Tab) move through the
conversations. Escape clears the search, or leaves a new message.

`emaki-messages --device <id> --view messages|input|new` opens on a phone and a
view, and `--thread-of <name or number>` on that person's conversation once the
phone has sent it. Launched again while it runs, it switches to them. Without
`--device`, it picks the first connected phone that has messages.

A text message's notification (from Samsung's or Google's Messages) gets
**Open in Messages**, which opens that conversation here; it sits beside Open
on Phone.

Contact names and photos come from the vCards the phone's Contacts plugin
syncs. Without it, conversations show numbers.

Its sizes are the `messages.*` tokens, drawn in Penpot on "Messages — Components" and
"Messages":
- **The layout:** the sidebar's width, the avatars (36 in the rows and the header, 28 beside a
  group's runs, 32 in the suggestions), the unread dot, the badge and the stamp's inset.
- **The bubble:** its text size, padding, one-line height, radius, tail and narrowest width (the
  SVG's measures at 15 px), and the widest it gets (`bubble.max-width`, 0.7 of the pane).
- **The reply field and pills:** the reply field's five lines and its corner cap, and the pills'
  height.

They're fixed pixels; Messages doesn't follow the density. The bubble's text follows the text
scale, while its outline stays put. The received bubble's grey is `sys.color.bubble.received`.

## Phone Screen (scrcpy)

KDE Connect can't show the phone's screen, play its sound or use its camera.
scrcpy's server can, and `emaki-mirror` is a client for it drawn by amane. The
two work together:

- **KDE Connect finds the phone.** Its address (`PhoneDevice::addresses`) picks
  the adb device, and adb is asked to stay connected to it. Its name and
  battery are KDE Connect's.
- **The clipboard stays KDE Connect's.** While its Share Automatically is on,
  scrcpy's own clipboard sync is off, so a copy isn't echoed back. Ctrl+V in
  the screen still pastes this computer's clipboard into the phone.
- **Notifications stay KDE Connect's.** A phone notification gets **Open on
  Phone**, which opens its app (`services/phone/notifications.rs`).
- **Phone Input** stays for when adb isn't there. Messages offers **Show
  Screen** beside it when it is.

### Setting it up

1. On the phone, turn on Developer options, then **Wireless debugging**.
2. Phone page → the phone → **Set Up Screen Sharing…** shows a QR code. On the
   phone: Wireless debugging → **Pair device with QR code**, and scan it. The
   pairing code works too ("Pair device with pairing code"; type it under the
   QR code), and so does a USB cable with USB debugging on. The code goes to
   `adb pair` on its standard input, never as an argument other programs see.
3. The row becomes **Screen**. From then on the screen opens whenever Wireless
   debugging is on; Android turns it off on a reboot or another network.

The fork's `Adb` service finds Wireless debugging over mDNS, which needs UDP
5353 open (nixos-config `modules/desktop.nix`). The phones set up are kept in
`~/.config/emaki/phone-screen.json`.

### The screen

`emaki-mirror --device <id>` is a floating window shaped like the phone (its
niri rule is generated with the others).

- A press and drag is a finger, the right button is **Back**, the middle one
  **Home**, and the wheel or touchpad scrolls where the pointer is.
- Text typed is typed as text, so any layout and accent arrives as meant. Other
  keys are Android's keycodes, with Ctrl and Alt as shortcuts on the phone.
- The bar under it: volume, rotate, **Back**, **Home**, **Recents**, the
  notification shade, and the phone's own screen off while it's mirrored.
- Files dropped on it go to the phone's Downloads, and APKs are installed.
- The phone's sound plays here as the stream "Phone" (Android 11 and later),
  and its speaker goes quiet meanwhile.
- H.264 is decoded on the GPU through VAAPI (`EMAKI_MIRROR_SOFTWARE=1` uses the
  processor), and each frame goes to amane's `Video` as NV12, drawn by the quad
  pipeline.

What the phone sends is another device's data, so `emaki-mirror` holds it in:
- **The port.** The phone connects back (`adb reverse`) to a port on
  localhost, which another user's program could reach first: only a
  connection from a program of this user's (adb) is taken, by its owner in
  `/proc/net/tcp`.
- **Sizes.** A video or sound packet is at most 16 MiB, and a clipboard text
  from the phone at most 1 MiB; a bigger one ends the stream rather than being
  allocated.
- **The decoders.** The threads that decode video and sound confine their file
  access (Landlock) to the store and the system's files, read only, and the
  GPU's render node (video) or the sound server's settings and shared memory
  (sound): nothing of yours, whatever a bug in ffmpeg would let the phone
  do. The rest of the app, and the adb it runs, isn't confined.

### The phone's apps

The launcher finds the phone's apps by name ("On Galaxy S24"; Emaki Settings →
Launcher → Phone apps). Each one opens alone in a window of its own: a virtual
display on the phone, twice the window's size at 320 dpi, that resizes with
the window. Its app id is `emaki-mirror.<package>`, and its bar is just Back.
The list comes from `emaki-mirror --list-apps` into
`~/.cache/emaki/phone-apps.json`, asked for again after a day. An app's
window has no sound of its own; the screen's window plays the phone's.

### The camera as a webcam

Phone page → the phone → **Use as Webcam** (and **Back Camera**). It runs
`emaki-mirror --camera front|back` as the user unit `emaki-phone-camera`, so a
shell restart doesn't end a call. It writes 1280×720 frames to the v4l2loopback
device named "Phone Camera", which video call apps list. It needs Android 12.

## Limits

These come from adb and scrcpy:

- Wireless debugging is off after the phone restarts; turn it on again.
- The camera needs Android 12, and sound needs Android 11.
- scrcpy's server version is the client's protocol: it comes from nixpkgs'
  `scrcpy` at build time, and the two move together.

These come from KDE Connect itself:

- A sleeping phone can't be woken. It answers when it wakes.
- A conversation's history only exists once the phone has sent it.
- A message can't be marked read on the phone, or deleted; read marks are
  this computer's own.
- The phone sends thumbnails with messages; a whole file is fetched when it's
  opened.
- A reply that shows as sent was taken by the daemon; delivery is the phone's.

## Checking it with a phone

1. Phone page → Other Devices → the phone; accept on the phone with the same
   code. `kdeconnect-cli -l` says paired and reachable.
2. The charge shows on its row, on its page and in the bar.
3. Ring rings it; a notification on the phone shows as a toast; its music shows
   in Now Playing.
4. Send Files… arrives on the phone; a file shared from the phone lands in
   `~/Downloads`.
5. Control This Computer on → the phone's Run Command lists the four; Lock
   Screen locks.
6. Messages: the inbox fills with names and photos. A conversation loads, and a
   reply appears as Sending… and then sent. A text to the phone appears live,
   and scrolled up, the New Message pill shows. Load Earlier keeps your place.
   A photo MMS shows its thumbnail and opens; a picture attached here arrives.
   New Message to a contact makes a conversation that becomes the phone's. A
   right click copies a code. An SMS toast's Open in Messages opens its
   conversation.
7. Clipboard both ways; Share Automatically off writes `autoShare=false`.
8. Phone Input: the trackpad moves the phone's pointer; with a field focused on
   the phone, typing lands there.
9. Forget This Phone… unpairs it.
