#!/bin/sh
# Wi-Fi for slot: starts and stops wpa_supplicant on wlan0 and keeps a DHCP lease while it is
# connected. Slot talks to the supplicant itself over its control socket for scanning and
# joining; this only does what needs a shell.
#
#   slotwifi.sh up CONF     start the supplicant on CONF, and the watcher that runs DHCP
#   slotwifi.sh down        stop both, release the lease and turn the radio off
#   slotwifi.sh wlan0 CONNECTED|DISCONNECTED
#                           what wpa_cli's watcher calls as the link comes and goes
set -u

NET="${AGS_NET_SYS:-/sys/class/net}"
RUN="${AGS_RUN:-/run}"
CTRL_DIR="${AGS_CTRL_DIR:-/var/run/wpa_supplicant}"
WPA_SUPPLICANT="${AGS_WPA_SUPPLICANT:-wpa_supplicant}"
WPA_CLI="${AGS_WPA_CLI:-wpa_cli}"
UDHCPC="${AGS_UDHCPC:-udhcpc}"
IP="${AGS_IP:-ip}"
RFKILL="${AGS_RFKILL:-rfkill}"
WAIT_S="${AGS_WIFI_WAIT_S:-5}"

LINK_MARK="$RUN/slotlink.session"
LINK_WAIT_S="${AGS_WIFI_LINK_WAIT_S:-10}"
WATCH_PID="$RUN/slotwifi-watch.pid"
LEASE_PID="$RUN/slotwifi-udhcpc.pid"
OWNER="$RUN/slotwifi.owner"

stop_pid() {
	[ -f "$1" ] || return 0
	kill "$(cat "$1")" 2>/dev/null || true
	/bin/rm -f "$1"
}

wifi_up() {
	conf="$RUN/slotwifi.conf"
	[ ! -f "$LINK_MARK" ] || return 1
	umask 077
	# A link session owns wlan0 until slotlink.sh takes it down and clears its mark.
	m=0
	while [ -f "$LINK_MARK" ] && [ "$m" -lt $((LINK_WAIT_S * 4)) ]; do
		sleep 0.25 2>/dev/null || sleep 1
		m=$((m + 1))
	done
	w=0
	while [ ! -d "$NET/wlan0" ] && [ "$w" -lt 40 ]; do
		sleep 0.25 2>/dev/null || sleep 1
		w=$((w + 1))
	done
	[ -d "$NET/wlan0" ] || return 1
	$RFKILL unblock wifi 2>/dev/null || true
	$IP link set wlan0 up 2>/dev/null || true
	/bin/mkdir -p "$CTRL_DIR"
	if [ ! -s "$conf" ]; then
		/bin/mkdir -p "$(dirname "$conf")"
		{
			echo "ctrl_interface=$CTRL_DIR"
			echo 'update_config=0'
		} > "$conf"
	fi
	if $WPA_CLI -p "$CTRL_DIR" -i wlan0 ping 2>/dev/null | grep -q PONG; then
        [ -f "$OWNER" ] || return 1
    else
		/bin/rm -f "$CTRL_DIR/wlan0"
		$WPA_SUPPLICANT -B -i wlan0 -c "$conf" -Dnl80211 >/dev/null 2>&1 || return 1
	fi
	echo "$$" > "$OWNER"
	s=0
	while [ ! -e "$CTRL_DIR/wlan0" ] && [ "$s" -lt $((WAIT_S * 4)) ]; do
		sleep 0.25 2>/dev/null || sleep 1
		s=$((s + 1))
	done
	[ -e "$CTRL_DIR/wlan0" ] || return 1
	stop_pid "$WATCH_PID"
	$WPA_CLI -p "$CTRL_DIR" -i wlan0 -B -P "$WATCH_PID" -a "$0" >/dev/null 2>&1 || true
	return 0
}

stop_sharing() {
    helper="$(dirname "$0")/slotsmb.sh"
    [ ! -f "$helper" ] || sh "$helper" stop >/dev/null 2>&1 || true
}

wifi_down() {
    stop_sharing
    [ -f "$OWNER" ] || return 0
    /bin/rm -f "$OWNER"
	stop_pid "$WATCH_PID"
	stop_pid "$LEASE_PID"
	$WPA_CLI -p "$CTRL_DIR" -i wlan0 terminate >/dev/null 2>&1 || true
	$IP addr flush dev wlan0 2>/dev/null || true
	$IP link set wlan0 down 2>/dev/null || true
	$RFKILL block wifi 2>/dev/null || true
	/bin/rm -f "$RUN/slotwifi.conf"
	return 0
}

lease() {
	stop_pid "$LEASE_PID"
	$UDHCPC -i wlan0 -b -p "$LEASE_PID" >/dev/null 2>&1 || true
}

# Read-only, allowlisted facts. Never read configuration, scan results or logs.
diagnostics() {
    present() { if [ -e "$2" ]; then echo "$1=yes"; else echo "$1=no"; fi; }
    process() {
        if [ ! -f "$2" ]; then echo "$1=missing"; return; fi
        pid="$(cat "$2" 2>/dev/null)"
        case "$pid" in ''|*[!0-9]*|0|1) echo "$1=stale"; return ;; esac
        if kill -0 "$pid" 2>/dev/null; then echo "$1=alive"; else echo "$1=stale"; fi
    }
    available() { if command -v "$2" >/dev/null 2>&1; then echo "$1=yes"; else echo "$1=no"; fi; }
    present interface_present "$NET/wlan0"
    present owner_present "$OWNER"
    present link_session "$LINK_MARK"
    process watcher "$WATCH_PID"
    process dhcp_client "$LEASE_PID"
    available wpa_cli_available "$WPA_CLI"
    available udhcpc_available "$UDHCPC"
    available ip_available "$IP"
    if [ -x "$0" ]; then echo callback_executable=yes; else echo callback_executable=no; fi
    if grep -q '^[[:space:]]*nameserver[[:space:]]' "${AGS_RESOLV_CONF:-/etc/resolv.conf}" 2>/dev/null; then
        echo resolver_configured=yes
    else echo resolver_configured=no; fi
}

case "${1:-} ${2:-}" in
"diagnostics ") diagnostics ;;
"up "*) wifi_up "${2:-}" ;;
"down "*) wifi_down ;;
"wlan0 CONNECTED") lease ;;
"wlan0 DISCONNECTED")
    stop_sharing
	stop_pid "$LEASE_PID"
	$IP addr flush dev wlan0 2>/dev/null || true
	;;
*)
	echo "usage: slotwifi.sh up CONF | down" >&2
	exit 2
	;;
esac
