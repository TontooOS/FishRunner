use crate::i18n::{self, Lang};
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const INFO_FILE: &str = "Info.tontoo";
const BIN_DIR_NAME: &str = "App";
const RESOURCES_DIR_NAME: &str = "Resources";
const NON_EXECUTABLE_SUFFIXES: [&str; 8] = ["png", "jpg", "jpeg", "json", "txt", "md", "icns", "tontoo"];

pub struct Bundle {
    pub dir: PathBuf,
    pub display_name: String,
    pub bundle_id: String,
    pub version: String,
    pub resources: PathBuf,
    pub binary: PathBuf,
    temp_dir: Option<PathBuf>,
}

impl Drop for Bundle {
    fn drop(&mut self) {
        if let Some(temp) = &self.temp_dir {
            let _ = fs::remove_dir_all(temp);
        }
    }
}

pub fn open(arg: &Path, lang: &Lang) -> Result<Bundle, String> {
    let resolved = resolve(arg).ok_or_else(|| {
        i18n::t(lang, "err.path_not_found", &[("path", &arg.to_string_lossy())])
    })?;

    if resolved.is_dir() {
        open_dir(resolved, None, lang)
    } else if resolved.is_file() {
        let (dir, temp_dir) = extract_zip(&resolved, lang)?;
        open_dir(dir, Some(temp_dir), lang)
    } else {
        Err(i18n::t(lang, "err.path_not_found", &[("path", &arg.to_string_lossy())]))
    }
}

/// Accepts an exact path or a path without the `.app` suffix.
fn resolve(arg: &Path) -> Option<PathBuf> {
    if arg.exists() {
        return arg.canonicalize().ok();
    }
    let mut with_suffix = arg.as_os_str().to_os_string();
    with_suffix.push(".app");
    let candidate = PathBuf::from(with_suffix);
    if candidate.exists() {
        return candidate.canonicalize().ok();
    }
    None
}

fn open_dir(dir: PathBuf, temp_dir: Option<PathBuf>, lang: &Lang) -> Result<Bundle, String> {
    let info_path = dir.join(INFO_FILE);
    if !info_path.is_file() {
        return Err(i18n::t(lang, "err.not_bundle", &[("path", &dir.to_string_lossy())]));
    }
    let info: Value = serde_json::from_str(
        &fs::read_to_string(&info_path)
            .map_err(|_| i18n::t(lang, "err.read_info", &[("path", &info_path.to_string_lossy())]))?,
    )
    .map_err(|_| i18n::t(lang, "err.parse_info", &[("path", &dir.to_string_lossy())]))?;

    let bundle_id = info
        .get("bundle_id")
        .and_then(Value::as_str)
        .ok_or_else(|| i18n::t(lang, "err.missing_field", &[("field", "bundle_id")]))?
        .to_string();
    let version = info
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| i18n::t(lang, "err.missing_field", &[("field", "version")]))?
        .to_string();

    let display_name = display_name(&dir, &info);
    let bin_dir = dir.join(BIN_DIR_NAME);
    let binary = find_binary(&bin_dir, lang)?;
    make_executable(&binary);

    Ok(Bundle {
        resources: dir.join(RESOURCES_DIR_NAME),
        dir,
        display_name,
        bundle_id,
        version,
        binary,
        temp_dir,
    })
}

fn display_name(dir: &Path, info: &Value) -> String {
    let fallback = dir
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "app".to_string());
    match info.get("name") {
        Some(Value::String(name)) => name.clone(),
        Some(Value::Object(names)) => {
            let locale = i18n::detect_locale();
            names
                .get(locale)
                .or_else(|| names.get("en_us"))
                .or_else(|| names.values().next())
                .and_then(Value::as_str)
                .map(|s| s.to_string())
                .unwrap_or(fallback)
        }
        _ => fallback,
    }
}

fn find_binary(bin_dir: &Path, lang: &Lang) -> Result<PathBuf, String> {
    if !bin_dir.is_dir() {
        return Err(i18n::t(
            lang,
            "err.no_binary",
            &[("path", &bin_dir.to_string_lossy())],
        ));
    }

    let mut candidates: Vec<PathBuf> = Vec::new();
    for entry in WalkDir::new(bin_dir) {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if name.starts_with('.') || name == INFO_FILE || name == "icon.png" {
            continue;
        }
        let skip = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .map(|e| NON_EXECUTABLE_SUFFIXES.contains(&e.as_str()))
            .unwrap_or(false);
        if skip {
            continue;
        }
        candidates.push(path.to_path_buf());
    }

    let executable = candidates
        .iter()
        .find(|path| is_executable(path))
        .cloned();
    if let Some(binary) = executable {
        return Ok(binary);
    }
    if candidates.len() == 1 {
        return Ok(candidates.remove(0));
    }
    Err(i18n::t(lang, "err.no_binary", &[("path", &bin_dir.to_string_lossy())]))
}

fn is_executable(path: &Path) -> bool {
    fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

fn make_executable(path: &Path) {
    if let Ok(mut perms) = fs::metadata(path).map(|m| m.permissions()) {
        perms.set_mode(0o755);
        let _ = fs::set_permissions(path, perms);
    }
}

/// Extracts a zipped `.app` file (as produced by TBuild) into a temp directory.
fn extract_zip(zip_path: &Path, lang: &Lang) -> Result<(PathBuf, PathBuf), String> {
    let staging = std::env::temp_dir().join(format!("tapp-{}", std::process::id()));
    let _ = fs::remove_dir_all(&staging);
    fs::create_dir_all(&staging).map_err(|e| {
        i18n::t(
            lang,
            "err.extract",
            &[("path", &zip_path.to_string_lossy()), ("error", &e.to_string())],
        )
    })?;

    let bytes = fs::read(zip_path)
        .map_err(|e| {
            i18n::t(
                lang,
                "err.extract",
                &[("path", &zip_path.to_string_lossy()), ("error", &e.to_string())],
            )
        })?;
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).map_err(|e| {
        i18n::t(
            lang,
            "err.extract",
            &[("path", &zip_path.to_string_lossy()), ("error", &e.to_string())],
        )
    })?;
    archive.extract(&staging).map_err(|e| {
        i18n::t(
            lang,
            "err.extract",
            &[("path", &zip_path.to_string_lossy()), ("error", &e.to_string())],
        )
    })?;

    // TBuild zips contain a single top-level "<Name>.app" directory.
    for entry in fs::read_dir(&staging).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.is_dir()
            && path
                .file_name()
                .map(|n| n.to_string_lossy().ends_with(".app"))
                .unwrap_or(false)
            && path.join(INFO_FILE).is_file()
        {
            return Ok((path, staging));
        }
    }

    if staging.join(INFO_FILE).is_file() {
        return Ok((staging.clone(), staging));
    }

    let _ = fs::remove_dir_all(&staging);
    Err(i18n::t(lang, "err.not_bundle", &[("path", &zip_path.to_string_lossy())]))
}
