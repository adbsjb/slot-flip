#!/bin/sh
# slot. for the Miyoo Mini Plus (miyoomini), launched as a MinUI Tool.
#
# EXPERIMENTAL / WORK IN PROGRESS. The Mini Plus has no GPU, so the graphics
# path that works on the Flip cannot run here. This build currently probes the
# hardware and writes slot.log, but does not draw to the screen yet. See
# TESTERS.md. Please share the slot.log it produces on the GitHub thread.
#
# Keeps all of its data in /mnt/SDCARD/Slot so it never touches your MinUI Roms
# or Saves. Drop legally dumped ROMs into Slot/Games/<GBA|GB|GBC>/.

DIR="$(dirname "$0")"
SD="${SDCARD_PATH:-/mnt/SDCARD}"
SLOT_ROOT="$SD/Slot"

export LD_LIBRARY_PATH="/usr/lib:/usr/miyoo/lib:$LD_LIBRARY_PATH"
export SLOT_ROOT

LOG="$SLOT_ROOT/slot.log"

# Diagnostics are opt-in. Drop a file named DEBUG next to this script to turn on
# the verbose input trace while chasing a problem.
if [ -f "$DIR/DEBUG" ]; then
	export SLOT_TRACE=1
	export SLOT_TRACE_LATENCY=1
fi

# Seed the emulator cores into Slot/System on first run. NOTE: the Mini Plus is
# 32-bit ARM (armhf); it needs armhf cores, which are not bundled yet.
mkdir -p "$SLOT_ROOT/System"
for c in mgba_libretro.so gpsp_libretro.so; do
	if [ ! -f "$SLOT_ROOT/System/$c" ] && [ -f "$DIR/System/$c" ]; then
		cp "$DIR/System/$c" "$SLOT_ROOT/System/$c"
	fi
done

cd "$DIR"
./slot --init-root "$SLOT_ROOT" > "$LOG" 2>&1
./slot >> "$LOG" 2>&1
sync
