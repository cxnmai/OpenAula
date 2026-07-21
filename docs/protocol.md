# Reverse-engineered AULA HID protocol

Status: enough is known to implement safe discovery, complete read-only dumps,
normal configuration writes, and round-trip backups for the Mini60 HE Pro.
Reset, calibration, screen transfer, and firmware operations are documented but
must be treated as hazardous.

## Evidence and confidence

The mapping combines:

1. Static inspection of the exact JavaScript bundle deployed at
   <https://hec.aulacn.com/> on 2026-07-20 (`app.c35ca532.js` and
   `chunk-keyboard.970a9da9.js`).
2. Live inspection of the app's Vue state and WebHID descriptors.
3. A pass-through WebHID trace of 1,161 input/output events from three
   initialization cycles on `MINI 60 HE PRO Dongle`, firmware 1.52.

The instrumentation recorded arguments and input reports but did not alter
packets. No setting, reset, calibration, or firmware command was issued.

Labels in this document:

- **Captured**: appeared in the live trace.
- **Builder**: exact deployed packet-building code exists, but it was not sent
  during the trace.
- **Inferred**: semantics come from parser/UI behavior and need a write/readback
  test on disposable settings before being considered final.

## HID interfaces

| Transport | VID:PID | Product | Usage page | Usage | Report |
|---|---|---|---:|---:|---:|
| Wired | `0c45:80a2` | `MINI 60 HE PRO` | `0xff68` | `0x61` | 64 bytes |
| Dongle | `0c45:fefe` | `MINI 60 HE PRO Dongle` | `0xff60` | `0x61` | 32 bytes |

Both expose input report 0 and output report 0. Do not identify a supported
keyboard from PID `fefe` alone: the common driver uses that generic PID for
many dongles, so product name, vendor, usage page, and descriptor shape should
also match.

## Bulk-frame format

Read and state-write commands use a fixed-size HID report with an eight-byte
header:

| Offset | Size | Meaning |
|---:|---:|---|
| 0 | 1 | `0xaa` host request; `0x55` device response |
| 1 | 1 | command |
| 2 | 1 | payload length in this frame |
| 3 | 2 | destination/source offset, little-endian |
| 5 | 1 | metadata; usually zero |
| 6 | 1 | final-chunk flag (`1` on last frame) |
| 7 | 1 | reserved, zero |
| 8 | N | payload |

The payload capacity is 56 bytes wired and 24 bytes through the dongle. The
sender waits for a matching input report before sending the next chunk. The
web app times a bulk sequence out after five seconds. Its lower-level sender
retries a silent report twice at roughly three-second intervals.

For reads, the host sends zero-filled payload chunks sized for the destination
buffer. The device replies with the same command, length, offset, and final
flag, replacing the payload. Response data is copied to `buffer[offset..]`.

After a write sequence, command `0x00` with four zero data bytes is used as a
commit/finalize barrier. The official receiver ignores incoming command
`0x37`; its purpose is not established.

### Short commands

Reset, status, calibration, and key-test packets use another shape:

```text
aa COMMAND ARG0 ARG1 ARG2 ARG3 00 ...
```

Arguments begin at byte 2 and the rest of the current report is zero-filled.

## Initialization and polling

The official app opens report 0, installs an input listener, then performs this
acknowledged sequence:

| Order | Command | Logical buffer | Bytes | Captured |
|---:|---:|---|---:|:---:|
| 1 | `0x10` | device info | 32 | yes |
| 2 | `0x11` | keyboard settings | 32 | yes |
| 3 | `0x12` | normal keymap | 512 | yes |
| 4 | `0x13` | lighting | 16 plus preserved padding | yes |
| 5 | `0x14` | per-key colors | 512 | yes |
| 6 | `0x15` | macros | 1024 maximum | yes |
| 7 | `0x16` | Fn keymap | 512 | yes |
| 8 | `0x17` | rapid trigger | 1024 | yes |
| 9 | `0x18` | DKS | 1024 | yes |
| 10 | `0x00` | finalize | 4 | yes |
| 11 | `0x1c` | default Fn map, conditional | 512 | no on Mini60 |

The macro read stops early if a returned payload is all zero after offset 400;
do not require a full 1024 bytes or final flag in that special case. The
lighting buffer starts at 16 bytes, but later dongle reads can expand it to the
full 24-byte payload; a round-trip implementation must preserve unknown tail
bytes.

