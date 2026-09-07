# FishRunner – Wiki

FishRunner is the TontooOS app runner. The `tapp` binary launches `.app`
bundles built with TBuild, extracted or zipped, and is wired into TontooOS
so that executing any `*.app` file dispatches to it automatically.

- Repository: https://github.com/TontooOS/TontooOS
- License: TCL
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Development and usage rules |
| Tapp | [Tapp.md](Tapp.md) | The `.app` runner binary |

## Quick Start

Build the runner and launch an installed app:

```bash
cargo build --release
./target/release/tapp /Users/arlo1/Applications/Steam.app
```

Launch a zipped `.app` file directly (TBuild marks built bundles executable):

```bash
./target/release/tapp Downloads/Demo.app
```

On TontooOS the `tapp-binfmt` LaunchPad registers a `binfmt_misc` handler,
so both forms also work without typing `tapp`:

```bash
./Demo.app
```

See [Tapp.md](Tapp.md) for details.

## Changelog

- 2026-08-25: Initial wiki, `tapp` runner with bundle resolution, ZIP
  extraction, environment injection and localized messages.
