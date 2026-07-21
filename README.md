# OpenAula

An open-source, local configurator for supported AULA keyboards.

The project is organized as a Cargo workspace:

- `openaula-core`: device discovery, protocol encoding/decoding, validation,
  and configuration models.
- `openaula-cli`: the command-line frontend.
- `openaula-desktop`: a dependency-free placeholder for a future graphical
  frontend.

Both frontends depend on `openaula-core`; hardware access does not belong in a
frontend crate.

Reverse-engineering documentation:

- [Complete feature map](docs/feature-map.md)
- [HID protocol and state formats](docs/protocol.md)
- [Mini60 key slots and action catalog](docs/mini60-layout.md)

The core crate already contains tested packet framing and codecs for device
info, settings, lighting, key assignments, rapid trigger, and DKS records. The
next safety milestone is a read-only HID backend and `aula dump`; setters should
not land before byte-for-byte backup and readback verification are available.