When idle on the dongle, the app repeats command `0x10` every five seconds to
refresh the battery percentage.

## Command table

| Value | Direction/use | Meaning | Evidence |
|---:|---|---|---|
| `0x00` | read/write barrier | finalize/commit | Captured |
| `0x0f` | short write | reset; `05` key config, `ff` all settings | Builder |
| `0x10` | read | device identity, firmware, battery | Captured |
| `0x11` | read | keyboard settings | Captured |
| `0x12` / `0x22` | read/write | normal keymap | Captured / Builder |
| `0x13` / `0x23` | read/write | global lighting | Captured / Builder |
| `0x14` / `0x24` | read/write | per-key RGB | Captured / Builder |
| `0x15` / `0x25` | read/write | macro buffer | Captured / Builder |
| `0x16` / `0x26` | read/write | Fn keymap | Captured / Builder |
| `0x17` / `0x27` | read/write | rapid-trigger records | Captured / Builder |
| `0x18` / `0x28` | read/write | DKS records | Captured / Builder |
| `0x1b` / `0x2b` | read/write | side/light-box state | Builder; hidden on Mini60 |
| `0x1c` | read | factory/default Fn map | Builder; conditional |
| `0x21` | write | keyboard settings | Builder |
| `0x32` / `0x42` | read/write | 64-byte music RGB; `0x42` also GIF flash | Builder |
| `0x34` | write | clock or host telemetry for screen devices | Captured clock echo; hidden |
| `0x35` | write | 32-byte music RGB stream | Builder |
| `0x41` | write | screen flash frame | Builder; hazardous |
| `0x50` | write | 4 KiB TFT frame | Builder; hazardous |
| `0x62` | short | status | Builder |
| `0x64` | short write | start/clear calibration | Builder; hazardous |
| `0x65` | short write | stop/save calibration | Builder; hazardous |
| `0x66` / `0x67` | short write | start/stop key-test mode | Builder |
| `0xfa` | input/read | screen information | Parser; hidden |
| `0xfb` | input | calibration data/progress | Parser |
| `0xfc` | input | control: subcode 4 disconnect, 5 reload | Parser |

The firmware and screen builders use large 4104-byte application buffers and
a different page/chunk index header. They should not be generalized from the
ordinary 32/64-byte bulk frame without hardware recovery testing.

## Device-info buffer (`0x10`, 32 bytes)

| Offset | Meaning |
|---:|---|
| 3-7 | opaque identity/model bytes |
| 8 | packed BCD-like firmware minor (`0x52` → 52) |
| 9 | firmware major (`1`) |
| 12-16 | opaque capability/model bytes |
| 17 | battery percent |
| 20-21 | axial/switch model ID, little-endian |
| 22-23 | maximum frame count |
| 24-25 | maximum GIF frames |
| 26-27 | maximum LED frames |
| 28 | rotate screen-layout definition when nonzero |

Firmware display formula:

```text
(low_nibble(byte8) + 10*high_nibble(byte8) + 100*byte9) / 100
```

The last captured Mini60 data decoded to firmware 1.52, battery 61%, and axial
model 16. Battery changed across the trace, confirming the five-second poll.

## Keyboard-settings buffer (`0x11` / `0x21`)

| Offset | Meaning | Unit/values |
|---:|---|---|
| 3 | sleep time | `0` off; otherwise 1-30 minutes |
| 4 | response time | device-specific |
| 5 | report rate | `3`=1 kHz, `5`=4 kHz, `6`=8 kHz |
| 6 | OS mode | device-specific enum |
| 7 | TFT timeout | device-specific |
| 8 | top dead zone | 0.01 mm |
| 9 | bottom dead zone | 0.01 mm |
| 11 | stability mode | boolean |
| 14 | adaptive dynamic calibration | boolean |
| 15 | wake behavior | `0` all-key wake; `1` single-key wake |

The last captured state was sleep 1 minute, response time 0, rate code 6,
OS/TFT mode 0, 0.30/0.30 mm dead zones, stability on, adaptive calibration on,
and wake off. The web writer sends 16 bytes and then finalizes.

## Keymaps (`0x12`/`0x22`, `0x16`/`0x26`)

Normal and Fn maps contain 128 four-byte records. See
[mini60-layout.md](mini60-layout.md) for the physical slots, complete action
catalog, record tags, and SOCD values.

