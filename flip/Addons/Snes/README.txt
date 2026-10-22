slot-flip add-on: Super Nintendo
================================

This adds Super Nintendo (SNES) games to slot-flip on the Miyoo Flip.

Install
-------
1. Unzip this onto the root of your SD card, so that the `Slot` folder here
   merges with the `Slot` folder that slot-flip already made. You should end
   up with  <SD CARD>/Slot/System/snes9x2005_plus_libretro.so
2. Put your Super Nintendo games (.sfc, .smc, .swc, .fig) in
   <SD CARD>/Slot/Games/SNES
3. Open Slot. A new SNES shelf appears. Cover art goes in
   <SD CARD>/Slot/Labels/SNES, named after the game.

Without this add-on, a SNES game in that folder shows "Add-on pack needed"
and does not start. Nothing else is affected.

Why it is separate
------------------
The emulator inside is snes9x2005 (a Snes9x derivative). Its licence allows use,
copying, modification and distribution for NON-COMMERCIAL purposes only, as long
as the notice in Slot/System/licenses/snes9x2005-copyright.txt goes with every
copy. slot-flip itself is free software and ships without it, so that the main
download carries no such restriction. This pack is free. Do not sell it, and do
not bundle it with anything that is sold.

The licence text, the build recipe stamp and the source this core was built from
are in Slot/System/licenses/.

Not affiliated with Nintendo. Super Nintendo Entertainment System is a trademark
of Nintendo. Use your own legally dumped games.
