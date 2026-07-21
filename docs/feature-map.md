# AULA Mini60 HE Pro feature map

This is the complete product-facing map for the Mini60 HE Pro in AULA's
official web configurator. It was built from a live dongle session and the
deployed configurator bundle at <https://hec.aulacn.com/> on 2026-07-20.

Protocol details are in [protocol.md](protocol.md), and the physical key-index
map plus assignable action catalog are in [mini60-layout.md](mini60-layout.md).

## Device and profile shell

- Connect/disconnect a WebHID device.
- Show product name, connected state, battery level, and firmware version.
- Show a firmware-update prompt when the deployed `update.json` contains a
  newer device build. Firmware 1.52 was installed and Windows-only firmware
  1.55 was offered during the survey.
- Create, rename, copy, delete, import, and export named profiles.
- Select a profile and write its complete state to the keyboard.
- Share a profile through the vendor account/community service. This is a
  cloud feature, not part of the local keyboard protocol.
- Store profile metadata in browser local storage. The keyboard itself exposes
  one active state, not the web UI's list of named profiles.

Selecting a profile writes, in order: normal keymap, lighting, per-key colors,
macros, optional light-box state, Fn keymap, rapid-trigger state, DKS state,
and a final commit command.

## Custom Keys

- Select any physical key on the 60% layout.
- Edit the normal layer or Fn layer.
- Assign basic USB keyboard usages.
- Assign extended keys: modifiers, navigation, F1-F12, and numpad keys.
- Assign mouse buttons, wheel directions, and browser/media consumer usages.
- Assign one of the 22 special keyboard functions exposed for this model.
- Assign a recorded macro with playback mode and repeat count.
- Reset one mapping or the layer and save it.

The underlying four-byte entry format also supports three-key combos. The
Mini60's advertised Advanced Keys list does not expose that editor, but the
deployed parser and serializer support the record.

## Lighting Settings

The keyboard exposes 20 effects:

| Value | Effect | Speed | Direction | Selectable color |
|---:|---|:---:|:---:|:---:|
| `0x01` | Static Bright | no | no | yes |
| `0x02` | Single Point On | yes | no | yes |
| `0x03` | Single Point Off | yes | no | yes |
| `0x04` | Starry Sky | yes | no | yes |
| `0x05` | Snowfall | yes | no | yes |
| `0x06` | Floral Competition | yes | no | no |
| `0x07` | Dynamic Breathing | yes | no | yes |
| `0x08` | Spectrum Cycle | yes | no | no |
| `0x09` | Color Fountain | yes | no | yes |
| `0x0a` | Colorful Interchange | yes | yes | yes |
| `0x0b` | Flowing with the Waves | yes | yes | yes |
| `0x0c` | Turning Peaks | yes | yes | yes |
| `0x0d` | One Touch to Fire | yes | no | yes |
| `0x0e` | Two Birds with One Stone | yes | no | yes |
| `0x0f` | Ripples Spread | yes | no | yes |
| `0x10` | Endless Flow | yes | yes | yes |
| `0x11` | Layered Mountains | yes | no | yes |
| `0x12` | Gentle Rain and Wind | yes | yes | yes |
| `0x13` | Back and Forth | yes | no | yes |
| `0x80` | Custom/per-key | no | no | yes |

Controls:

- Lighting on/off.
- Brightness levels 1-5.
- Speed levels 1-5 where supported.
- Two direction values where supported.
- Primary and secondary RGB colors, hex input, and eight color presets.
- Colourful/rainbow toggle.
- Gear selector for effects that advertise it. None of the Mini60's default
  20 effects use it.
- Per-key RGB editor for the custom effect, including all/group/key selection
  and reset.

The common configurator has four additional effect records (`0x17`, `0x18`,
`0x19`, and `0xfe`), but the Mini60 definition does not enable them.

## Macro Manager

- New, delete, copy, import, export, and rename macro definitions.
- Record keyboard and mouse input.
- Insert keyboard or mouse actions above or below the current action.
- Reorder or delete individual actions.
- Choose real recorded delays, no delays, or a default delay.
- Edit delays up to 9999 ms.
- Save macros into the keyboard's shared macro buffer.
- Bind a macro to a key as Routine (`0`), Press Again to End (`2`), or Repeat
  Playback (`1`). The key record also contains a repeat-count byte. The shared
  translation catalog contains a Hold label, but the Mini60 mode list does not
  expose it.

Supported mouse actions are left, middle, right, wheel up/down, forward, and
back. Keyboard recording stores USB HID keycodes and key-down/key-up events.

## Performance

Preset tabs:

- Custom
- Office Mode: all keys at 2.0 mm except Space at 3.0 mm; RT off.
- Beginner Mode: 1.5 mm base, Space at 3.0 mm, and WASD/Left Ctrl at 1.0 mm
  with 0.56 mm press/release sensitivity.
