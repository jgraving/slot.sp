#!/bin/sh
# Private SMB3 upload inbox. Password arrives on stdin; runtime secrets stay in RAM.
set -eu
# Never write SMB session material to a card crash dump.
ulimit -c 0
RUN="${AGS_RUN:-/run}"
STATE="$RUN/slot-smb"
SELF="$0"

stop_share() {
    if [ -f "$STATE/owner" ]; then
        pid="$(cat "$STATE/owner")"
        case "$pid" in ''|*[!0-9]*) return 1;; esac
        # Only terminate our own supervisor group, never a recycled or unrelated PID.
        if [ -r "/proc/$pid/cmdline" ] && tr '\000' ' ' < "/proc/$pid/cmdline" | grep -F 'slotsmb.sh serve ' >/dev/null; then
            kill -TERM "-$pid" 2>/dev/null || true
        fi
    fi
    rm -f "$STATE/owner" "$STATE/smbpasswd" "$STATE/smbpasswd.old" "$STATE/smb.conf"
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
    [ -x "$TOOLS/bin/$name" ] || exit 1
done
# No loader or library-path changes escape into the launcher/emulators.
samba() {
    name="$1"; shift
    GCONV_PATH="$TOOLS/gconv" "${SLOT_SMB_LOADER:-$TOOLS/lib/ld-linux-aarch64.so.1}" --library-path "$TOOLS/lib" "$TOOLS/bin/$name" "$@"
}
if [ "$1" = prepare ]; then
    address="$3"
    case "$address" in ''|*[!0-9.]*|0.*|169.254.*|255.*) exit 1;; esac
    [ ! -L "$STATE" ] || exit 1
    umask 077
    mkdir -p "$STATE" "$STATE/private" "$STATE/lock" "$STATE/cache" "$STATE/state"
    for dir in "$SD/Inbox" "$SD/Inbox/GB" "$SD/Inbox/GBC" "$SD/Inbox/GBA" "$SD/Inbox/Wallpapers"; do
        [ ! -L "$dir" ] || exit 1
        mkdir -p "$dir"
    done
    IFS= read -r password
    # A generated 80-bit password uses an unambiguous controller-readable alphabet.
    [ "${#password}" -eq 16 ] || exit 1
    case "$password" in *[!ABCDEFGHJKLMNPQRSTUVWXYZ23456789]*) exit 1;; esac
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
[Games]
    path = $SD/Inbox
    comment = Upload games and PNG wallpapers, then turn sharing off to import
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
    printf '%s\n%s\n' "$password" "$password" | samba smbpasswd -s -a -c "$STATE/smb.conf" nobody >/dev/null 2>&1
    unset password
    samba testparm -s "$STATE/smb.conf" >/dev/null 2>&1
    exit 0
fi
# setsid/process_group(0) is supplied by the launcher before this supervisor starts.
[ -s "$STATE/smbpasswd" ] && [ -f "$STATE/smb.conf" ] || exit 1
parent="$3"
cleanup() {
    trap - TERM INT EXIT
    rm -f "$STATE/owner" "$STATE/smbpasswd" "$STATE/smbpasswd.old" "$STATE/smb.conf"
    kill -TERM "-$$" 2>/dev/null || true
}
trap cleanup TERM INT EXIT
printf '%s\n' "$$" > "$STATE/owner"
samba samba-dcerpcd --foreground --no-process-group --np-only --configfile="$STATE/smb.conf" "$TOOLS/libexec/rpcd_classic" "$TOOLS/libexec/rpcd_winreg" &
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
