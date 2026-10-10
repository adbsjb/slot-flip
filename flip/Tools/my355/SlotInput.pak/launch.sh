#!/bin/sh
# SlotInput: captures the Flip's pad through SDL (the way MinUI reads it) and
# logs every button/axis/hat with its SDL id to slot-input.log on the SD card.
# PRESS EVERY BUTTON while it runs (~30s): D-pad, A B X Y, L1 R1 L2 R2,
# Select, Start, Menu, and the volume keys.

DIR="$(dirname "$0")"
SD="${SDCARD_PATH:-/mnt/sdcard}"

export LD_LIBRARY_PATH="/usr/lib:/usr/miyoo/lib:$LD_LIBRARY_PATH"
export SLOT_LOG="$SD/slot-input.log"
export SLOT_INPUT_SECS="30"

cd "$DIR"
./flip_input > "$SD/slot-input.out" 2>&1
sync
