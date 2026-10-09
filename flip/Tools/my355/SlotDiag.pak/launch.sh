#!/bin/sh
# SlotDiag: collects what the slot port needs to know about this Miyoo Flip.
# Run it from MinUI's Tools menu. It writes slot-diag.txt to the root of the SD card.
# It only reads system state and changes nothing on the device.

SD="${SDCARD_PATH:-/mnt/SDCARD}"
OUT="$SD/slot-diag.txt"

section() {
	printf '\n===== %s =====\n' "$1" >> "$OUT"
}

run() {
	printf '\n$ %s\n' "$*" >> "$OUT"
	"$@" >> "$OUT" 2>&1
}

sh_run() {
	printf '\n$ %s\n' "$1" >> "$OUT"
	sh -c "$1" >> "$OUT" 2>&1
}

dump_dir() {
	# every small readable file in a sysfs directory, name = value
	for f in "$1"/*; do
		[ -f "$f" ] || continue
		v="$(head -c 200 "$f" 2>/dev/null | tr '\n' ' ')"
		printf '%s = %s\n' "$f" "$v" >> "$OUT"
	done
}

: > "$OUT"
echo "slot-diag $(date '+%F %T')" >> "$OUT"

section "env"
sh_run "env | sort"

section "kernel and libc"
run uname -a
sh_run "cat /etc/os-release"
sh_run "cat /proc/version"
sh_run "ls -l /lib/ld-linux* /lib/ld-* /lib64 2>&1"
sh_run "ls -l /lib/libc.so* /lib/aarch64-linux-gnu/libc.so* /lib/aarch64-linux-gnu/libc-* 2>&1"
sh_run "ldd --version 2>&1 | head -2"
sh_run "/lib/libc.so.6 2>&1 | head -2"
sh_run "/lib/aarch64-linux-gnu/libc.so.6 2>&1 | head -2"
sh_run "grep -E 'model name|Hardware|CPU part|Features' /proc/cpuinfo | sort -u"
sh_run "free -m; df -h; mount"

section "graphics libraries"
sh_run "ls -l /usr/lib/libSDL2* /usr/miyoo/lib/libSDL2* $SD/.system/my355/lib/libSDL2* 2>&1"
sh_run "ls -l /usr/lib/libEGL* /usr/lib/libGLES* /usr/lib/libmali* /usr/lib/libgbm* /usr/lib/libdrm* /usr/lib/libasound* 2>&1"
sh_run "ls -l /usr/miyoo/lib 2>&1"
sh_run "ls -l $SD/.system/my355/lib $SD/.system/my355/bin 2>&1"
sh_run "ls -l /dev/dri /dev/fb* /dev/mali* 2>&1"
sh_run "ls /sys/class/drm 2>&1"
sh_run "for c in /sys/class/drm/card*-*; do echo \$c: \$(cat \$c/status 2>/dev/null) \$(cat \$c/modes 2>/dev/null | head -3 | tr '\n' ' '); done"
dump_dir /sys/class/graphics/fb0

section "MinUI video log (SDL drivers and display modes)"
LOGS="${LOGS_PATH:-$SD/.userdata/my355/logs}"
sh_run "head -60 $LOGS/minui.txt 2>&1"
sh_run "ls -l $LOGS 2>&1"

section "input"
sh_run "ls -l /dev/input"
for d in /sys/class/input/event*; do
	n="$(cat "$d/device/name" 2>/dev/null)"
	printf '%s: %s | key=%s | abs=%s | sw=%s\n' "$d" "$n" \
		"$(cat "$d/device/capabilities/key" 2>/dev/null)" \
		"$(cat "$d/device/capabilities/abs" 2>/dev/null)" \
		"$(cat "$d/device/capabilities/sw" 2>/dev/null)" >> "$OUT"
done
sh_run "cat /proc/bus/input/devices"
sh_run "ls -l /sys/class/miyooio_chr_dev 2>&1; cat /sys/class/miyooio_chr_dev/joy_type 2>&1"
sh_run "ps | grep -E 'miyoo|keymon|input' | grep -v grep"

section "lid and hall sensor"
sh_run "ls -l /sys/devices/platform/hall-mh248 2>&1"
sh_run "cat /sys/devices/platform/hall-mh248/hallvalue 2>&1"
sh_run "ls /sys/devices/platform | head -80"

section "backlight, battery, leds, rtc"
sh_run "ls -l /sys/class/backlight /sys/class/power_supply /sys/class/leds /sys/class/rtc 2>&1"
for d in /sys/class/backlight/*; do
	[ -d "$d" ] || continue
	printf '\n-- %s\n' "$d" >> "$OUT"
	dump_dir "$d"
done
for d in /sys/class/power_supply/*; do
	[ -d "$d" ] || continue
	printf '\n-- %s\n' "$d" >> "$OUT"
	dump_dir "$d"
done
for d in /sys/class/leds/*; do
	[ -d "$d" ] || continue
	printf '\n-- %s\n' "$d" >> "$OUT"
	dump_dir "$d"
done
sh_run "cat /sys/class/gpio/gpio20/value /sys/class/gpio/gpio150/value 2>&1"
sh_run "date; hwclock -r 2>&1"

section "audio"
sh_run "cat /proc/asound/cards /proc/asound/pcm 2>&1"
sh_run "ls -l /dev/snd 2>&1"
sh_run "aplay -l 2>&1; aplay -L 2>&1 | head -40"
sh_run "amixer scontrols 2>&1 | head -30"
sh_run "ls -l /etc/asound.conf /usr/share/alsa 2>&1"

section "cpu"
sh_run "ls /sys/devices/system/cpu/cpufreq/policy0 2>&1"
sh_run "cat /sys/devices/system/cpu/cpufreq/policy0/scaling_available_frequencies /sys/devices/system/cpu/cpufreq/policy0/scaling_available_governors 2>&1"

section "minui install"
sh_run "ls -la $SD"
sh_run "ls -l $SD/.system/my355/cores $SD/.system/my355/paks 2>&1"
sh_run "cat $SD/.system/version.txt 2>&1"
sh_run "ls -R $SD/Tools 2>&1 | head -40"
sh_run "ls $SD/Roms 2>&1"

echo "" >> "$OUT"
echo "done" >> "$OUT"
sync
