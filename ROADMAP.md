# Roadmap

What's planned for slot-flip, roughly in order. This is a hobby project, so
nothing here is a promise — but it's the honest current direction.

Want to add something? See **[Requesting a feature](#requesting-a-feature)**
at the bottom. Want to build one? Check
**[CONTRIBUTING.md](CONTRIBUTING.md)**, and comment on (or open) the issue.

Legend: ✅ done · 🚧 in progress · 📋 planned · 💡 wanted / needs discussion

---

## Milestones

Two releases, on purpose:

| Milestone | What it means |
| --- | --- |
| **1.0** | Every system that can run at full speed on a Miyoo Flip, and PlayStation working end to end. One device, done properly. |
| **2.0** | The platform layer: input, audio, power and display behind one seam per device, so slot runs on other Linux handhelds. Ports stop being a fork of the frontend. |

Keeping them apart is deliberate. Systems are additive and can ship one at a time;
the platform layer rewrites the device seams and cannot. Mixed into one release,
a device port in progress would hold the systems work back, and a systems release
would keep re-shipping half-finished ports.

### 1.0 — all supportable systems, on the Flip

Set by the maintainer: support as many systems as can be supported, and make
PlayStation work properly.

- **[SYSTEMS.md](SYSTEMS.md) is the register**: every system considered, its core,
  its licence, and whether it is bundled, planned or not viable. A core is only
  added after its licence has been read.
- **Batches**, because each needs its own real-core test and hardware check:
  1. **No new core:** Sega CD / Mega CD and SG-1000 on Genesis Plus GX (both just
     need a system table entry, art and a test). Famicom Disk System on fceumm.
  2. **Cheap 2D:** PC Engine, Neo Geo Pocket, WonderSwan, Atari Lynx, Atari 7800,
     Intellivision, Virtual Boy.
  3. **Free-only:** Sega 32X on PicoDrive (non-commercial, like Genesis Plus GX).
  4. **PlayStation short of 1.0:** a game actually asking for its second disc,
     and timing a heavy 3D game.
- **1.0 criteria:** every system in the register that is marked viable has run a
  real game on a Flip; PlayStation has no known failure; the download still passes
  `tools/check-pak-licences.sh`; the README table matches reality.
- **Stay free and open source.** No plans to sell. Donations never unlock
  anything. Non-commercial cores are bundled knowingly in the one free download
  (snes9x2005, Genesis Plus GX, and PicoDrive if it is added), with their notices.
- **Not a goal:** a desktop app. The `host` build is development and render-test
  tooling.

### 2.0 — the platform layer, other handhelds

The second half of "run on as many handhelds as realistic", deliberately after 1.0.

- **One seam per device**: input, audio, power and display, with a backend per
  device instead of `cfg(feature = "flip")` reaching into shared code; the
  frontend, store and UI stop knowing which device they are on.
- **Runtime detection where possible**: probe for the lid, LED, rumble motor, stick
  or battery node; a missing one is a no-op, not an error. Device facts a user can
  fix (key map, paths) live in a config file.
- **A core built per CPU**: aarch64 for the Flip, armhf for the Miyoo A30 and
  Mini Plus, then whatever a new device needs.
- **Each port ships as an experimental pre-release** until someone with the
  hardware sends a clean `slot.log`; only then does it move to stable. The
  existing `experimental` branch and `exp-v*` tags become the 2.0 line.
- **Candidates:** Miyoo A30 (`my282`, Mali-400, reuses the GLES path), Miyoo Mini
  Plus (`miyoomini`, no GPU, needs a CPU compositor), Anbernic RG35XX family,
  TrimUI Smart Pro / Brick, Powkiddy RGB30. Each needs a tester.
- **2.0 criteria:** a second device verified on hardware, the Flip still passing
  its own checks, and no `cfg(feature = ...)` for a device left in shared code.

---

## Stable — Miyoo Flip (`my355`)

The Flip build is the supported release. It tracks upstream slot one release at
a time; the fork's own work is the Flip port itself and a few small additions.

**Done**
- ✅ SDL2 + GLES2 display, Flip button map, lid sleep, power LED, 48 kHz audio
- ✅ Synced to upstream slot **v1.8.0**: the redesigned shelf and menus, in-game
  menu, favorites and recents, in-app wallpaper, bezels, **Controls** (in-game
  buttons, shortcuts and shelf keys, with chords and clash warnings), and the
  new palette and audio fixes
- ✅ **Key Map** under Controls: which physical key each button is, saved to
  `buttons.ini` ([#8](https://github.com/boomchu-123/slot-flip/issues/8))
- ✅ Lid power-off timer (upstream's **General > Lid Power Off**; the old Sleep
  Timeout setting carries over once)
- ✅ Shaders default to Off on the Flip
- ✅ **About shows both versions**: the fork build and the upstream slot release
  it follows, plus a one-time "Updated to ..." word after an update
- ✅ **Cart labels that are forgiving to add**: one flat `Labels/` folder works,
  and a PNG named after the game pairs even when the ROM has region or revision
  tags
- ✅ **Released v0.5.0**: the v1.8.0 sync, Key Map under Controls, and the rebuilt
  mGBA core
- ✅ **Released v0.6.0**: the About plate names the Flip, the one-time update word,
  and the forgiving cart labels

- ✅ **Released v0.7.0**: PlayStation discs. A PS1 shelf, discs that spin into a
  console-style slot with cover art, a PlayStation pad on the Flip's buttons, and
  the PCSX-ReARMed core bundled ([#20](https://github.com/boomchu-123/slot-flip/issues/20))

- ✅ **Released v0.8.0**: NES games on a drawn Game Pak, with fceumm bundled

- ✅ **Released v0.9.0**: Super Nintendo on a drawn cart, snes9x2005 bundled (the download is now free-only)

- ✅ **Released v0.9.1**: the slot is drawn as each console ([#27](https://github.com/boomchu-123/slot-flip/issues/27)), and Lid Power Off gains **Never**

- ✅ **Released v0.10.0**: Sega Mega Drive / Genesis, Master System and Game Gear on drawn black carts, with Genesis Plus GX bundled

- ✅ **Released v0.10.1**: label scans keep their own shape on the NES, SNES and Sega carts ([#30](https://github.com/boomchu-123/slot-flip/issues/30))

- ✅ **Released v0.11.0**: PlayStation memory cards are kept with the other saves, a multi-disc game can change disc from the in-game menu, and the sticks can be passed to a game as a DualShock's (off by default)

**Next**
- 💡 **Card repair on MinUI.** Upstream's SELECT + START repair is switched off
  on the Flip (its script is written for upstream's own devices and unmounts the
  card, which MinUI owns). A safe, tested version for the Flip needs its own
  design; for now a read-only card is repaired from a computer. See
  [#17](https://github.com/boomchu-123/slot-flip/issues/17)
- 💡 Cartridge-label artwork: fetch real label images automatically (blocked
  on a ScreenScraper developer ID; currently you supply your own labels)
- 🚧 **More systems** ([#20](https://github.com/boomchu-123/slot-flip/issues/20)).
  PlayStation is out in v0.7.0 and NES in v0.8.0. PlayStation memory cards are kept and
  multi-disc games can change disc (v0.11.0), and the sticks can go to a game (a
  DualShock, off by default, with the stick clicks as L3/R3). Still to do for
  PlayStation: a game actually asking for its second disc (they tend to ask late),
  and timing a heavy 3D game. Super Nintendo is built (v0.9.0), with a core
  whose licence is non-commercial, so the download as a whole is free-only (see
  `AGENTS.md`, section 9). The Sega systems (Mega Drive / Genesis, Master System, Game
  Gear) are out in v0.10.0. Next: the cheaper 2D systems, one core at a time, each only
  once someone has games for it to try. Nothing here is promised until it runs on
  hardware

**Housekeeping that has bitten before**
- Rebuild the mGBA and gpSP cores whenever a sync changes `cores/` (v0.4.0
  shipped a core missing one of v1.7's patches). `AGENTS.md` has the checklist.

---

## Experimental — other Miyoo devices (the 2.0 line)

These are the start of **2.0**, and stay on the `experimental` branch as `exp-v*`
**pre-releases** until verified on hardware. The stable `flip` branch keeps
shipping 1.x system work in the meantime; nothing here blocks 1.0.

> **Status:** `experimental` is still on upstream **v1.6**, behind the stable
> branch. It needs the v1.8 sync, the Key Map, and the armhf builds re-checked
> before testers get any of it. Until then, treat these ports as stale.

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
