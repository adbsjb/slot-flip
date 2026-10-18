# slot. — Miyoo Flip fork

A cartridge-style Game Boy / Game Boy Color / Game Boy Advance frontend for the
**Miyoo Flip (my355)**, installed as a **MinUI Tool**. Browse your games as
cartridges on a shelf, slot one in, and play — with sound, saves, and save
states.

> **This is a fork of [slot](https://slot-cfw.fyi) by Brandon T. Kowalski**,
> which was originally built for the Anbernic RG SP. This fork ports it to the
> Miyoo Flip's hardware and adds a handful of features (below). All credit for
> the original design and frontend goes upstream — see
> [Credits](#credits--licence). It is licensed **GPL-3.0-or-later**, the same as
> upstream.

It keeps everything it owns in a single `Slot` folder and **never touches your
MinUI Roms or Saves**.

## Device support

| Device | MinUI platform | Pak folder | Status | Download |
| --- | --- | --- | --- | --- |
| **Miyoo Flip** | `my355` | `Tools/my355/Slot.pak` | ✅ **Supported** (stable) | [Latest release](https://github.com/boomchu-123/slot-flip/releases/latest) |
| **Miyoo Mini Plus** | `miyoomini` | `Tools/miyoomini/Slot.pak` | 🧪 **Experimental — not working yet** | [Pre-releases](https://github.com/boomchu-123/slot-flip/releases) |
| Any other MinUI device | — | — | ❌ Not supported | — |

**Why not other devices?** Each MinUI device loads tools only from its own
`Tools/<platform>/` folder and has different hardware (CPU architecture,
display, inputs). The Flip pak will simply **not appear** on a device whose
platform folder doesn't match, and even if copied across it won't run.

**Miyoo Mini Plus is experimental and does not play games yet.** The Mini Plus
has no GPU, so slot's graphics path is still a work in progress — the current
build only probes the hardware and writes a log. If you have a Mini Plus and
want to help make it work, see the
[`experimental` branch](https://github.com/boomchu-123/slot-flip/tree/experimental)
and its `TESTERS.md`, and grab a `Slot.pak-miyoomini.zip`
[pre-release](https://github.com/boomchu-123/slot-flip/releases).

## Watch the launch video

<div align="center">
  <a href="https://www.youtube.com/watch?v=-XE-_v9wOAk">
    <img src="https://i.ytimg.com/vi/-XE-_v9wOAk/maxresdefault.jpg"
         alt="Introducing: Slot Flip — launch video" width="720">
  </a>
</div>

---

## Features

### From slot (upstream)

These come from the original [slot](https://slot-cfw.fyi) and are the heart of
the experience — all credit to the upstream author:

- **Game Boy, Game Boy Color, and Game Boy Advance** in one cartridge-style
  carousel, one shelf per system.
- **Plays with sound, battery saves, and save states**, all stored under `Slot`.
- **Cartridge artwork** — a per-game label image on each cart.
- **Wallpapers** — drop PNGs in `Wallpapers/` and it shuffles one in on boot.
- **Per-game core override** — pick mGBA or gpSP for a specific GBA title.
- **Quick menu** — fast-forward, colour correction, rumble, date & time, and the
  whole look and feel.

### New in this fork

Added on top of upstream for the Miyoo Flip (and portable to other MinUI
devices):

- **Miyoo Flip port** — SDL2 + GLES display, the Flip's button map, lid sensor,
  and power LED, packaged as a MinUI Tool.
- **Pin a specific background** — a `background =` key in `slot.ini` to choose
  one wallpaper instead of the random shuffle (or set `random`).
- **Remappable buttons** — an in-app **Buttons** screen plus a plain-text
  `buttons.ini`, so any device's keys can be rebound without a rebuild.
- **Lid-close sleep + auto power-off** — shut the lid and it sleeps instantly,
  then powers off after a timeout you choose in the menu (1–30 min, or never).
- **Right power LED behaviour** — green while running, red at low battery, both
  while charging.
- **Portable config** — the button map and settings are data-driven, so the port
  can be adapted to other MinUI devices (see [INSTALL.md](INSTALL.md) /
  [PORTING.md](PORTING.md)).

## Compatibility

- **Device:** Miyoo Flip (`my355`), running **MinUI**.
- **Systems:** GBA (`.gba`), GB (`.gb`), GBC (`.gbc`).
- Other MinUI devices can be supported with small changes — see
  [INSTALL.md](INSTALL.md).

---

## Install on MinUI (the easy way)

You need a MinUI SD card and about two minutes.

> **Don't have MinUI yet?** It's a minimal, clean launcher for handhelds like the
> Miyoo Flip. Grab it and follow its setup first:
> [MinUI by Shaun Inman](https://github.com/shauninman/MinUI) (see its
> [latest release](https://github.com/shauninman/MinUI/releases/latest) and
> README for flashing your SD card). Come back here once MinUI boots on your
> device.

1. **Download** the latest `Slot.pak` from this repo's
   [Releases](https://github.com/boomchu-123/slot-flip/releases/latest).
2. **Power off** the device and move the SD card to your computer.
3. Copy the **`Slot.pak`** folder into the Tools folder for your device:

   ```
   <SD CARD>/Tools/my355/Slot.pak
   ```

   Copy the whole folder — it contains `launch.sh`, the `slot` program, and a
   `System/` folder with the emulator cores.
4. Put the card back and boot into MinUI.
5. Open **Tools → Slot**. On first launch it creates a `Slot` folder at the root
   of the card and writes starter settings.
6. Power off, put the card back in your computer, and drop your **legally-dumped**
   ROMs into the matching folder:

   ```
   <SD CARD>/Slot/Games/GBA    (.gba)
   <SD CARD>/Slot/Games/GB     (.gb)
   <SD CARD>/Slot/Games/GBC    (.gbc)
   ```

7. Boot back into MinUI and open Slot — your carts appear in the carousel.

That's it. Nothing else on the card changes, and your MinUI setup is untouched.

### Boot straight into Slot (optional)

To skip the MinUI menu and land in Slot on boot, create
`<SD CARD>/.userdata/my355/auto.sh` with:

```sh
#!/bin/sh
/mnt/SDCARD/Tools/my355/Slot.pak/launch.sh
```

MinUI runs this before its menu, so the device boots into Slot; quitting Slot
drops you back to MinUI. Delete `auto.sh` to undo it.

---

## The Slot folder

Everything Slot owns lives under `<SD CARD>/Slot`:

```
Slot/
  Games/   GBA GB GBC      your ROMs, one folder per system
  Saves/   GBA GB GBC      battery saves
  States/  GBA GB GBC      save states
  Labels/  GBA GB GBC      cart artwork, named <rom-name>.png
  Wallpapers/              background images (PNG)
  Config/                  slot.ini, buttons.ini, and other settings
  System/                  emulator cores
  BIOS/                    optional GBA BIOS (gba_bios.bin)
```

### Cartridge artwork

Slot shows a label on each cart. To set one, drop a PNG named exactly after the
ROM (without the extension) into the matching `Labels` folder:

```
Slot/Games/GBA/Pokemon Emerald.gba
Slot/Labels/GBA/Pokemon Emerald.png
```

Any image works — Slot scales it to fit the cart label.

---

## Settings

### Background

1. Put one or more PNGs in `Slot/Wallpapers/`.
2. Edit `Slot/Config/slot.ini`:

   ```ini
   background = mybackground.png
   ```

   Use `random` (the default) to shuffle through all of them on each boot. If a
   named file is missing, Slot falls back to a random one.

### Sleep & auto power-off

- **Close the lid → sleeps immediately.**
- While asleep, the device powers itself off after a timeout you pick in the
  quick menu under **Sleep Timeout**: `1 / 3 / 5 / 10 / 30` minutes, or `Off` to
  stay asleep indefinitely. The choice is saved across reboots.

### Button remapping

Slot ships with the correct map for the Flip, but you can rebind any button
without a rebuild:

- In Slot, open the quick menu and choose **Buttons** to see the current
  bindings and where to change them.
- To remap, edit `Slot/Config/buttons.ini`. Each line is a kernel key code
  mapped to a button:

  ```ini
  # <kernel-key-code> = <button>
  109 = r1
  104 = l2
  ```

  Codes may be decimal or hex (`0x135`). Button names:
  `up down left right  a b x y  l1 r1 l2 r2  start select menu  volup voldown`.
  Anything you don't list keeps its built-in mapping.

To discover your codes, drop an empty file named `DEBUG` next to `launch.sh`,
launch once, and read `Slot/slot.log` — each press is logged with its code.

---

## Notes for the Flip

- **Display:** SDL2 + GLES2 (KMSDRM), which MinUI provides on the Flip.
- **Buttons:** the face buttons are mapped to the printed letters
  (A = right, B = bottom, X = top, Y = left). The right shoulder stands in for a
  dead R1 on some units; fast-forward is on the right analog stick.
- **Power LED:** Slot drives both of the Flip's lights — the green "work" light
  and the red "charger" light:
  - **Running / fully charged:** green
  - **Low battery (≤20%):** red
  - **Charging:** both green and red lit
- **GBA core:** gpSP by default (it has an ARM64 dynarec; mGBA does not). Per-game
  overrides live in `Slot/Config/selected_core.ini`.

## Installing on other MinUI devices

See [INSTALL.md](INSTALL.md) for the full guide and [PORTING.md](PORTING.md) for
the device seams (button codes, LED/backlight paths, display/audio wiring).

---

## Support & troubleshooting

First step for almost anything: drop an empty file named **`DEBUG`** next to
`launch.sh` in `Slot.pak`, launch once, reproduce the problem, then read
**`Slot/slot.log`**. It records the display/audio/input setup and every button
press, and most issues are obvious from it.

**Slot doesn't appear in MinUI's Tools menu.**
Check the folder is exactly `<SD CARD>/Tools/my355/Slot.pak` and contains
`launch.sh`, `slot`, and `System/`. The card must be readable (FAT32/exFAT as
MinUI expects).

**A game shows a black or white screen (but others work).**
That's almost always a **bad ROM dump**, not Slot — the same file will also fail
in other emulators. Re-dump the cartridge or replace it with a known-good copy
(verify against the No-Intro set). Some ROM *hacks* are also just broken.

**A GBA game runs slow, or crashes on load.**
gpSP is the default (it's fast on this chip). If a specific title misbehaves,
force mGBA for just that game by adding a line to
`Slot/Config/selected_core.ini`:

```ini
Pokemon Emerald = mgba
```

(use the ROM's name without the extension). mGBA is more accurate but slower.

**Saves aren't being kept / "Read-only file system" in the log.**
The SD card's filesystem has gone read-only, usually after a hard power-off or a
yank mid-write, which can corrupt FAT. Put the card in a computer and run a
filesystem check/repair (`fsck`/"Repair" on Windows/macOS). Always power the
device down cleanly.

**The buttons are wrong.**
Open the quick menu → **Buttons** to see the current map, and edit
`Slot/Config/buttons.ini` to rebind. Use the `DEBUG` log to find each button's
key code. See [Button remapping](#button-remapping).

**Closing the lid doesn't sleep.**
Confirm you're on the latest build. If a future device reports its lid
differently, that's a porting detail — see [PORTING.md](PORTING.md).

**It won't boot / I want to undo a change.**
Everything Slot owns is under the `Slot` folder and the `Slot.pak` — deleting
either is harmless to MinUI. If you auto-boot Slot, delete
`<SD CARD>/.userdata/my355/auto.sh` to return to the normal MinUI menu.

**Getting help.** This is a personal fork; for issues specific to it, open an
issue on this fork's repository and include your `slot.log` (captured with the
`DEBUG` file). For questions about the original frontend's look and behaviour,
upstream's guide at [slot-cfw.fyi](https://slot-cfw.fyi) is the reference.

---

## Credits & licence

**slot. is by [Brandon T. Kowalski](https://slot-cfw.fyi)** — a bespoke,
Game-Boy-centric frontend for the Anbernic RG SP that evokes using a GBA SP as a
kid. All of the frontend's design, look, and feel are his. This repository is a
**fork** that ports it to the Miyoo Flip and adds the features listed above; it
is **not** affiliated with or endorsed by the original author.

- Upstream project & guide: [slot-cfw.fyi](https://slot-cfw.fyi)
- Upstream release notes: [CHANGELOG.md](CHANGELOG.md)
- Licence: **GPL-3.0-or-later** (see [LICENSE](LICENSE)), same as upstream. As a
  GPL fork, source stays open.

Emulation is by the **mGBA** and **gpSP** libretro cores. Game Boy is a
trademark of Nintendo; this project is not affiliated with Nintendo. Use your
own legally-dumped ROMs.

The upstream frontend was written with an AI assistant and reviewed by its
author; this Flip port was likewise done with an AI coding assistant.
