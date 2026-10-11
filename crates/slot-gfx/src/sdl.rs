//! SDL2-backed GLES2 surface for devices whose display is driven through
//! KMSDRM + Mali rather than the fake-fbdev EGL window `fbdev.rs` assumes.
//!
//! Like the EGL and ALSA backends elsewhere in slot, libSDL2 is pulled in at
//! runtime with `libloading`, so nothing new is linked at build time. We only
//! use SDL for window + GL context creation and buffer swaps; the whole slot
//! GL pipeline renders on top of the context unchanged.

use std::ffi::{c_char, c_int, c_void, CStr, CString};

use libloading::Library;

use crate::surface::{GfxError, Surface};

// SDL_InitSubSystem flags.
const SDL_INIT_VIDEO: u32 = 0x0000_0020;

// SDL_WindowFlags.
const SDL_WINDOW_OPENGL: u32 = 0x0000_0002;
const SDL_WINDOW_SHOWN: u32 = 0x0000_0004;
const SDL_WINDOW_FULLSCREEN_DESKTOP: u32 = 0x0000_1001;
const SDL_WINDOWPOS_UNDEFINED: c_int = 0x1FFF_0000u32 as c_int;

// SDL_GLattr.
const SDL_GL_RED_SIZE: c_int = 0;
const SDL_GL_GREEN_SIZE: c_int = 1;
const SDL_GL_BLUE_SIZE: c_int = 2;
const SDL_GL_DEPTH_SIZE: c_int = 6;
const SDL_GL_DOUBLEBUFFER: c_int = 5;
const SDL_GL_CONTEXT_MAJOR_VERSION: c_int = 17;
const SDL_GL_CONTEXT_MINOR_VERSION: c_int = 18;
const SDL_GL_CONTEXT_PROFILE_MASK: c_int = 21;
const SDL_GL_CONTEXT_PROFILE_ES: c_int = 0x0004;

type Ptr = *mut c_void;

type InitSubSystem = unsafe extern "C" fn(u32) -> c_int;
type GetError = unsafe extern "C" fn() -> *const c_char;
type GlSetAttribute = unsafe extern "C" fn(c_int, c_int) -> c_int;
type CreateWindow =
    unsafe extern "C" fn(*const c_char, c_int, c_int, c_int, c_int, u32) -> Ptr;
type GlCreateContext = unsafe extern "C" fn(Ptr) -> Ptr;
type GlMakeCurrent = unsafe extern "C" fn(Ptr, Ptr) -> c_int;
type GlSetSwapInterval = unsafe extern "C" fn(c_int) -> c_int;
type GlSwapWindow = unsafe extern "C" fn(Ptr);
type GlGetProcAddress = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type GlGetDrawableSize = unsafe extern "C" fn(Ptr, *mut c_int, *mut c_int);
type ShowCursor = unsafe extern "C" fn(c_int) -> c_int;
type GetCurrentVideoDriver = unsafe extern "C" fn() -> *const c_char;
type GlDeleteContext = unsafe extern "C" fn(Ptr);
type DestroyWindow = unsafe extern "C" fn(Ptr);

struct Sdl {
    init_subsystem: InitSubSystem,
    get_error: GetError,
    gl_set_attribute: GlSetAttribute,
    create_window: CreateWindow,
    gl_create_context: GlCreateContext,
    gl_make_current: GlMakeCurrent,
    gl_set_swap_interval: GlSetSwapInterval,
    gl_swap_window: GlSwapWindow,
    gl_get_proc_address: GlGetProcAddress,
    gl_get_drawable_size: GlGetDrawableSize,
    show_cursor: ShowCursor,
    get_video_driver: GetCurrentVideoDriver,
    gl_delete_context: GlDeleteContext,
    destroy_window: DestroyWindow,
    _lib: Library,
}

fn open(name: &str) -> Result<Library, GfxError> {
    unsafe { Library::new(name) }.map_err(|e| GfxError::Context(format!("{name}: {e}")))
}

unsafe fn sym<T: Copy>(lib: &Library, name: &str) -> Result<T, GfxError> {
    lib.get::<T>(name.as_bytes())
        .map(|s| *s)
        .map_err(|e| GfxError::Context(format!("{name}: {e}")))
}

impl Sdl {
    fn load() -> Result<Self, GfxError> {
        let lib = open("libSDL2-2.0.so.0")
            .or_else(|_| open("libSDL2-2.0.so"))
            .or_else(|_| open("libSDL2.so"))?;
        unsafe {
            Ok(Sdl {
                init_subsystem: sym(&lib, "SDL_InitSubSystem")?,
                get_error: sym(&lib, "SDL_GetError")?,
                gl_set_attribute: sym(&lib, "SDL_GL_SetAttribute")?,
                create_window: sym(&lib, "SDL_CreateWindow")?,
                gl_create_context: sym(&lib, "SDL_GL_CreateContext")?,
                gl_make_current: sym(&lib, "SDL_GL_MakeCurrent")?,
                gl_set_swap_interval: sym(&lib, "SDL_GL_SetSwapInterval")?,
                gl_swap_window: sym(&lib, "SDL_GL_SwapWindow")?,
                gl_get_proc_address: sym(&lib, "SDL_GL_GetProcAddress")?,
                gl_get_drawable_size: sym(&lib, "SDL_GL_GetDrawableSize")?,
                show_cursor: sym(&lib, "SDL_ShowCursor")?,
                get_video_driver: sym(&lib, "SDL_GetCurrentVideoDriver")?,
                gl_delete_context: sym(&lib, "SDL_GL_DeleteContext")?,
                destroy_window: sym(&lib, "SDL_DestroyWindow")?,
                _lib: lib,
            })
        }
    }

