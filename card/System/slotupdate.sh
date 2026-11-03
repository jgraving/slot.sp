#!/bin/sh
# Staged System-only OTA. Based on upstream slot 87788eaf, GPL-3.0-or-later.
# Keep staging intact until every atomic file replacement is synced, so a retry
# after power loss can finish without ever depending on a missing System folder.
set -eu
SD="${SLOT_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
NEW="$SD/System.new"
[ -d "$NEW" ] || exit 0
if [ ! -f "$NEW/.manifest" ]; then
    rm -rf "$NEW"
    exit 0
fi
SPLASH="${AGS_SPLASH:-/usr/bin/baseos-splash}"
say() { "$SPLASH" --important -1 "$1" 2>/dev/null || true; }
# Validate everything before touching the installed tree. Config/Games/Saves
# are outside this directory and are never installation destinations.
while IFS= read -r f; do
    case "$f" in
        ""|/*|..|../*|*/../*|*/..|./*|*/./*|*\\*) say "UPDATE INVALID"; exit 1 ;;
    esac
    [ -f "$NEW/$f" ] && [ ! -L "$NEW/$f" ] || { say "UPDATE INCOMPLETE"; exit 1; }
done < "$NEW/.manifest"
for f in slot launch_frontend.sh mgba_libretro.so gpsp_libretro.so build-info.json; do
    grep -qxF "$f" "$NEW/.manifest" || { say "UPDATE INCOMPLETE"; exit 1; }
done
say "UPDATING SLOT.SP"
put() {
    mkdir -p "$(dirname "$SYS/$1")"
    rm -f "$SYS/$1.slot-update"
    cp -p "$NEW/$1" "$SYS/$1.slot-update" && mv -f "$SYS/$1.slot-update" "$SYS/$1"
}
while IFS= read -r f; do
    case "$f" in slot|launch_frontend.sh) continue ;; esac
    put "$f" || { say "UPDATE FAILED - RESTART TO RETRY"; exit 1; }
done < "$NEW/.manifest"
for f in slot launch_frontend.sh; do
    put "$f" || { say "UPDATE FAILED - RESTART TO RETRY"; exit 1; }
done
sync
rm -rf "$NEW"
sync
say "UPDATE COMPLETE"
