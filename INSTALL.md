# Installing Slot on MinUI

Slot is a cartridge-style front-end for Game Boy, Game Boy Color, and Game Boy
Advance. This fork runs as a **MinUI Tool**: you drop one folder onto your SD
card and launch it from MinUI's Tools menu. It keeps all of its own data in a
single `Slot` folder and never touches your MinUI Roms or Saves.

## Quick install (Miyoo Flip / my355)

1. Power off your device and take out the SD card. Put it in your computer.
2. Copy the `Slot.pak` folder from this release into:

   ```
   <SD CARD>/Tools/my355/Slot.pak
   ```

   (The whole folder, with `launch.sh`, `slot`, and `System/` inside it.)
3. Put the card back and boot the device into MinUI.
4. Open **Tools → Slot**. On first launch Slot creates its `Slot` folder at the
   root of the card and fills in the starter config.
5. Power off, put the card back in your computer, and drop your legally-dumped
   ROMs into the matching folder:

   ```
   <SD CARD>/Slot/Games/GBA   (.gba)
   <SD CARD>/Slot/Games/GB    (.gb)
   <SD CARD>/Slot/Games/GBC   (.gbc)
   ```

6. Boot back into MinUI, launch Slot, and your carts appear in the carousel.

That's it. Nothing else on the card changes.

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

## Backgrounds (configurable)

1. Put one or more PNG files in `Slot/Wallpapers/`.
2. Open `Slot/Config/slot.ini` and set which one to use:

   ```ini
   background = mybackground.png
   ```

   Use `background = random` (the default) to shuffle through every wallpaper
   on each boot. If the named file is missing, Slot falls back to a random one.

## Sleep & auto power-off

Closing the lid puts the device to sleep **immediately**. While asleep, it
powers itself off after a timeout you choose in the quick menu:

- Open the quick menu and set **Sleep Timeout** to `1`, `3`, `5`, `10`, or
  `30` minutes, or `Off` to stay asleep indefinitely (no auto power-off).

The choice is saved and survives reboots.

## Remapping buttons (menu with mappable buttons)

Slot ships with a sensible map for each device it knows about, but any MinUI
handheld can be remapped without a rebuild.

- In Slot, open the quick menu and choose **Buttons** to see the current
  bindings and where to change them.
- To remap, edit `Slot/Config/buttons.ini`. Each line is:

  ```ini
  # <kernel-key-code> = <button>
  109 = r1
  104 = l2
  ```

  Codes may be decimal (`109`) or hex (`0x135`). Button names are:
  `up down left right  a b x y  l1 r1 l2 r2  start select menu  volup voldown`.
  Any code you don't list keeps its built-in mapping, so you only change what
  you need.

To find your device's key codes, drop an empty file named `DEBUG` next to
`launch.sh`, launch once, and read `Slot/slot.log` — each button press is
logged with its code.

## Auto-booting Slot (optional)

To boot straight into Slot and skip the MinUI menu, create
`<SD CARD>/.userdata/<platform>/auto.sh` containing:

```sh
#!/bin/sh
/mnt/SDCARD/Tools/<platform>/Slot.pak/launch.sh
```

Replace `<platform>` with your device (e.g. `my355`). MinUI runs this before
its menu loop, so the device boots into Slot; quitting Slot drops you back to
MinUI. Delete `auto.sh` to undo it.

## Installing on other MinUI devices

Slot's data folder and config are device-independent. To bring it to another
MinUI handheld:

1. Copy `Slot.pak` into that device's `Tools/<platform>/` folder (the
   `<platform>` is the same name MinUI uses for that device, e.g. `tg5040`).
2. Launch it once to generate the `Slot` folder.
3. If the buttons are wrong, remap them in `Slot/Config/buttons.ini` using the
   `DEBUG` log trick above to read your device's key codes.

The display and audio use SDL2, which MinUI provides on its supported devices.
LED and backlight paths in `launch.sh` are device-specific — see `PORTING.md`
for what to adjust when bringing Slot to a new device.
