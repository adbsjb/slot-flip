#!/bin/sh
# slot. for the Miyoo Flip (my355), launched as a MinUI Tool.
#
# Keeps all of its data in /mnt/sdcard/Slot so it never touches your MinUI Roms
# or Saves. Drop legally dumped ROMs into Slot/Games/<GBA|GB|GBC>/.

DIR="$(dirname "$0")"
SD="${SDCARD_PATH:-/mnt/sdcard}"
SLOT_ROOT="$SD/Slot"

export LD_LIBRARY_PATH="/usr/lib:/usr/miyoo/lib:$LD_LIBRARY_PATH"
export SLOT_ROOT

# Verbose trace this build so any freeze point lands in the log.
export SLOT_TRACE=1

# Optional isolation: if a file named NO_CORE sits in the pak, run the built-in
# mock instead of a real emulator core. Lets us tell a UI problem from a core
# problem without rebuilding.
if [ -f "$DIR/NO_CORE" ]; then
	export SLOT_NO_CORE=1
fi

mkdir -p "$SLOT_ROOT/System"
for c in mgba_libretro.so gpsp_libretro.so; do
	if [ ! -f "$SLOT_ROOT/System/$c" ] && [ -f "$DIR/System/$c" ]; then
		cp "$DIR/System/$c" "$SLOT_ROOT/System/$c"
	fi
done

LOG="$SD/slot.log"
cd "$DIR"
./slot --init-root "$SLOT_ROOT" > "$LOG" 2>&1
echo "---- run ----" >> "$LOG"
./slot >> "$LOG" 2>&1
echo "---- exited rc=$? ----" >> "$LOG"
sync
