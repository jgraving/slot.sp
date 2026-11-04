#!/bin/sh
# Staged System-only OTA. Based on upstream slot 87788eaf, GPL-3.0-or-later.
# Keep staging intact until every atomic file replacement is synced, so a retry
# after power loss can finish without ever depending on a missing System folder.
set -eu
SD="${SLOT_ROOT:-/mnt/sdcard}"
SYS="$SD/System"
NEW="$SD/System.new"
[ -e "$NEW" ] || [ -L "$NEW" ] || exit 0
# Refuse redirected trees before cleanup or installation can touch their targets.
[ -d "$NEW" ] && [ ! -L "$NEW" ] && [ ! -L "$SYS" ] || exit 1
[ ! -e "$SYS" ] || [ -d "$SYS" ] || exit 1
if [ ! -f "$NEW/.manifest" ]; then
    rm -rf "$NEW"
    exit 0
fi
SPLASH="${AGS_SPLASH:-/usr/bin/baseos-splash}"
say() { "$SPLASH" --important -1 "$1" 2>/dev/null || true; }
# Validate every component, including existing destination parents. Checking only
# the leaf would still follow a symlinked System/samba (or staged) directory.
regular_path() {
    base="$1"; rest="$2"
    while :; do
        part="${rest%%/*}"
        base="$base/$part"
        [ ! -L "$base" ] || return 1
        if [ "$rest" = "$part" ]; then
            [ ! -e "$base" ] || [ -f "$base" ] || return 1
            return 0
        fi
        [ ! -e "$base" ] || [ -d "$base" ] || return 1
        rest="${rest#*/}"
    done
}
[ ! -L "$NEW/.manifest" ] || exit 1
# Validate everything before touching the installed tree. Config/Games/Saves
# are outside this directory and are never installation destinations.
while IFS= read -r f; do
    case "$f" in
        ""|/*|..|../*|*/../*|*/..|.|./*|*/./*|*/.|*//*|*/|*\\*) say "UPDATE INVALID"; exit 1 ;;
    esac
    regular_path "$NEW" "$f" && regular_path "$SYS" "$f" && regular_path "$SYS" "$f.slot-update" || { say "UPDATE INVALID PATH"; exit 1; }
    [ -f "$NEW/$f" ] && [ ! -L "$NEW/$f" ] || { say "UPDATE INCOMPLETE"; exit 1; }
done < "$NEW/.manifest"
for f in slot launch_frontend.sh mgba_libretro.so gpsp_libretro.so build-info.json; do
    grep -qxF "$f" "$NEW/.manifest" || { say "UPDATE INCOMPLETE"; exit 1; }
done
# Make the complete recovery source durable before replacing installed files.
sync
say "UPDATING SLOT.SP"
put() {
    # put is called in an OR-list: errexit does not guard commands in here.
    mkdir -p "$(dirname "$SYS/$1")" || return 1
    rm -f "$SYS/$1.slot-update" || return 1
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
