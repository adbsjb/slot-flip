# slot. — Miyoo Flip fork

A cartridge-style Game Boy / Game Boy Color / Game Boy Advance frontend, ported
to run on the **Miyoo Flip (my355)** as a **MinUI Tool**.

This is a fork of [slot](https://slot-cfw.fyi) (originally for the Anbernic RG SP),
adapted for the Flip's hardware (SDL2 + GLES display, the Flip's button map and
lid sensor, green power LED) and with a few extras: configurable backgrounds, an
in-app button-remap menu, lid-close sleep with a configurable auto power-off, and
a data-driven config so it can be adapted to other MinUI devices.

It keeps everything it owns in a single `Slot` folder and never touches your
MinUI Roms or Saves.

---

## Install on MinUI (the easy way)

You need a MinUI SD card and about two minutes.

1. **Power off** the device and move the SD card to your computer.
2. Copy the **`Slot.pak`** folder from the release into the Tools folder for your
   device:

   ```
   <SD CARD>/Tools/my355/Slot.pak
   ```

   Copy the whole folder — it contains `launch.sh`, the `slot` program, and a
   `System/` folder with the emulator cores.
3. Put the card back and boot into MinUI.
4. Open **Tools → Slot**. On first launch it creates a `Slot` folder at the root
   of the card and writes starter settings.
5. Power off, put the card back in your computer, and drop your **legally-dumped**
   ROMs into the matching folder:

   ```
   <SD CARD>/Slot/Games/GBA    (.gba)
   <SD CARD>/Slot/Games/GB     (.gb)
   <SD CARD>/Slot/Games/GBC    (.gbc)
   ```

6. Boot back into MinUI and open Slot — your carts appear in the carousel.

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

## Credits & AI disclosure

Upstream slot. is by **Brandon T. Kowalski** — a bespoke frontend for the
Anbernic RG SP that evokes using a GBA SP as a kid. Original guide:
[slot-cfw.fyi](https://slot-cfw.fyi). Release notes: [changelog](CHANGELOG.md).

The upstream Rust frontend was put together with Claude Opus and reviewed by the
author; this Miyoo Flip port was likewise done with an AI coding assistant.
Use your own legally-dumped ROMs.