    fn fail(&self, what: &str) -> GfxError {
        let msg = unsafe {
            let p = (self.get_error)();
            if p.is_null() {
                String::new()
            } else {
                CStr::from_ptr(p).to_string_lossy().into_owned()
            }
        };
        GfxError::Context(format!("{what}: {msg}"))
    }
}

pub struct SdlSurface {
    sdl: Sdl,
    window: Ptr,
    context: Ptr,
    size: (u32, u32),
}

impl SdlSurface {
    pub fn new() -> Result<Self, GfxError> {
        let sdl = Sdl::load()?;
        unsafe {
            if (sdl.init_subsystem)(SDL_INIT_VIDEO) != 0 {
                return Err(sdl.fail("SDL_InitSubSystem(VIDEO)"));
            }
            (sdl.show_cursor)(0);

            (sdl.gl_set_attribute)(SDL_GL_CONTEXT_PROFILE_MASK, SDL_GL_CONTEXT_PROFILE_ES);
            (sdl.gl_set_attribute)(SDL_GL_CONTEXT_MAJOR_VERSION, 2);
            (sdl.gl_set_attribute)(SDL_GL_CONTEXT_MINOR_VERSION, 0);
            (sdl.gl_set_attribute)(SDL_GL_RED_SIZE, 8);
            (sdl.gl_set_attribute)(SDL_GL_GREEN_SIZE, 8);
            (sdl.gl_set_attribute)(SDL_GL_BLUE_SIZE, 8);
            (sdl.gl_set_attribute)(SDL_GL_DEPTH_SIZE, 0);
            (sdl.gl_set_attribute)(SDL_GL_DOUBLEBUFFER, 1);

            // Fullscreen desktop: SDL takes the panel's native mode, which on
            // the Flip is the fixed 640x480 DSI. A zero width/height here lets
            // SDL fill whatever the current display mode reports.
            let title = CString::new("slot").unwrap();
            let flags = SDL_WINDOW_OPENGL | SDL_WINDOW_SHOWN | SDL_WINDOW_FULLSCREEN_DESKTOP;
            let window = (sdl.create_window)(
                title.as_ptr(),
                SDL_WINDOWPOS_UNDEFINED,
                SDL_WINDOWPOS_UNDEFINED,
                0,
                0,
                flags,
            );
            if window.is_null() {
                return Err(sdl.fail("SDL_CreateWindow"));
            }

            let context = (sdl.gl_create_context)(window);
            if context.is_null() {
                (sdl.destroy_window)(window);
                return Err(sdl.fail("SDL_GL_CreateContext"));
            }
            if (sdl.gl_make_current)(window, context) != 0 {
                (sdl.gl_delete_context)(context);
                (sdl.destroy_window)(window);
                return Err(sdl.fail("SDL_GL_MakeCurrent"));
            }

            // Prefer vsync; fall back to immediate if the driver refuses it.
            let vsync = (sdl.gl_set_swap_interval)(1);
            if vsync != 0 {
                let _ = (sdl.gl_set_swap_interval)(0);
            }
            let driver = {
                let p = (sdl.get_video_driver)();
                if p.is_null() {
                    String::from("(null)")
                } else {
                    CStr::from_ptr(p).to_string_lossy().into_owned()
                }
            };
            eprintln!("slot: sdl video driver {driver}, swap_interval(1) rc={vsync} (0=ok)");

            let (mut w, mut h) = (0, 0);
            (sdl.gl_get_drawable_size)(window, &mut w, &mut h);
            let size = if w > 0 && h > 0 {
                (w as u32, h as u32)
            } else {
                (640, 480)
            };

            Ok(SdlSurface {
                sdl,
                window,
                context,
                size,
            })
        }
    }
}

impl Surface for SdlSurface {
    fn make_current(&mut self) -> Result<(), GfxError> {
        match unsafe { (self.sdl.gl_make_current)(self.window, self.context) } {
            0 => Ok(()),
            _ => Err(self.sdl.fail("SDL_GL_MakeCurrent")),
        }
    }

    fn window_size(&self) -> (u32, u32) {
        self.size
    }

    fn swap(&mut self) -> Result<(), GfxError> {
        unsafe { (self.sdl.gl_swap_window)(self.window) };
        Ok(())
    }

    fn proc_address(&self, name: &str) -> *const c_void {
        let Ok(c) = CString::new(name) else {
            return std::ptr::null();
        };
        unsafe { (self.sdl.gl_get_proc_address)(c.as_ptr()) as *const c_void }
    }
}

impl Drop for SdlSurface {
    fn drop(&mut self) {
        unsafe {
            (self.sdl.gl_delete_context)(self.context);
            (self.sdl.destroy_window)(self.window);
        }
    }
}