The write path can update a whole 512-byte map or send one 4-byte record at
offset `4 * slot`. Rappy Snappy and SOCD write the same record to both members
of the pair. DKS records refer to a separate 16-byte DKS slot.

## Lighting (`0x13` / `0x23`)

| Offset | Meaning |
|---:|---|
| 0 | effect; zero turns lighting off |
| 1-3 | primary R, G, B |
| 4 | writer forces `0xff`; returned value may differ |
| 5-7 | secondary R, G, B |
| 8 | colorful/rainbow toggle |
| 9 | brightness, normally 1-5 |
| 10 | speed, normally 1-5 |
| 11 | direction, zeroed for effects without direction |
| 12 | gear/mode selector |
| 13 | reserved |
| 14-15 | writer signature `aa 55`; returned values may be zero |

The live state was effect `0x0b`, white primary, black secondary, colorful on,
brightness 5, speed 3, direction 0. The device returned zero at offset 4 and
for the signature, while the official writer forces `ff` and `aa 55`.

Per-key color state is 128 records of `[slot, red, green, blue]`. Effect
`0x80` activates the custom editor.

## Rapid-trigger buffer (`0x17` / `0x27`)

There are 128 eight-byte records:

| Offset | Meaning |
|---:|---|
| 0 | switch type; zero selects the model default |
| 1 bit 0 | full-distance RT |
| 1 bit 1 | bottom optimization |
| 1 bit 2 | Rampage Mode |
| 2-3 | actuation, little-endian, 0.01 mm |
| 4-5 | press/down sensitivity, little-endian, 0.01 mm |
| 6-7 | release/up sensitivity, little-endian, 0.01 mm |

An all-zero entry inherits the device definition: switch value 6, 1.2 mm
normal actuation, and sensitivity defaults. The official writer supports a
whole 1024-byte update and finalizes after it.

## DKS buffer (`0x18` / `0x28`)

There are 64 sixteen-byte records:

| Offset | Meaning |
|---:|---|
| 0-3 | four trigger points |
| 4,6,8,10 | reserved zero bytes |
| 5,7,9,11 | four USB keycodes |
| 12-15 | actions at four travel phases |

For each phase byte, bits 0-3 are one-shot actions for keys 1-4 and bits 4-7
are held/dragged actions for keys 1-4. The phases are Key Start, Key Bottom,
Lift When Pressed to Bottom, and Completed Lift. A keymap DKS record stores the
DKS slot number in byte 1.

## Macro buffer (`0x15` / `0x25`)

The first 400 bytes are up to 100 little-endian 32-bit record pointers. A zero
pointer terminates the list. At each pointer:

| Offset | Meaning |
|---:|---|
| 0-1 | action-byte count; divide by two for number of 4-byte actions |
| 2-3 | reserved/header |
| 4+ | action records |

Each action is:

| Offset | Meaning |
|---:|---|
| 0-1 | delay in ms, little-endian |
| 2 | key/button code |
| 3 bit 7 | key down when set; key up when clear |
| 3 bit 5 | keyboard when set; mouse when clear |

The app allocates 1024 bytes but sets a Mini60 UI macro maximum of 512. A safe
implementation should preserve and round-trip the full returned buffer while
enforcing the device-specific write limit once confirmed.

## Profiles and write transaction

Profiles are web-app data. A full profile activation writes:

```text
22 keymap
23 lighting
24 per-key colors
25 macros
2b light-box (only when enabled)
26 Fn keymap
27 rapid trigger
28 DKS
00 finalize
```

For safety, `aula dump` should be implemented before any setter. Writes should
default to a dry-run that prints exact reports, preserve unknown/padding bytes,
validate all lengths and units, write the minimum changed section, finalize,
then read back and byte-compare.

## Known unknowns

- The meaning of opaque identity bytes in device info.
- Exact enum meanings for response time and OS mode on this model.
- Meanings of report-rate codes other than the UI's `3`, `5`, and `6`; unknown
  values must be preserved.
- Why command `0x37` input reports are explicitly ignored.
- The macro buffer's true safe write limit (512 vs allocated/read 1024).
- Firmware, TFT, GIF, music, light-box, and telemetry behavior on Mini60
  hardware, because its UI hides most of those features.
- Recovery behavior after an interrupted write or firmware flash.

These uncertainties do not block read-only dumps, normal keymap/lighting/
performance parsers, or exact preservation of unknown bytes.