- Game Mode: 1.5 mm base, Space at 3.0 mm, and WASD/Left Ctrl at 0.5 mm with
  0.16 mm press/release sensitivity and full-distance RT.

Normal actuation:

- Per-key trigger distance.
- Displayed range for this model: 0.1-3.4 mm.
- Current model default: 1.2 mm.
- Selection helpers: all keys, WASD, and number keys.
- Per-key switch model selection.

Rapid Trigger:

- Enable separate press and release sensitivity or use one sensitivity.
- Full-distance Rapid Trigger.
- Bottom optimization bit supported by the data model.
- Rampage Mode bit.
- The UI warns against values below 0.04 mm.

Advanced performance settings:

- Top dead zone, 0.00-0.50 mm.
- Bottom dead zone, 0.00-0.50 mm.
- Per-key travel calibration.
- Clear calibration.
- Save/stop calibration.
- Wireless mode cannot display calibration state live; key lighting reports
  progress instead.

Switch choices in the deployed definition:

| Value | Translation key/name | Travel |
|---:|---|---:|
| 1 | `axis_6` | 3.3 mm wired; 3.4 mm dongle definition |
| 4 | `axis_7` | 3.4 mm |
| 2 | Magnetic Jade Pro | 3.3 mm wired; 3.4 mm dongle definition |
| 3 | Magneto Shaft | 3.3 mm wired; 3.4 mm dongle definition |
| 5 | `axis_10` | 3.4 mm |
| 6 | `axis_11` (default) | 3.4 mm |
| 7 | `axis_12` | 3.4 mm |
| 8 | Ice Cloud (`axis_35`) | 3.4 mm |

The vendor can replace the translated switch names using a remote axial-model
catalog, so OpenAula should preserve numeric values and treat names as labels.

## Advanced Keys

Each profile can contain up to 40 advanced bindings. An advanced binding may
not occupy multiple layers of the same physical key, but the same feature may
be used on different physical keys/layers.

- **RS / Rappy Snappy:** pair two physical keys and activate whichever is
  pressed farther. It shares the selected keys' RT actuation settings.
- **SOCD:** pair two keys with Last Input Priority, absolute Key 1 priority,
  absolute Key 2 priority, or Neutral/Anti-Ghosting mode. It can also carry RT
  actuation and sensitivity settings for the pair.
- **DKS / Dynamic Keystroke:** attach up to four key actions to four pressure
  points and four travel phases: Key Start, Key Bottom, Lift When Pressed to
  Bottom, and Completed Lift. Each key/phase can be a one-shot point or a held
  path. Default points are 1.6/3.0/3.0/1.6 mm; outer and inner pairs move
  together in 0.1 mm steps.
- **MT / Mod-Tap:** one action on hold and another on tap. Hold time is
  adjustable from 10-1000 ms in 10 ms units and defaults to 400 ms.
- **TGL / Toggle:** tapping locks/unlocks continuous activation; holding acts
  like a normal key press.

The DKS, MT, and TGL target pickers expose Basic and Extended keyboard actions
only. RS and SOCD select physical key pairs.

The common app also implements a three-key Combo/CB editor, but that feature is
not enabled by the Mini60 device definition.

## Settings

Interface settings are host-side and offer Light, Dark, and system-synchronized
themes. OpenAula's native UI is intentionally English-only, so the vendor
language picker is not part of the port.

Wired Mini60:

- Wake behavior.
- Stability mode.
- Adaptive Dynamic Calibration (Beta).
- 1 kHz, 4 kHz, and 8 kHz report-rate selector.
- Firmware version/update.
- No sleep-time control.

Mini60 dongle:

- Sleep/idle mode: byte 3 is `0` for off or 1-30 minutes.
- Wake behavior.
- Stability mode.
- Adaptive Dynamic Calibration (Beta).
- Firmware version/update.
- Report-rate selector hidden, although the current rate byte is readable.

The shared configurator also has fields for response time, OS mode, TFT
timeout, screen/GIF upload, telemetry, music-reactive lighting, side-light
effects, and community pages. The Mini60 definition hides those pages or
controls (`listBtnHideIds` 6, 7, and 8), but their protocol commands are
documented because the core parser may eventually support related devices.

Wake byte 15 is `1` for the lower-power single-key wake mode and `0` for
all-key wake. The wired report-rate choices are 1 kHz (`3`), 4 kHz (`5`), and
8 kHz (`6`). Firmware metadata/update download belongs on the Update page, but
native firmware flashing remains out of scope.

## Destructive operations

- Reset current key configuration: reset command subcode `0x05`.
- Factory-reset all settings: reset command subcode `0xff`.
- Calibration start/clear and stop/save.
- Firmware/TFT/GIF flashing commands.

OpenAula should require explicit confirmation for resets and calibration.
Firmware flashing remains out of scope until a recovery procedure is proven.
