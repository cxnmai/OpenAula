# Native desktop UI specification

This is the implementation brief for `openaula-desktop`. It maps the Mini60
HE Pro screens in AULA's deployed web configurator to the state and commands
owned by `openaula-core`. The survey used the English UI, a live
`MINI 60 HE PRO Dongle`, and the deployed `app.c35ca532.js` and
`chunk-keyboard.970a9da9.js` bundles on 2026-07-20.

The machine-readable catalog beside this document is
[`mini60-gui.json`](mini60-gui.json). Wire formats are in
[`protocol.md`](protocol.md), and key slots/action values are in
[`mini60-layout.md`](mini60-layout.md).

## Product scope

- The initial desktop app supports the wired Mini60 HE Pro (`0c45:80a2`) and
  Mini60 HE Pro Dongle (`0c45:fefe`).
- The interface is English-only. Do not port the web app's language picker or
  translation catalog.
- Support light, dark, and system theme preferences as host-only state.
- Profiles are local files. Accounts, community sharing, vendor telemetry,
  music-reactive plugins, displays, and cloud services are out of scope.
- Show firmware/update information, but do not expose firmware flashing until
  a tested recovery path exists.
- Preserve every unknown/reserved byte. Edits begin from a fresh device dump,
  modify only modeled fields, create a backup, write, and verify by readback.

## Application shell and state model

The app has a device/profile sidebar, a page navigation row, a physical
keyboard preview, a contextual editor, and one persistent save/status area.
The six pages are Custom Keys, Lighting Settings, Macro Manager, Performance,
Advanced Keys, and Settings.

Use one in-memory `EditorSession` per connected device:

1. Discover a supported configuration interface and read a complete
   `ConfigurationDump`.
2. Decode typed views from that lossless dump.
3. Keep `original`, `working`, and `dirty_sections` separately.
4. Make normal controls stage changes locally. Do not copy the web app's
   immediate-write behavior for settings sliders and toggles.
5. On Save, create a full backup, write only dirty sections in protocol order,
   finalize once, reread, and compare.
6. Disable editors and show a recoverable disconnected state after an HID
   timeout. A dongle may remain enumerated while the keyboard is unavailable.

The device header displays product name, Connected/Disconnected, battery for
the dongle, and firmware version. The official dongle app polls device info
every five seconds; the native app should poll only while idle and suspend the
poll during writes.

Profiles are host-side snapshots, not firmware slots. Required actions are
create, rename, copy, import, export, delete, and activate. Activating a profile
writes its complete keyboard state. Built-in/default profiles should be
copyable and exportable but not deletable. A profile picker must visibly mark
unsaved edits and confirm before replacing them.

## Shared keyboard selection behavior

Render the 61-key layout from `mini60-layout.md`, retaining sparse firmware
slot numbers. Keys need selected, multi-selected, changed, disabled, advanced
binding, and live-test visual states. The Performance page provides All,
WASD, and number-row selection helpers. Advanced Keys uses the same preview to
choose one or two physical keys.

Normal and Fn layers share the preview but read/write separate 512-byte maps.
Assignment drawers use these groups:

- Basic characters: A-Z, number row, punctuation, non-US backslash, and Fn.
- Extended characters: modifiers, navigation, F1-F12, and numpad usages.
- Special characters: mouse buttons/wheel and consumer/media/browser usages.
- Function keys: the 22 Mini60 special functions.
- Macros: a saved macro plus playback mode and repeat count.

For DKS, Mod-Tap, and Toggle targets, the web app intentionally exposes only
Basic Characters and Extended Characters. RS and SOCD select physical key
pairs instead of target actions.

## Custom Keys

Controls:

- Normal Layer / Fn layer tabs.
- One selected physical key at a time.
- Basic, Extended, Special, Function, and Macro assignment groups.
- Reset selected mapping, reset current layer, and Save.

Assignments are four-byte `KeyAssignment` records. Removing an override writes
an all-zero record so firmware supplies the factory action. Reset operations
must be scoped clearly: selected key, current layer, key configuration, or all
device settings are different operations.

## Lighting Settings

The page contains an on/off switch, 20 effect tiles, and controls that appear
only when supported by the selected effect. The exact IDs and capability
matrix are in `mini60-gui.json` and `feature-map.md`.

Shared control limits:

