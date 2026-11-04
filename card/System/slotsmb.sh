#!/bin/sh
# Private SMB3 library shares. Password arrives on stdin; runtime secrets stay in RAM.
set -eu
# Never write SMB session material to a card crash dump.
ulimit -c 0
RUN="${AGS_RUN:-/run}"
STATE="$RUN/slot-smb"
LOADER="$STATE/loader"
SELF="$0"

clear_runtime() {
    rm -f "$STATE/owner" "$STATE/smbpasswd" "$STATE/smbpasswd.old" "$STATE/smb.conf" \
        "$LOADER" "$STATE/rpcd_classic" "$STATE/rpcd_winreg"
}

stop_share() {
    if [ -f "$STATE/owner" ]; then
        pid="$(cat "$STATE/owner")"
        case "$pid" in ''|*[!0-9]*) return 1;; esac
        # Only terminate our own supervisor group, never a recycled or unrelated PID.
        if [ -r "/proc/$pid/cmdline" ] && tr '\000' ' ' < "/proc/$pid/cmdline" | grep -F 'slotsmb.sh serve ' >/dev/null; then
            kill -TERM "-$pid" 2>/dev/null || true
        fi
    fi
    clear_runtime
}
case "${1:-}" in
stop) stop_share; exit 0;;
prepare|serve) ;;
*) exit 2;;
esac
SD="$(cd "$2" && pwd -P)"
# Samba interprets % substitutions and line breaks in configuration paths.
case "$SD" in *'%'*|*'['*|*']'*|*'
'*|*'
'*) exit 1;; esac
TOOLS="$SD/System/samba"
for name in smbd smbpasswd testparm wsdd2 samba-dcerpcd rpcd_classic rpcd_winreg; do
    [ -r "$TOOLS/bin/$name" ] || exit 1
done
# No loader or library-path changes escape into the launcher/emulators.
samba() {
    name="$1"; shift
    GCONV_PATH="$TOOLS/gconv" "$LOADER" --library-path "$TOOLS/lib" "$TOOLS/bin/$name" "$@"
}
if [ "$1" = prepare ]; then
    address="$3"
    case "$address" in ''|*[!0-9.]*|0.*|169.254.*|255.*) exit 1;; esac
    [ ! -L "$STATE" ] || exit 1
    umask 077
    trap 'clear_runtime' EXIT
    mkdir -p "$STATE" "$STATE/private" "$STATE/lock" "$STATE/cache" "$STATE/state"
    # FAT can omit execute bits. The private loader maps the Samba binaries
    # from SD, while the loader and RPC entry points execute from runtime storage.
    rm -f "$LOADER" "$STATE/rpcd_classic" "$STATE/rpcd_winreg"
    cp "${SLOT_SMB_LOADER:-$TOOLS/lib/ld-linux-aarch64.so.1}" "$LOADER"
    for worker in rpcd_classic rpcd_winreg; do
        cat > "$STATE/$worker" <<'WRAPPER'
#!/bin/sh
set -eu
exec "$SLOT_SMB_LOADER_PATH" --library-path "$SLOT_SMB_TOOLS/lib" "$SLOT_SMB_TOOLS/bin/${0##*/}" "$@"
WRAPPER
    done
    chmod 700 "$LOADER" "$STATE/rpcd_classic" "$STATE/rpcd_winreg"
    for dir in "$SD/Games" "$SD/Games/GB" "$SD/Games/GBC" "$SD/Games/GBA" \
        "$SD/Labels" "$SD/Labels/GB" "$SD/Labels/GBC" "$SD/Labels/GBA" "$SD/Wallpapers"; do
        [ ! -L "$dir" ] || exit 1
        mkdir -p "$dir"
    done
    IFS= read -r password
    # The controller keyboard and launcher accept 8–32 printable ASCII bytes.
    LC_ALL=C
    export LC_ALL
    [ "${#password}" -ge 8 ] && [ "${#password}" -le 32 ] || exit 1
    case "$password" in *[![:print:]]*) exit 1;; esac
    extra=
    if IFS= read -r extra || [ -n "$extra" ]; then exit 1; fi
    printf 'nobody = slot\n' > "$STATE/users"
    cat > "$STATE/smb.conf" <<CONF
[global]
    server role = standalone server
    security = user
    workgroup = WORKGROUP
    netbios name = SLOTSP
    server string = slot.sp
    interfaces = $address/32
    bind interfaces only = yes
    smb ports = ${SLOT_SMB_TEST_PORT:-445}
    server min protocol = SMB3_00
    server max protocol = SMB3_11
    server smb encrypt = required
    server signing = mandatory
    ntlm auth = ntlmv2-only
    map to guest = Never
    restrict anonymous = 2
    null passwords = no
    passdb backend = smbpasswd:$STATE/smbpasswd
    username map = $STATE/users
    private dir = $STATE/private
    lock directory = $STATE/lock
    state directory = $STATE/state
    cache directory = $STATE/cache
    pid directory = $STATE
    ncalrpc dir = $STATE/ncalrpc
    rpc start on demand helpers = no
    log file = /dev/null
    log level = 0
    load printers = no
    disable spoolss = yes
    dns proxy = no
    multicast dns register = no
    usershare max shares = 0
    allow insecure wide links = no
    max smbd processes = 8
    deadtime = 5
CONF
    for folder in Games Labels Wallpapers; do
        cat >> "$STATE/smb.conf" <<CONF
[$folder]
    path = $SD/$folder
    browseable = yes
    read only = no
    guest ok = no
    valid users = nobody
    force user = root
    follow symlinks = no
    wide links = no
    hide dot files = yes
    veto files = /.*/
    delete veto files = no
    max connections = 4
CONF
    done
    printf '%s\n%s\n' "$password" "$password" | samba smbpasswd -s -a -c "$STATE/smb.conf" nobody >/dev/null 2>&1
    unset password
    samba testparm -s "$STATE/smb.conf" >/dev/null 2>&1
    trap - EXIT
    exit 0
fi
# setsid/process_group(0) is supplied by the launcher before this supervisor starts.
[ -s "$STATE/smbpasswd" ] && [ -f "$STATE/smb.conf" ] || exit 1
export SLOT_SMB_LOADER_PATH="$LOADER" SLOT_SMB_TOOLS="$TOOLS"
parent="$3"
cleanup() {
    trap - TERM INT EXIT
    clear_runtime
    kill -TERM "-$$" 2>/dev/null || true
}
trap cleanup TERM INT EXIT
printf '%s\n' "$$" > "$STATE/owner"
samba samba-dcerpcd --foreground --no-process-group --np-only --configfile="$STATE/smb.conf" "$STATE/rpcd_classic" "$STATE/rpcd_winreg" &
rpc=$!
samba smbd --foreground --no-process-group --configfile="$STATE/smb.conf" &
server=$!
samba wsdd2 -4 -i "${SLOT_SMB_TEST_INTERFACE:-wlan0}" -H SLOTSP -N SLOTSP -G WORKGROUP &
# Close the entire group if slot crashes, or if the Wi-Fi hook revokes this session.
(
    while kill -0 "$parent" 2>/dev/null && kill -0 "$server" 2>/dev/null && kill -0 "$rpc" 2>/dev/null && [ -f "$STATE/owner" ]; do sleep 1; done
    kill -TERM "-$$" 2>/dev/null || true
) &
wait "$server"
