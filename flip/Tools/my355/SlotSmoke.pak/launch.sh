#!/bin/sh
# SlotSmoke: proves the slot SDL2/GLES2 surface and input mapping on the Flip.
# Draws a slowly cycling colour for ~20 seconds and logs display info plus the
# gamepad's raw button codes to slot-smoke.log at the root of the SD card.

DIR="$(dirname "$0")"
SD="${SDCARD_PATH:-/mnt/sdcard}"

# The device's own SDL2 and Mali/EGL stack. keymon (started by MinUI) keeps
# running, so volume/brightness keep working while this is on screen.
export LD_LIBRARY_PATH="/usr/lib:/usr/miyoo/lib:$LD_LIBRARY_PATH"

export SLOT_LOG="$SD/slot-smoke.log"
export SLOT_INPUT="/dev/input/event5"
export SLOT_SMOKE_SECS="20"

cd "$DIR"
./flip_smoke > "$SD/slot-smoke.out" 2>&1
sync
