# Testers wanted — Miyoo Mini Plus (`miyoomini`)

There is an **experimental** Mini Plus build. It is **not working yet** and will
not play games. This page explains exactly where it stands and how you can help
move it forward. I don't own a Mini Plus, so this port depends on logs from
people who do.

## Status

| Part | State |
| --- | --- |
| Builds for the Mini Plus (32-bit ARM / armhf) | ✅ |
| Appears in MinUI **Tools** (`Tools/miyoomini/Slot.pak`) | ✅ expected |
| Button map (D-pad, A/B/X/Y, L/R, Start/Select/Menu) | 🟡 written from MinUI's codes, unverified |
| Reads the framebuffer and writes `slot.log` | ✅ |
| **Draws anything to the screen** | ❌ not implemented |
| Sound, saves, save states | ❌ blocked on the renderer |
| Emulator cores bundled | ❌ needs armhf builds |

### Why it doesn't draw yet

The Flip (RK3566) has a Mali GPU and runs Slot's OpenGL ES renderer. The Mini
Plus uses a SigmaStar SSD202D, which has **no GPU** — just a raw Linux
framebuffer (`/dev/fb0`, typically 640×480 RGB565). Slot's whole renderer is
GLES-based, so the Mini Plus needs a brand-new **software** compositor that
doesn't exist yet. That's the main piece of work left.

This build therefore does the honest thing: it probes the framebuffer, logs what
it sees, and exits — instead of crashing or showing a black screen with no clue
why.

## How to help

1. Install MinUI on your Mini Plus.
2. Download `Slot.pak-miyoomini.zip` from the latest
   [release](https://github.com/boomchu-123/slot-flip/releases), unzip it, and
   copy the `Slot.pak` folder to `<SD CARD>/Tools/miyoomini/Slot.pak`.
3. Boot MinUI, open **Tools → Slot**. It will flash and return to MinUI (that's
   expected right now).
4. Grab the log it wrote: `<SD CARD>/Slot/slot.log`.
5. For the most useful log, create an empty file named `DEBUG` inside the
   `Slot.pak` folder before launching — this also records your button presses to
   `Slot/input-trace.log`. Press every button a couple of times, then quit.
6. Post `slot.log` (and `input-trace.log` if you made one) on the GitHub thread
   or open an issue.

That log tells me the panel geometry and exact pixel format, and the trace tells
me whether the button codes are right — the two things I need to build the
software renderer and confirm input without owning the device.

## What I'm looking for in the log

- The `miyoomini fb0 ...` line: resolution, bpp, pitch, and the R/G/B bit
  offsets (confirms RGB565 vs something else).
- Any `framebuffer probe failed: ...` line (tells me the fb node or ioctl is
  different than expected).
- The `input ...` lines showing which `eventN` nodes were found.

Thanks for helping. This is how the Flip port got finished too — run it, read
the log, fix, repeat.