- Brightness: integer 1-5.
- Speed: integer 1-5.
- Direction: two values, encoded 0/1.
- Primary and secondary colors: RGB fields plus `#RRGGBB` input.
- Palette: black, blue, cyan, green, yellow, red, magenta, white.
- Colorful/rainbow toggle.
- Custom effect `0x80`: select all, WASD, number keys, direction keys, or
  individual keys, then edit per-key RGB and reset the selection.

Effect zero is lighting off. Retain fields hidden by the selected effect rather
than destroying them in the working model. The writer must preserve lighting
padding/reserved bytes returned by the device.

## Macro Manager

The left list supports New, Delete, Copy, Import, Export, and Rename. The
editor supports keyboard or mouse actions, insertion above/below the cursor,
recording, reordering, deletion, and editable delays.

Timing modes are recorded interval, no interval, and a default interval
(10 ms initially). Delay values are stored as 16-bit milliseconds; the web UI
accepts up to 9999 ms. Supported mouse actions are left, middle, right,
forward, back, wheel up, and wheel down.

The Mini60 assignment UI exposes three playback modes:

| Value | Mode |
|---:|---|
| `0` | Routine |
| `2` | Press Again to End |
| `1` | Repeat Playback |

The assignment record also stores a repeat-count byte. A translated Hold label
exists in the shared bundle, but it is not present in the Mini60 playback-mode
list and must not be shown without new device evidence.

The web editor caps `macro definitions + action records` at 512. The protocol
buffer is 1024 bytes: up to 400 bytes of macro offsets followed by variable
records. The native serializer must enforce actual buffer size and valid
offsets rather than relying only on the web app's approximate item count.

## Performance

### Presets

Tabs are Custom, Office Mode, Beginner Mode, and Game Mode. Selecting a preset
updates the working per-key RT records; it does not create a distinct firmware
mode.

- Office: unspecified keys at 2.0 mm; Space at 3.0 mm; RT off.
- Beginner: unspecified keys at 1.5 mm; Space at 3.0 mm; WASD and Left Ctrl at
  1.0 mm with 0.56 mm press/release sensitivity; full-distance RT off.
- Game: unspecified keys at 1.5 mm; Space at 3.0 mm; WASD and Left Ctrl at
  0.5 mm with 0.16 mm press/release sensitivity; full-distance RT on.

Custom restores the profile's staged per-key values. A preset preview should
show the affected keys before Save.

### Normal Mode

- Per-key Trigger Distance: 0.10-3.40 mm, step 0.01 mm.
- Current device default: 1.20 mm.
- Switch type: eight model values, with 6 as the Mini60 default. Names and
  travel values are in `mini60-gui.json`; preserve numeric IDs because the
  vendor can replace labels remotely.

### RT mode

- Fast Trigger enabled when press/release sensitivity is nonzero.
- Shared Sensitivity or independently set Press Sensitivity and Release
  Sensitivity.
- Sensitivity range: 0.01-3.40 mm, step 0.01 mm. Warn below 0.04 mm as the web
  app does, but allow representable values.
- Full Distance Fast Trigger flag.
- Bottom Optimization and Rampage flags exist in the record model even though
  the surveyed Mini60 screen does not expose them.

### Advanced Settings

Top Dead Zone and Bottom Dead Zone each range from 0.00 to 0.50 mm in 0.01 mm
steps. They are global keyboard settings even though the web app presents them
inside Performance.

### Recalibrate

Calibration start/clear (`0x64`) and stop/save (`0x65`) are hazardous control
commands, not ordinary staged fields. Require a backup and explicit
confirmation before start/clear. On wireless, the app cannot show live travel
status; explain that the keyboard LEDs report progress. Never leave the device
in calibration mode when the window closes normally.

## Advanced Keys

Each profile can use at most 40 advanced bindings. One physical key/layer may
have only one advanced binding, while the same physical key may use different
bindings on Normal and Fn layers. Provide a Current Bindings counter and a
start/stop Test Your Bindings action (`0x66`/`0x67`).

### RS / Rappy Snappy

Choose exactly two physical keys. The editor shows the pair and Trigger
Distance, default 1.20 mm, range 0.10-3.40 mm for the Mini60. The pair shares
per-key RT records and writes assignment type 12 to both physical slots.

### SOCD

Choose exactly two physical keys, Trigger Distance, optional Fast Trigger
settings, and one behavior:

