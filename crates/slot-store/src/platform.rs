use std::path::Path;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Platform {
    #[default]
    Gba,
    Gb,
    Gbc,
    Ps1,
    Nes,
}

impl Platform {
    pub const ALL: [Platform; 5] = [
        Platform::Gba,
        Platform::Gb,
        Platform::Gbc,
        Platform::Ps1,
        Platform::Nes,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Platform::Gba => "Game Boy Advance",
            Platform::Gb => "Game Boy",
            Platform::Gbc => "Game Boy Color",
            Platform::Ps1 => "PlayStation",
            Platform::Nes => "Nintendo Entertainment System",
        }
    }

    pub fn dir_name(self) -> &'static str {
        match self {
            Platform::Gba => "GBA",
            Platform::Gb => "GB",
            Platform::Gbc => "GBC",
            Platform::Ps1 => "PS1",
            Platform::Nes => "NES",
        }
    }

    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Platform::Gba => &["gba"],
            Platform::Gb | Platform::Gbc => &["gb", "gbc"],
            Platform::Ps1 => &["cue", "chd", "m3u", "pbp"],
            Platform::Nes => &["nes", "fds", "unf", "unif"],
        }
    }

    pub fn picture(self) -> (u32, u32) {
        match self {
            Platform::Gba => (240, 160),
            Platform::Gb | Platform::Gbc => (160, 144),
            Platform::Ps1 => (320, 240),
            Platform::Nes => (256, 240),
        }
    }

    /// Whether the picture is handed over at the size the core made it, rather than
    /// centred on the GBA-sized canvas the Game Boy family is drawn on.
    pub fn native_video(self) -> bool {
        matches!(self, Platform::Ps1 | Platform::Nes)
    }

    /// Whether the screen shader, colour correction and bezel mean anything for
    /// this system. They are built around a Game Boy's LCD; a disc or a NES picture
    /// has none of that to change.
    pub fn has_screen_options(self) -> bool {
        !self.native_video()
    }

    pub fn accepts(self, path: &Path) -> bool {
        path.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|ext| {
                self.extensions()
                    .iter()
                    .any(|k| ext.eq_ignore_ascii_case(k))
            })
    }
}
