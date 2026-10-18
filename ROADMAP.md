# Roadmap

What's planned for slot-flip, roughly in order. This is a hobby project, so
nothing here is a promise — but it's the honest current direction.

Want to add something? See **[Requesting a feature](#requesting-a-feature)**
at the bottom. Want to build one? Check
**[CONTRIBUTING.md](CONTRIBUTING.md)**, and comment on (or open) the issue.

Legend: ✅ done · 🚧 in progress · 📋 planned · 💡 wanted / needs discussion

---

## Stable — Miyoo Flip (`my355`)

The Flip build is the supported release. Work here is mostly maintenance.

- ✅ SDL2 + GLES2 display, Flip button map, lid sleep, power LED
- ✅ Remappable buttons (`buttons.ini` + in-app screen)
- ✅ Pinned/random wallpapers, sleep timeout, auto power-off
- ✅ Synced to upstream slot **v1.6.0** (plus the v1.7 gpSP JIT fix)
- 📋 Upstream **v1.7.0** (skeuomorphic redesign, favorites/recents, in-game menu) — a deliberate decision, not a quick merge. See [#11](https://github.com/boomchu-123/slot-flip/issues/11)
- 💡 Cartridge-label artwork: fetch real label images automatically (blocked
  on a ScreenScraper developer ID; currently you supply your own labels)

---

## Experimental — other Miyoo devices

These live on the `experimental` branch and ship as `exp-v*` **pre-releases**.
They stay there until verified on hardware.

### Miyoo A30 (`my282`)

Has a Mali-400 GPU, so it reuses the existing GLES renderer.

- ✅ armhf build, pak, launch script, input map (best guess)
- 🚧 **Verify on hardware** — untested; needs a tester's `slot.log`
- 📋 Confirm/correct the button map from `input-trace.log`
- 📋 Bundle armhf emulator cores
- 💡 Shader tweaks if Mali-400's older GLES2 misbehaves

### Miyoo Mini Plus (`miyoomini`)

No GPU, so it needs a software renderer.

- ✅ armhf build, pak, launch script, hardware/framebuffer probe
- 🚧 **Software compositor** — CPU rasteriser that consumes the same `Draw`
  list as the GLES pipeline and blits to `/dev/fb0` (RGB565)
- 📋 Wire the renderer into the frontend and verify the shelf draws
- 📋 Bundle armhf emulator cores
- 📋 Confirm the button map from `input-trace.log`

---

## Wanted — needs discussion

Ideas that would be nice, but aren't committed. Open an issue to push one
forward or volunteer.

- 💡 **More MinUI devices** — e.g. Anbernic RG35XX/RG35XX Plus, TrimUI Smart
  Pro / Brick, Powkiddy RGB30. Each is its own SoC + display path, so it's a
  real port, not a repackage. If a device has a GPU, it's usually much cheaper
  (see the A30). Device support table is in the [README](README.md).
- 💡 **NextUI** — a MinUI-based custom OS. Because it keeps MinUI's
  `Tools/<platform>/*.pak` model (and the same `my355` slug on the Flip),
  Slot's Flip pak is expected to be **drop-in with no code changes** — it just
  needs someone on NextUI to verify. See
  [#7](https://github.com/boomchu-123/slot-flip/issues/7).
- 💡 **Theme presets** — ship a few ready-made `theme.txt` looks.
- 💡 **More wallpaper options** in-app.
- 💡 **Docs**: short video/GIF walkthroughs per feature.

---

## Not planned

To set expectations:

- ❌ **Non-MinUI installs** (OnionOS, stock Miyoo firmware, ArkOS…). Slot is a
  MinUI Tool; other launchers need their own integration.
- ❌ **Bundling or sourcing ROMs/BIOS.** Bring your own legally dumped files.
- ❌ **Non-Miyoo hardware with no GPU** (framebuffer-only) beyond the Mini Plus
  case, unless someone wants to own the port.

---

## Requesting a feature

1. **Check this roadmap and the open issues** — it may already be listed.
2. Open a **[feature request](.github/ISSUE_TEMPLATE/feature_request.yml)**.
   Say what you want, why, and which device it's for.
3. If you want to build it, say so in the issue — that moves it up the list.
   Device ports especially need someone who can test on the hardware.

Feature requests are triaged with these labels:

| Label | Meaning |
| --- | --- |
| `enhancement` | A new feature or improvement |
| `help wanted` | Up for grabs — I'd welcome a PR |
| `good first issue` | Approachable for a newcomer |
| `device: a30` / `device: mini plus` | Specific to an experimental device |
| `porting` | Adding/adapting a device |
| `wontfix` | Out of scope (see *Not planned*) |
