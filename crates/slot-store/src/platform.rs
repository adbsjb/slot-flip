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

/// Which buttons the in-game Controls page offers for a system.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pad {
    /// Everything the GBA has: shoulders as well as the D-pad and face buttons.
    Full,
    /// A D-pad, A, B, Select, Start and the turbo keys.
    Compact,
}

/// Where a game's title comes from before its file name is used.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Header {
    Gba,
    GameBoy,
    /// Nothing worth reading: the shelf uses the file name.
    None,
}

/// Everything about a system that is a fact rather than behaviour. Adding a system
/// means a [`Platform`] variant and one of these; the rest of the store reads it
/// from here instead of matching on the platform itself.
#[derive(Debug)]
pub struct Spec {
    pub name: &'static str,
    pub dir: &'static str,
    pub extensions: &'static [&'static str],
    pub picture: (u32, u32),
    /// The picture is handed over at the size the core made it, rather than centred
    /// on the GBA-sized canvas the Game Boy family is drawn on. Such a system is a
    /// console, with no handheld LCD for a shader, colour correction or bezel to act on.
    pub native_video: bool,
    /// Cores that can run it, the first being the default.
    pub cores: &'static [crate::Core],
    pub pad: Pad,
    pub header: Header,
    /// Its `.m3u` playlists list discs that should not be shelved on their own.
    pub playlists: bool,
}

const GBA: Spec = Spec {
    name: "Game Boy Advance",
    dir: "GBA",
    extensions: &["gba"],
    picture: (240, 160),
    native_video: false,
    cores: &[crate::Core::Mgba, crate::Core::Gpsp],
    pad: Pad::Full,
    header: Header::Gba,
    playlists: false,
};

const GB: Spec = Spec {
    name: "Game Boy",
    dir: "GB",
    extensions: &["gb", "gbc"],
    picture: (160, 144),
    native_video: false,
    cores: &[crate::Core::Mgba],
    pad: Pad::Compact,
    header: Header::GameBoy,
    playlists: false,
};

const GBC: Spec = Spec {
    name: "Game Boy Color",
    dir: "GBC",
    ..GB
};

const PS1: Spec = Spec {
    name: "PlayStation",
    dir: "PS1",
    extensions: &["cue", "chd", "m3u", "pbp"],
    picture: (320, 240),
    native_video: true,
    cores: &[crate::Core::Pcsx],
    // The PlayStation pad has more buttons than the controls page names (X/Y,
    // L2/R2, the stick clicks); until it grows them it shares the GBA set, which
    // is the widest it knows.
    pad: Pad::Full,
    header: Header::None,
    playlists: true,
};

const NES: Spec = Spec {
    name: "Nintendo Entertainment System",
    dir: "NES",
    extensions: &["nes", "fds", "unf", "unif"],
    picture: (256, 240),
    native_video: true,
    cores: &[crate::Core::Fceumm],
    // A D-pad, A, B, Select and Start; the turbo keys pulse A and B just the same.
    pad: Pad::Compact,
    header: Header::None,
    playlists: false,
};

impl Platform {
    pub const ALL: [Platform; 5] = [
        Platform::Gba,
        Platform::Gb,
        Platform::Gbc,
        Platform::Ps1,
        Platform::Nes,
    ];

    pub fn spec(self) -> &'static Spec {
        match self {
            Platform::Gba => &GBA,
            Platform::Gb => &GB,
            Platform::Gbc => &GBC,
            Platform::Ps1 => &PS1,
            Platform::Nes => &NES,
        }
    }

    pub fn name(self) -> &'static str {
        self.spec().name
    }

    pub fn dir_name(self) -> &'static str {
        self.spec().dir
    }

    pub fn extensions(self) -> &'static [&'static str] {
        self.spec().extensions
    }

    pub fn picture(self) -> (u32, u32) {
        self.spec().picture
    }

    pub fn native_video(self) -> bool {
        self.spec().native_video
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
