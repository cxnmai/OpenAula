# OpenAula

OpenAula is an open-source, local configurator for the AULA Mini60 HE Pro.
It talks directly to the keyboard's vendor HID configuration interface; no
account, browser, or cloud service is involved.

The repository is a Rust workspace:

- `openaula-core` owns discovery, transport, protocol codecs, validation, and
  lossless configuration backups.
- `openaula-cli` provides the `aula` command.
- `openaula-desktop` is reserved for the later graphical frontend.

## Status

The CLI supports the wired Mini60 HE Pro (`0c45:80a2`) and its 2.4 GHz dongle
(`0c45:fefe`). It can discover the exact configuration endpoint, inspect the
device, make or restore complete JSON backups, edit settings and lighting,
remap normal/Fn keys, tune per-key Hall-effect settings, access raw protocol
sections, run calibration/key-test commands, and reset configuration.

Writes are deliberately conservative:

1. Every setter is a dry-run unless `--apply` is present.
2. Before an applied configuration change, `aula` creates a complete JSON
   backup with exclusive file creation (it never overwrites a backup).
3. It writes the requested section, reads it back, and verifies the result.
4. Calibration and reset require an additional exact confirmation phrase.

Firmware flashing and other unverified screen/music commands are not exposed.

## Install

Rust 1.85 or newer is required.

```sh
cargo install --path crates/openaula-cli
aula --version
```

For development, replace `aula` in the examples with
`cargo run -p openaula-cli --`.

On Linux, the user running `aula` must be able to open the matching
`/dev/hidraw*` node. `aula devices` can enumerate endpoints without opening
them and is useful for diagnosing permissions or connection state.

## Quick start

```sh
# Find supported configuration interfaces.
aula devices

# Read a compact summary, then capture a complete lossless backup.
aula info
aula dump --output mini60-backup.json

# Preview changes. These commands perform reads but do not write.
aula lighting set --effect 0x0b --color ff8000 --brightness 4
aula performance set --slots 49,50,51 --actuation 1.20 --press 0.20 --release 0.20
aula keymap set --layer fn --slot 34 --action key:up

# Apply only after reviewing the preview. A new backup is made automatically.
aula lighting set --effect 0x0b --color ff8000 --brightness 4 --apply

# Validate a backup without opening a keyboard, then restore and verify it.
aula restore mini60-backup.json
aula restore mini60-backup.json --apply
```

Use `--device INDEX`, `--device 0c45:fefe`, or an exact HID path when more
than one supported interface is present. Add `--json` for machine-readable
output where supported.

See [the CLI guide](docs/cli.md) for the full command map, assignment grammar,
and safety behavior.

## Reverse-engineering notes

- [Complete feature map](docs/feature-map.md)
- [HID protocol and state formats](docs/protocol.md)
- [Mini60 key slots and action catalog](docs/mini60-layout.md)

The protocol remains independently reverse-engineered. Keep a known-good
backup before experimenting with raw section writes.

## Development

```sh
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```
