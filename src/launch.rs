use crate::bundle::Bundle;
use crate::i18n::{self, Lang};
use std::path::Path;
use std::process::Command;

/// When FishBox is installed, apps run behind the freeze-and-ask sandbox
/// supervisor; the filter is inherited by the whole app process tree.
const FISHBOX: &str = "/usr/bin/fishbox";

pub fn launch(bundle: &Bundle, args: &[String], lang: &Lang) -> Result<i32, String> {
    let sandboxed = Path::new(FISHBOX).is_file();

    let mut command = if sandboxed {
        let mut c = Command::new(FISHBOX);
        c.arg("--");
        c.arg(&bundle.binary);
        c
    } else {
        Command::new(&bundle.binary)
    };

    let status = command
        .args(args)
        .current_dir(&bundle.dir)
        .env("TONTOO_APP_BUNDLE_ID", &bundle.bundle_id)
        .env("TONTOO_APP_NAME", &bundle.display_name)
        .env("TONTOO_APP_VERSION", &bundle.version)
        .env("TONTOO_APP_PATH", &bundle.dir)
        .env("TONTOO_APP_RESOURCES", &bundle.resources)
        .status()
        .map_err(|e| {
            i18n::t(
                lang,
                "err.launch",
                &[("path", &bundle.binary.to_string_lossy()), ("error", &e.to_string())],
            )
        })?;
    Ok(status.code().unwrap_or(1))
}
