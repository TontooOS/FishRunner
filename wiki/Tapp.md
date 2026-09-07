# Tapp

`tapp` is the TontooOS app runner. It opens `.app` bundles built with TBuild,
reads their `Info.tontoo` metadata, locates the executable and starts it with
the correct environment. Installed apps live as directory bundles in the
Applications folders; zipped `.app` files (the download/distribution format)
are supported too and are extracted to a temp directory per run.

## Usage

```bash
tapp <APP> [-- <args>...]
```

| Argument | Description |
|---|---|
| `<APP>` | Path to an installed `.app` bundle (directory or zipped `.app` file) |
| `-- <args>` | Everything after `--` is passed to the launched app |

```bash
# installed directory bundle
tapp /Users/arlo1/Applications/Steam.app

# zipped .app file
tapp Downloads/Demo.app

# pass arguments to the app
tapp Demo.app -- --verbose
```

## Bundle Resolution

`tapp` resolves its argument in this order:

1. Use the path as given when it exists.
2. Otherwise append `.app` and use that path when it exists.
3. `Returns Err` (`err.path_not_found`) when neither exists.

When the resolved path is a **file**, it is treated as a zipped bundle:

- The ZIP is extracted into `$TMPDIR/tapp-<pid>`.
- A top-level directory ending in `.app` that contains `Info.tontoo` is used
  as the bundle root; otherwise the staging directory itself is used when it
  contains `Info.tontoo` directly.
- The staging directory is removed automatically when the runner exits.
- `Returns Err` (`err.extract`, `err.not_bundle`) when extraction or lookup fails.

When the resolved path is a **directory**, it must contain `Info.tontoo`,
otherwise `tapp` returns `err.not_bundle`.

## Info.tontoo

| Field | Type | Description |
|---|---|---|
| `bundle_id` | `string` | Required. Missing field returns `err.missing_field`. |
| `version` | `string` | Required. Missing field returns `err.missing_field`. |
| `name` | `string` or `object` | Optional display name. Objects map locales to names, e.g. `{"en_us": "Demo", "de_de": "DemoDE"}`. |

The display name is picked from the detected system locale, falls back to
`en_us`, then to any entry, then to the file stem of the bundle path.

## Executable Discovery

The binary is searched inside the `App/` directory of the bundle:

1. All regular files are collected, skipping hidden files, `Info.tontoo`,
   `icon.png` and known non-executable suffixes (`png`, `jpg`, `jpeg`, `json`,
   `txt`, `md`, `icns`, `tontoo`).
2. The first candidate with an execute bit set wins.
3. If none has an execute bit but exactly one candidate exists, it is used.
4. Otherwise `tapp` returns `err.no_binary`.

The chosen binary always gets mode `0o755` before launch.

## Launch Behavior

The app runs with its bundle directory as working directory, inheriting
stdin/stdout/stderr. `tapp` waits for it and exits with the same exit code
(a killed app without exit code becomes `1`). These variables are injected:

| Variable | Value |
|---|---|
| `TONTOO_APP_BUNDLE_ID` | `bundle_id` from `Info.tontoo` |
| `TONTOO_APP_NAME` | Display name of the bundle |
| `TONTOO_APP_VERSION` | `version` from `Info.tontoo` |
| `TONTOO_APP_PATH` | Absolute path of the bundle root |
| `TONTOO_APP_RESOURCES` | Absolute path of the `Resources/` directory |

## Messages

All user-facing messages are localized via `lang/en_us.json` and
`lang/de_de.json`. The locale is detected from `LANG` / `LC_ALL`, falling back
to `/etc/locale.conf`; anything not starting with `de` uses `en_us`.

## binfmt Integration on TontooOS

TontooOS stages a `binfmt_misc` registration through the `tapp-binfmt`
LaunchPad service (`/usr/local/bin/tapp-binfmt.sh`):

```
:tapp:E::app::/usr/bin/tapp:
```

After registration, executing any file ending in `.app` dispatches to `tapp`
automatically, so `./Steam.app` in a shell works like `tapp Steam.app`.

> **Note:** The kernel only consults `binfmt_misc` for executable files.
> Zipped bundles must carry the execute bit (TBuild marks built `.app` files
> as `0o755`). Extracted directory bundles cannot be executed directly;
> use `tapp <dir>.app` or open them through the desktop integration
> (`xdg-open` / MIME type `application/x-tontoo-app`).

## Cross References

- [MAIN.md](MAIN.md) – overview
- TBuild wiki: `App.md` – how `.app` bundles are built
