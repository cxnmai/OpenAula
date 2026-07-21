# Mini60 HE Pro layout and assignable actions

The firmware uses 128 sparse key slots. Each normal/Fn assignment occupies
four bytes at `4 * slot`; each per-key lighting record also starts with the
slot. These are the 60 physical slots exposed by the official layout.

| Row | Slot → physical key |
|---|---|
| 1 | `0` Esc, `17` 1, `18` 2, `19` 3, `20` 4, `21` 5, `22` 6, `23` 7, `24` 8, `25` 9, `26` 0, `27` -, `28` =, `92` Backspace |
| 2 | `32` Tab, `33` Q, `34` W, `35` E, `36` R, `37` T, `38` Y, `39` U, `40` I, `41` O, `42` P, `43` [, `44` ], `60` Backslash |
| 3 | `48` Caps, `49` A, `50` S, `51` D, `52` F, `53` G, `54` H, `55` J, `56` K, `57` L, `58` ;, `59` ', `76` Enter |
| 4 | `64` Left Shift, `65` Z, `66` X, `67` C, `68` V, `69` B, `70` N, `71` M, `72` comma, `73` period, `74` slash, `75` Right Shift |
| 5 | `80` Left Ctrl, `81` Left Win, `82` Left Alt, `83` Space, `84` Right Alt, `86` Menu/Super, `87` Right Ctrl, `85` Fn |

The live keyboard returned an all-zero normal keymap, meaning the factory
layout is implicit. Its only explicit Fn records were W→Up, A→Left, S→Down,
and D→Right at slots 34, 49, 50, and 51.

## Basic characters

These use standard USB Keyboard/Keypad usage IDs:

- A-Z: `4`-`29`.
- 1-0: `30`-`39`.
- `-` `45`, `=` `46`, `[` `47`, `]` `48`, backslash `49`, `;` `51`,
  apostrophe `52`, backtick `53`, comma `54`, period `55`, slash `56`.
- Non-US backslash: `100`.
- The app's internal Fn action is `175`; Fn is also a physical slot whose
  factory keycode is zero.

The official UI supplies localized legends for US, German, Portuguese, and
Japanese layouts, but it writes the same HID usage IDs.

## Extended keyboard actions

| Name | HID usage | Name | HID usage |
|---|---:|---|---:|
| Escape | 41 | Tab | 43 |
| Caps Lock | 57 | Backspace | 42 |
| Enter | 40 | Space | 44 |
| Left Ctrl | 224 | Right Ctrl | 228 |
| Left Shift | 225 | Right Shift | 229 |
| Left Alt | 226 | Right Alt | 230 |
| Left Win | 227 | Menu/Super | 101 |
| Up | 82 | Down | 81 |
| Right | 79 | Left | 80 |
| Home | 74 | End | 77 |
| Insert | 73 | Delete | 76 |
| Page Up | 75 | Page Down | 78 |
| Print Screen | 70 | Scroll Lock | 71 |
| Pause | 72 | F1-F12 | 58-69 |
| Num Lock | 83 | Numpad 1-9 | 89-97 |
| Numpad 0 | 98 | Numpad `/` | 84 |
| Numpad `.` | 99 | Numpad `*` | 85 |
| Numpad `-` | 86 | Numpad `+` | 87 |
| Numpad Enter | 88 | Disabled/no-op symbol | 107 |

## Mouse and consumer actions

Mouse records use a mouse-kind byte plus an eight-bit code:

| Action | Kind | Code |
|---|---:|---:|
| Left button | 1 | 1 |
| Middle button | 1 | 4 |
| Right button | 1 | 2 |
| Wheel up | 3 | 1 |
| Wheel down | 3 | 255 |
| Forward button | 1 | 16 |
| Back button | 1 | 8 |

Consumer usages are stored little-endian in the middle two bytes:

| Action | Usage | Action | Usage |
|---|---:|---|---:|
| My Computer | 404 | Calculator | 402 |
| Email | 394 | Multimedia | 387 |
| Play/Pause | 205 | Stop | 183 |
| Previous track | 182 | Next track | 181 |
| Volume up | 233 | Volume down | 234 |
| Mute | 226 | Browser Home | 547 |
| Refresh | 551 | Forward | 549 |
| Back | 548 | Favorites | 554 |
| Search | 545 | | |

## Mini60 special functions

Special-function records store a 24-bit big-endian value. The Mini60 device
definition filters the common catalog to values 1-22:

| Value | Function | Value | Function |
|---:|---|---:|---|
| 1 | Factory reset | 12 | Lighting color cycle |
| 2 | Bluetooth channel 1 | 13 | Brightness up |
| 3 | Bluetooth channel 2 | 14 | Brightness down |
| 4 | Bluetooth channel 3 | 15 | Lighting speed up |
| 5 | Wireless reconnect | 16 | Lighting speed down |
| 6 | Number-row/F-row conversion | 17 | Lighting off |
| 7 | Show/query battery | 18 | Alt+Tab |
| 8 | Windows mode | 19 | Win+E |
| 9 | macOS mode | 20 | Win+Tab |
| 10 | Android mode | 21 | Win+H |
| 11 | Lighting effect cycle | 22 | Windows-key lock |

## Assignment record types

| Byte 0 | Meaning | Remaining bytes |
|---:|---|---|
| `0` | Empty/factory default | zero |
| `1` | Mouse | kind, code, reserved |
| `2` | Keyboard | `0, usage` for ordinary; `usage, 0` for extended |
| `3` | Consumer/media | 16-bit little-endian usage, reserved |
| `6` | Macro | macro index, playback mode, repeat count |
| `7` | Three-key combo | key 1, key 2, key 3 |
| `8` | DKS | DKS record index, reserved, reserved |
| `9` | Mod-Tap | hold key, tap key, hold time / 10 ms |
| `10` | Toggle | key, reserved, reserved |
| `11` | SOCD | mode, key 1, key 2 |
| `12` | Rappy Snappy | reserved, key 1, key 2 |
| `13` | Special function | 24-bit big-endian function value |

SOCD mode bytes are `1` Key 1 priority, `2` Key 2 priority, `3` last-input
priority, and `4` neutral when both are active.
