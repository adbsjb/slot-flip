use std::path::{Path, PathBuf};

use slot_store::ini;

/// Settings file holding the user's general Slot preferences.
pub const SETTINGS_FILE: &str = "Config/slot.ini";
/// Key in SETTINGS_FILE naming the background image to use.
pub const BACKGROUND_KEY: &str = "background";

fn is_png(p: &Path) -> bool {
    !slot_store::is_hidden(p) && p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png"))
}

fn all(root: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(root.join("Wallpapers"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| is_png(p))
        .collect();
    files.sort();
    files
}

/// Pick the background image to show.
///
/// If `Config/slot.ini` names a `background` (a filename under `Wallpapers/`),
/// that image is used. The special value `random` (or a missing key) falls back
/// to the seeded random pick so the old behaviour is preserved. The seed only
/// matters for the random case.
pub fn pick(root: &Path, seed: u64) -> Option<PathBuf> {
    if let Some(choice) = ini::value(root, SETTINGS_FILE, BACKGROUND_KEY) {
        let choice = choice.trim();
        if !choice.is_empty() && !choice.eq_ignore_ascii_case("random") {
            let named = root.join("Wallpapers").join(choice);
            if named.is_file() {
                return Some(named);
            }
            // Named file is gone: fall through to a random pick rather than
            // showing nothing.
        }
    }
    let files = all(root);
    if files.is_empty() {
        return None;
    }
    let i = (seed % files.len() as u64) as usize;
    files.into_iter().nth(i)
}

/// List the available background filenames (just the file names, not full
/// paths) so a settings menu can offer them.
pub fn choices(root: &Path) -> Vec<String> {
    all(root)
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect()
}
