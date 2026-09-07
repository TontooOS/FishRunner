use clap::Parser;
use std::path::PathBuf;
use std::process::exit;

mod bundle;
mod i18n;
mod launch;

#[derive(Parser)]
#[command(name = "tapp", version, about = "TontooOS app runner")]
struct Args {
    /// Path to an installed .app bundle (directory or zipped .app file)
    #[arg(value_name = "APP")]
    app_path: PathBuf,

    /// Arguments passed through to the app (after --)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    passthrough: Vec<String>,
}

fn main() {
    let args = Args::parse();
    let lang = i18n::load();

    let opened = match bundle::open(&args.app_path, &lang) {
        Ok(bundle) => bundle,
        Err(err) => {
            eprintln!("tapp: {}", err);
            exit(1);
        }
    };

    eprintln!(
        "{}",
        i18n::t(&lang, "msg.launching", &[("name", &opened.display_name)])
    );

    let code = match launch::launch(&opened, &args.passthrough, &lang) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("tapp: {}", err);
            exit(1);
        }
    };

    eprintln!(
        "{}",
        i18n::t(
            &lang,
            "msg.exited",
            &[("name", &opened.display_name), ("code", &code.to_string())]
        )
    );
    drop(opened);
    exit(code);
}
