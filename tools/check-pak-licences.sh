#!/bin/sh
# Fails if the pak ships an emulator core that is not on the allowlist below.
#
# The main Slot.pak must stay free of non-commercial licences so the project can
# be sold, bundled or monetised without anyone having to untangle it later. Every
# core listed here is MPL-2.0 or GPL, which allow that. A core with a
# non-commercial licence (snes9x2005, Genesis Plus GX, PicoDrive, ...) belongs in the
# separate add-on pack, never here. To allow a new core you must have read its
# licence, then add a line below and ship its licence text and source with it.
set -eu

pak="${1:-flip/Tools/my355/Slot.pak}"
sys="$pak/System"
lic="$sys/licenses"

# core file | licence text that must ship with it | source archive that must ship (or -)
ALLOWED='
mgba_libretro.so|mgba-MPL-2.0.txt|-
gpsp_libretro.so|gpsp-GPL-2.0.txt|gpsp-src.tar.gz
pcsx_rearmed_libretro.so|pcsx-GPL-2.0.txt|pcsx-src.tar.gz
fceumm_libretro.so|fceumm-GPL-2.0.txt|fceumm-src.tar.gz
'

fail=0
for so in "$sys"/*_libretro.so; do
	[ -e "$so" ] || continue
	name="$(basename "$so")"
	row="$(printf '%s\n' "$ALLOWED" | grep "^$name|" || true)"
	if [ -z "$row" ]; then
		echo "NOT ALLOWED: $name is not a core cleared for the main pak (see this script)" >&2
		fail=1
		continue
	fi
	licence="$(echo "$row" | cut -d'|' -f2)"
	source="$(echo "$row" | cut -d'|' -f3)"
	{ [ -s "$lic/$licence" ] || { echo "MISSING OR EMPTY: $lic/$licence for $name" >&2; fail=1; }; }
	[ -s "$lic/$name.meta" ] || { echo "MISSING: $lic/$name.meta for $name" >&2; fail=1; }
	if [ "$source" != "-" ]; then
		[ -s "$lic/$source" ] || { echo "MISSING: $lic/$source for $name" >&2; fail=1; }
	fi
done
exit "$fail"
