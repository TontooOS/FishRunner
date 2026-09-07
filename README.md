# FishRunner

FishRunner is the TontooOS app runner. Its `tapp` binary launches `.app`
bundles built with [TBuild](https://github.com/TontooOS/TBuild), both
extracted directory bundles and zipped `.app` files.

## Usage

```bash
cargo build --release

# run an installed app bundle
./target/release/tapp /Users/arlo1/Applications/Steam.app

# run a zipped .app file directly
./target/release/tapp Downloads/Demo.app

# pass arguments to the app after --
./target/release/tapp Demo.app -- --verbose
```

## Made for TontooOS

Explore more at https://github.com/TontooOS/TontooOS

## License

TCL v26.1
