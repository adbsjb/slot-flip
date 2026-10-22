#!/bin/sh
# Checks an add-on pack (default: every folder under flip/Addons) before it is
# zipped: each core in it must be one slot knows is non-commercial, must carry its
# licence text, build stamp and source, and the pack must say it is free-only.
# The reverse guard, that none of these cores reaches the main pak, is
# tools/check-pak-licences.sh.
set -eu

# core file | licence text that must ship with it | source archive that must ship
KNOWN='
snes9x2005_plus_libretro.so|snes9x2005-copyright.txt|snes9x2005-src.tar.gz
'

fail=0
check() {
	dir="$1"
	sys="$dir/Slot/System"
	lic="$sys/licenses"
	found=0
	for so in "$sys"/*_libretro.so; do
		[ -e "$so" ] || continue
		found=1
		name="$(basename "$so")"
		row="$(printf '%s\n' "$KNOWN" | grep "^$name|" || true)"
		if [ -z "$row" ]; then
			echo "UNKNOWN: $name in $dir is not a core this script knows; read its licence, then add it" >&2
			fail=1
			continue
		fi
		licence="$(echo "$row" | cut -d'|' -f2)"
		source="$(echo "$row" | cut -d'|' -f3)"
		for need in "$licence" "$name.meta" "$source"; do
			[ -s "$lic/$need" ] || { echo "MISSING OR EMPTY: $lic/$need" >&2; fail=1; }
		done
		file "$so" | grep -q aarch64 || { echo "NOT AARCH64: $so" >&2; fail=1; }
	done
	[ "$found" = 1 ] || { echo "NO CORE in $dir" >&2; fail=1; }
	grep -qi "non-commercial" "$dir/README.txt" 2>/dev/null ||
		{ echo "$dir/README.txt does not state the non-commercial terms" >&2; fail=1; }
}

if [ $# -gt 0 ]; then
	for d in "$@"; do check "$d"; done
else
	for d in flip/Addons/*/; do check "${d%/}"; done
fi
exit "$fail"
