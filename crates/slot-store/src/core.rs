use std::collections::HashMap;
use std::path::Path;

use crate::Platform;

pub const SELECTED_CORE_FILE: &str = "Config/selected_core.txt";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Core {
    #[default]
    Mgba,
    Gpsp,
    Pcsx,
    Fceumm,
    Snes9x,
}

impl Core {
    /// The cores the picker board offers a choice between. A PlayStation disc has
    /// one core and nothing to pick, so `Pcsx` is not here; see [`Core::EVERY`].
    pub const ALL: [Core; 2] = [Core::Mgba, Core::Gpsp];

    /// Every core slot can run, pickable or not.
    pub const EVERY: [Core; 5] = [
        Core::Mgba,
        Core::Gpsp,
        Core::Pcsx,
        Core::Fceumm,
        Core::Snes9x,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Core::Mgba => "mgba",
            Core::Gpsp => "gpsp",
            Core::Pcsx => "pcsx",
            Core::Fceumm => "fceumm",
            Core::Snes9x => "snes9x",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn text(self) -> &'static str {
        match self {
            Core::Mgba => "mGBA",
            Core::Gpsp => "gpSP",
            Core::Pcsx => "PCSX-ReARMed",
            Core::Fceumm => "fceumm",
            Core::Snes9x => "Snes9x 2005 Plus",
        }
    }

    pub fn parse(s: &str) -> Option<Core> {
        match s.trim().to_ascii_lowercase().as_str() {
            "mgba" => Some(Core::Mgba),
            "gpsp" => Some(Core::Gpsp),
            "pcsx" => Some(Core::Pcsx),
            "fceumm" => Some(Core::Fceumm),
            "snes9x" => Some(Core::Snes9x),
            _ => None,
        }
    }

    /// Whether the core is left out of the main download. Its licence does not allow
    /// selling, so it ships as a separate add-on pack that the player installs, and
    /// slot says so when it is not there instead of failing.
    pub fn is_addon(self) -> bool {
        matches!(self, Core::Snes9x)
    }

    pub fn runs(self, platform: Platform) -> bool {
        platform.spec().cores.contains(&self)
    }

    pub fn default_for(platform: Platform) -> Core {
        // On the Flip, mGBA has no ARM64 dynarec and runs as an interpreter,
        // far too slowly for GBA games. gpSP has a dynarec and runs ~20x
        // faster, so GBA defaults to it there. GB/GBC still use mGBA.
        #[cfg(feature = "flip")]
        if platform == Platform::Gba {
            return Core::Gpsp;
        }
        platform.spec().cores[0]
    }
}

pub fn read_selected_cores(root: &Path) -> HashMap<String, Core> {
    crate::ini::read(root, SELECTED_CORE_FILE)
        .into_iter()
        .filter_map(|(stem, name)| Core::parse(&name).map(|core| (stem, core)))
        .collect()
}

pub fn core_for(root: &Path, stem: &str) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .unwrap_or_default()
}

pub fn core_for_platform(root: &Path, stem: &str, platform: Platform) -> Core {
    crate::ini::value(root, SELECTED_CORE_FILE, stem)
        .as_deref()
        .and_then(Core::parse)
        .filter(|core| core.runs(platform))
        .unwrap_or_else(|| Core::default_for(platform))
}

pub fn write_selected_core(root: &Path, stem: &str, core: Core) -> std::io::Result<()> {
    crate::ini::write(root, SELECTED_CORE_FILE, stem, core.as_str())
}
