# `aula` CLI guide

`aula` is the local command-line frontend for OpenAula. Run `aula --help` or
`aula <command> --help` for the authoritative option list.

## Device selection

```sh
aula devices
aula --device 0 info
aula --device 0c45:fefe info
aula --device /dev/hidraw12 info
```

With no selector, `aula` automatically opens the only supported configuration
interface. If more than one is present, it refuses to guess. Discovery matches
the model's vendor/product ID, usage page, usage, and report size, so ordinary
keyboard, mouse, media, and unrelated vendor interfaces are excluded.

A wireless dongle can enumerate while the keyboard itself is switched off or
using another connection mode. In that case `aula devices` still lists the
dongle, while read commands end with a timeout. Change the timeout with
`--timeout-ms` if needed.

## Reads and backups

```sh
aula info
aula --json info
aula dump --output backup.json
aula dump                         # JSON to stdout
aula keymap get --layer normal
aula keymap get --layer fn --slot 34
aula performance get --slot 49
aula lighting get
aula settings get
```

A complete dump is a versioned JSON document containing every known section as
hexadecimal bytes. Unknown and reserved values remain intact. Output files use
exclusive creation: choose a new path instead of overwriting an existing
backup.

`aula restore backup.json` validates the schema and every section length
without opening a device. Add `--apply` to back up the current device, restore
all writable sections, read them back, and compare them.

## Safe writes

Configuration setters first read the current section and show a preview. They
do not send a write until `--apply` is supplied:

```sh
aula settings set --report-rate 8000 --top-dead-zone 0.20
aula settings set --report-rate 8000 --top-dead-zone 0.20 --apply

aula lighting set --effect 0x0b --color ff8000 --secondary 000000 \
  --colorful true --brightness 5 --speed 3

aula performance set --slots 49,50,51 \
  --actuation 1.20 --press 0.15 --release 0.15 \
  --full-travel true
```

Applied writes create `aula-backup-<unix-ms>.json` in the current directory by
default. Use `--backup PATH` to select a path; the path must not already exist.
After writing, `aula` rereads the section and fails if verification differs.

Boolean setter values are explicit: use `--stability true` or
`--stability false`, for example.

The Settings-page device controls are available directly: use
`--sleep-minutes 0` to disable sleep or `1..30` for the dongle timeout;
`--wake true` selects lower-power single-key wake and `--wake false` selects
all-key wake. Stability, adaptive calibration, dead zones, and the wired
1/4/8 kHz report-rate choices map to their corresponding setter options.

## Key assignments

The Mini60 uses 128 sparse firmware slots. `aula keymap get` prints the 61
physical keys and their slot numbers. A setter modifies one slot while
preserving the rest of the 512-byte map.

```sh
aula keymap set --slot 49 --action key:a
aula keymap set --layer fn --slot 34 --action key:up
aula keymap set --slot 80 --action extended:224
aula keymap set --slot 49 --pair-slot 51 --action socd:3:a:d
aula keymap set --slot 0 --action factory
```

Supported action expressions are:

| Form | Meaning |
|---|---|
| `factory` | Remove the override and use the implicit factory action |
| `key:<name-or-usage>` | Standard keyboard usage |
| `extended:<usage>` | Extended/special keyboard usage encoding |
| `mouse:<kind>:<code>` | Mouse action |
| `consumer:<usage>` | Consumer/media usage |
| `macro:<index>:<mode>:<repeat>` | Macro reference |
| `combo:<key1>:<key2>:<key3>` | Three-key combo |
| `dks:<slot>` | Dynamic Keystroke record reference |
| `mt:<hold>:<tap>:<milliseconds>` | Mod-Tap action |
| `toggle:<key>` | Toggle key |
| `socd:<mode>:<key1>:<key2>` | SOCD pair |
| `rs:<key1>:<key2>` | Rappy Snappy pair |
| `special:<1-22>` | Mini60 special function |
| `raw:<8 hex digits>` | Exact four-byte record |

Key values accept decimal, `0x` hexadecimal, letters, digit-key names, common
key names, arrows, modifiers, and F1-F12. A one-character digit means that
keyboard key (`1` is usage 30); use hexadecimal for a low numeric usage (`0x01`
is usage 1). SOCD and Rappy Snappy require `--pair-slot`, and `aula` writes the
same assignment record to both physical slots as required by the firmware. The
complete physical slot/action reference is in
[mini60-layout.md](mini60-layout.md).

## Raw sections

Advanced workflows can read exact buffers as hex or binary files:

```sh
aula section read rapid-trigger --output rapid-trigger.bin
aula section write rapid-trigger --file rapid-trigger.bin
aula section write rapid-trigger --file rapid-trigger.bin --apply
```

Available sections are `device-info`, `settings`, `keymap`, `lighting`,
`per-key-lighting`, `macros`, `fn-keymap`, `rapid-trigger`, and `dks`.
`device-info` is read-only. Exact size checks, dry-run behavior, automatic
backup, and readback verification still apply to raw writes.

## Control commands

```sh
# Both lines are required to deliberately start calibration.
aula calibration start --apply --confirm CALIBRATE
# Fully press every key, then save and leave calibration mode.
aula calibration save --apply

aula key-test start --apply
aula key-test stop --apply

aula reset keys --apply --confirm "RESET KEYS"
aula reset all --apply --confirm "RESET ALL"
```

Calibration start and reset make backups first. These firmware control
commands do not have the same readback acknowledgement as ordinary section
writes, which is why they require explicit invocation and, for destructive
operations, an exact confirmation phrase.
