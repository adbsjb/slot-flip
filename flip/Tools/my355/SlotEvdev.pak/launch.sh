#!/bin/sh
# SlotEvdev (thorough): captures EVERY button and the full range of every analog
# axis across all input devices, with NO filtering. Press buttons in any order,
# as many times as you like. When done, hold VOLUME-UP + VOLUME-DOWN together to
# finish and write the summary. There is no time limit.
#
# Output: slot-evdev.log (summary) and slot-evdev.log.trace (full trace) at the
# root of the SD card.

DIR="$(dirname "$0")"
SD="${SDCARD_PATH:-/mnt/sdcard}"

export SLOT_LOG="$SD/slot-evdev.log"

cd "$DIR"
./flip_evdev > "$SD/slot-evdev.out" 2>&1
sync