| Value | Behavior | Meaning |
|---:|---|---|
| `3` | Last Input Priority | The last activated key overrides the previous key. |
| `1` | Absolute Key 1 Priority | Key 1 always takes precedence. |
| `2` | Absolute Key 2 Priority | Key 2 always takes precedence. |
| `4` | Anti-Ghosting/Neutral | When both activate, neither triggers. |

Write assignment type 11 to both physical slots. Press/release sensitivity and
flags live in those keys' RT records, not in the four-byte SOCD record.

### DKS / Dynamic Key Travel

Choose one physical key and up to four Basic/Extended target actions. The four
phases are Key Start, Key Bottom, Lift When Pressed to Bottom, and Completed
Lift. Every action/phase cell can be off, one-shot, or held across a path.

Default pressure points are 1.6, 3.0, 3.0, and 1.6 mm. The outer pair is linked
and the inner pair is linked. For a 3.4 mm switch, outer values are
0.1-3.3 mm and inner values are 0.1-3.4 mm, step 0.1 mm; inner must remain at
least 0.1 mm deeper than outer. Store points in tenths of a millimeter and the
phase matrix in the four one-shot/held bitfields described in `protocol.md`.

### MT / Mod-Tap

Choose one physical key, a hold action, and a tap action from Basic/Extended
characters. Hold Time ranges from 10 to 1000 ms in 10 ms steps and defaults to
400 ms. The four-byte assignment stores time divided by 10.

### TGL / Toggle

Choose one physical key and one Basic/Extended target action. Tapping locks or
unlocks continuous activation; holding behaves like a normal key press.

## Settings

Settings has three pages: Interface Settings, Device settings, and Update.
Language is deliberately omitted from OpenAula.

### Interface Settings

Theme choices are Light Theme, Dark Theme, and Sync theme with computer. This
is host-only persistent state and must never produce a device write.

### Device settings

| Control | Dongle | Wired | Encoding |
|---|:---:|:---:|---|
| Sleep Mode | yes | no | settings byte 3; `0` off, `1..30` minutes |
| Turn Off Time | yes | no | integer 1-30 min; web default when re-enabled is 1 |
| All Key & Single Key Wakeup | yes | yes | byte 15; `1` single-key, `0` all-key |
| Stability Mode | yes | yes | byte 11 boolean |
| Adaptive Dynamic Calibration (Beta) | yes | yes | byte 14 boolean |
| Report Rate | hidden | yes | byte 5; 1 kHz=`3`, 4 kHz=`5`, 8 kHz=`6` |
| Reset all settings | yes | yes | reset subcode `0xff` |

Single-key wake uses less power while asleep. Adaptive calibration continuously
adjusts switch travel; instruct the user to disconnect power before replacing
switches. Reset all settings requires a fresh backup and the same explicit
confirmation standard used by the CLI.

Response time, OS mode, TFT timeout, display upload, telemetry, and similar
controls exist in the shared configurator but are disabled by the Mini60
device definition. Preserve their raw bytes; do not render them.

### Update

Display current firmware and whether vendor metadata reports a newer version.
The surveyed device ran 1.52 and was offered 1.55 as a Windows-only package.
`Check for Updates` may refresh metadata and offer a download link, but the
native app must not execute a vendor updater or send flash commands.

## Core/frontend boundary

`openaula-core` already owns device discovery, lossless dumps, key assignment
records, keyboard settings, lighting, RT records, DKS records, transport, and
safe whole-section writes. Before building the desktop editors, add typed core
APIs for:

- physical layout and device capability catalogs;
- effect/switch/preset catalogs;
- macro parsing, validation, and serialization;
- per-key RGB records;
- profile file schema and migrations;
- high-level validated edit operations that return changed protocol sections;
- one transactional write/backup/readback service shared by CLI and desktop.

The desktop crate should depend on those APIs and never assemble HID packets,
edit opaque offsets, or duplicate safety policy itself.

## Acceptance checklist

- Every control listed above loads from a dump, stages edits, survives page
  navigation, saves through core, and verifies by readback.
- Switching profiles or disconnecting with dirty state prompts the user.
- Unsupported controls are absent based on device capabilities, not merely
  disabled visually.
- Unknown bytes and unknown enum values round-trip unchanged.
- No configuration write occurs from opening a page, changing theme, checking
  updates, or canceling an editor.
- Reset, calibration, and eventual firmware operations are visually separated
  from normal Save and require stronger confirmation.
- Keyboard-only navigation, visible focus, labels, numeric text entry, and
  high-contrast selected states work in light and dark themes.
