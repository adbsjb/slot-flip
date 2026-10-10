#!/bin/sh
# SlotEvdev: raw-evdev button probe. Reads ALL /dev/input/event* nodes at once
# and logs the exact evdev code for every button, including ones SDL hides
# (notably R1). PRESS EVERY BUTTON while it runs (~30s): D-pad, A B X Y,
# L1 R1 L2 R2, Select, Start, Menu, Volume +/-.
#
# Output: slot-evdev.log at the root of the SD card.

DIR="$(dirname "$0")"
SD="${SDCARD_PATH:-/mnt/sdcard}"

export SLOT_LOG="$SD/slot-evdev.log"
export SLOT_EVDEV_SECS="30"
# SLOT_GRAB=1 makes the probe try to grab each node itself and report whether
# something else already holds it. Left off by default so it just listens.
# export SLOT_GRAB=1

cd "$DIR"
./flip_evdev > "$SD/slot-evdev.out" 2>&1
sync
