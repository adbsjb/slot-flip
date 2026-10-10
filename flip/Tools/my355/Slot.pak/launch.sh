#!/bin/sh
# slot. for the Miyoo Flip (my355), launched as a MinUI Tool.
#
# Keeps all of its data in /mnt/sdcard/Slot so it never touches your MinUI Roms
# or Saves. First run creates that tree (Games/GBA, Games/GB, Games/GBC, Saves,
# States, BIOS, ...). Drop your legally dumped ROMs into Slot/Games/<system>/.

DIR="$(dirname "$0")"
SD="${SDCARD_PATH:-/mnt/sdcard}"
SLOT_ROOT="$SD/Slot"

# Device SDL2 + Mali/EGL stack, and ALSA, all resolved at runtime.
export LD_LIBRARY_PATH="/usr/lib:/usr/miyoo/lib:$LD_LIBRARY_PATH"
export SLOT_ROOT

# slot looks for its cores (mgba_libretro.so, gpsp_libretro.so) in
# $SLOT_ROOT/System and next to the binary. Point it at the pak's own System
# dir first, then fall back to MinUI's bundled cores if present.
mkdir -p "$SLOT_ROOT/System"
for c in mgba_libretro.so gpsp_libretro.so; do
	if [ ! -f "$SLOT_ROOT/System/$c" ] && [ -f "$DIR/System/$c" ]; then
		cp "$DIR/System/$c" "$SLOT_ROOT/System/$c"
	fi
done

LOG="$SD/slot.log"

cd "$DIR"
# Create/repair the data tree, then run.
./slot --init-root "$SLOT_ROOT" >> "$LOG" 2>&1
./slot > "$LOG" 2>&1
sync
