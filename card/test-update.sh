#!/bin/sh
set -eu
HERE="$(cd "$(dirname "$0")" && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
stage() {
    mkdir -p "$TMP/card/System.new" "$TMP/card/System" "$TMP/card/Config" "$TMP/card/Games" "$TMP/card/Saves"
    echo kept > "$TMP/card/Config/settings"
    echo kept > "$TMP/card/Games/game.gba"
    echo kept > "$TMP/card/Saves/game.sav"
    for f in slot launch_frontend.sh mgba_libretro.so gpsp_libretro.so build-info.json; do
        echo new > "$TMP/card/System.new/$f"
        echo old > "$TMP/card/System/$f"
    done
    printf '%s\n' slot launch_frontend.sh mgba_libretro.so gpsp_libretro.so build-info.json > "$TMP/card/System.new/.manifest"
}
apply() { SLOT_ROOT="$TMP/card" AGS_SPLASH=true sh "$HERE/System/slotupdate.sh"; }
stage
apply
[ "$(cat "$TMP/card/System/slot")" = new ]
[ ! -d "$TMP/card/System.new" ]
for f in Config/settings Games/game.gba Saves/game.sav; do [ "$(cat "$TMP/card/$f")" = kept ]; done
stage
# Missing staging file must leave the installed binary intact and retain staging.
rm "$TMP/card/System.new/gpsp_libretro.so"
if apply; then echo 'accepted incomplete update' >&2; exit 1; fi
[ "$(cat "$TMP/card/System/slot")" = old ]
[ -f "$TMP/card/System.new/.manifest" ]
echo repaired > "$TMP/card/System.new/gpsp_libretro.so"
apply
[ "$(cat "$TMP/card/System/slot")" = new ]
stage
printf '../Config/settings\n' >> "$TMP/card/System.new/.manifest"
if apply; then echo 'accepted path traversal' >&2; exit 1; fi
[ "$(cat "$TMP/card/Config/settings")" = kept ]
echo 'update: all passed'
