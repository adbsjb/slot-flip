mod draw;
mod fbdev;
mod fbo;
mod gl;
mod grade;
#[cfg(target_os = "macos")]
mod headless;
#[cfg(feature = "host")]
mod host;
mod lcd3x;
mod pipeline;
mod power;
mod quad;
#[cfg(feature = "flip")]
mod sdl;
mod shaders;
mod surface;

pub use draw::{Draw, TexId};
pub use fbdev::{egl_error, panel_mode, panel_size, FbdevSurface};
pub use fbo::{Compositor, BACKDROP};
pub use grade::{blue_light_gain, BLUE_LIGHT_MAX};
#[cfg(target_os = "macos")]
pub use headless::HeadlessSurface;
#[cfg(feature = "host")]
pub use host::HostSurface;
pub use gl::load as load_gl;
pub use lcd3x::{lcd3x_mask, mask_texture_rgba8};
pub use pipeline::{SCALE, SRC_H, SRC_W, WHOLE_TEXTURE};
pub use power::{screen_brightness, screen_scale, screen_width};
#[cfg(feature = "flip")]
pub use sdl::SdlSurface;
pub use surface::{blit_rect, blit_rect_fit, fit_rect, fit_scale, GfxError, Surface, OUT_H, OUT_W};

#[cfg(feature = "flip")]
pub fn tex_test() -> bool {
    use std::sync::OnceLock;
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("SLOT_TEXTEST").is_some())
}
