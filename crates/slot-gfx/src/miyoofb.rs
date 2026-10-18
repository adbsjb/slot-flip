//! Experimental Miyoo Mini Plus (miyoomini) framebuffer probe.
//!
//! The Mini Plus uses a SigmaStar SSD202D, which has **no GPU** — there is no
//! GLES/EGL stack to drive, only a raw Linux framebuffer at `/dev/fb0`
//! (typically 640x480 RGB565). The rest of slot's renderer is GLES-based, so a
//! working Mini Plus port needs a software compositor that is not written yet.
//!
//! Until that exists, this module probes the framebuffer with the standard
//! `FBIOGET_VSCREENINFO` / `FBIOGET_FSCREENINFO` ioctls and reports what it
//! finds. The point is to give testers a useful `slot.log` (panel geometry,
//! pixel format, line length) so the software renderer can be built against
//! real numbers instead of guesses. See TESTERS.md.

use std::ffi::{c_int, c_ulong, c_void};
use std::fs::OpenOptions;
use std::os::fd::AsRawFd;

const FB0: &str = "/dev/fb0";

// <linux/fb.h>
const FBIOGET_VSCREENINFO: c_ulong = 0x4600;
const FBIOGET_FSCREENINFO: c_ulong = 0x4602;

extern "C" {
    fn ioctl(fd: c_int, request: c_ulong, ...) -> c_int;
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct FbBitfield {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct FbVarScreeninfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: FbBitfield,
    green: FbBitfield,
    blue: FbBitfield,
    transp: FbBitfield,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    reserved: [u32; 4],
}

#[repr(C)]
struct FbFixScreeninfo {
    id: [u8; 16],
    smem_start: c_ulong,
    smem_len: u32,
    typ: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: c_ulong,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    reserved: [u16; 2],
}

impl Default for FbFixScreeninfo {
    fn default() -> Self {
        // SAFETY: a plain repr(C) struct of integers; all-zero is valid.
        unsafe { std::mem::zeroed() }
    }
}

/// Describes the framebuffer a Mini Plus exposes. Reported to `slot.log`.
pub struct FbInfo {
    pub xres: u32,
    pub yres: u32,
    pub bpp: u32,
    pub line_length: u32,
    pub rgba: [(u32, u32); 4],
    pub id: String,
    pub smem_len: u32,
}

impl FbInfo {
    pub fn report(&self) -> String {
        let [r, g, b, a] = self.rgba;
        format!(
            "miyoomini fb0 \"{}\": {}x{} {}bpp, pitch {} bytes, {} KiB, \
             R@{}/{} G@{}/{} B@{}/{} A@{}/{}",
            self.id,
            self.xres,
            self.yres,
            self.bpp,
            self.line_length,
            self.smem_len / 1024,
            r.0, r.1, g.0, g.1, b.0, b.1, a.0, a.1,
        )
    }
}

/// Probe `/dev/fb0`. Returns the panel geometry and pixel layout, or an error
/// string suitable for logging if the framebuffer cannot be read.
pub fn probe() -> Result<FbInfo, String> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(FB0)
        .map_err(|e| format!("open {FB0}: {e}"))?;
    let fd = file.as_raw_fd();

    let mut var = FbVarScreeninfo::default();
    // SAFETY: fd is a valid framebuffer device; var is the right size for the ioctl.
    let rc = unsafe { ioctl(fd, FBIOGET_VSCREENINFO, &mut var as *mut _ as *mut c_void) };
    if rc != 0 {
        return Err(format!("FBIOGET_VSCREENINFO: {}", std::io::Error::last_os_error()));
    }

    let mut fix = FbFixScreeninfo::default();
    // SAFETY: as above.
    let rc = unsafe { ioctl(fd, FBIOGET_FSCREENINFO, &mut fix as *mut _ as *mut c_void) };
    if rc != 0 {
        return Err(format!("FBIOGET_FSCREENINFO: {}", std::io::Error::last_os_error()));
    }

    let id_end = fix.id.iter().position(|&c| c == 0).unwrap_or(fix.id.len());
    let id = String::from_utf8_lossy(&fix.id[..id_end]).trim().to_string();

    Ok(FbInfo {
        xres: var.xres,
        yres: var.yres,
        bpp: var.bits_per_pixel,
        line_length: fix.line_length,
        rgba: [
            (var.red.offset, var.red.length),
            (var.green.offset, var.green.length),
            (var.blue.offset, var.blue.length),
            (var.transp.offset, var.transp.length),
        ],
        id,
        smem_len: fix.smem_len,
    })
}
