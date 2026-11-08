#!/bin/sh
# Pinned build tool. Verify the same published digest on either download route.
set -eu
task_version=v3.54.0
task_destination=${1:-/usr/local/bin}
case "$(uname -m)" in
    aarch64|arm64)
        task_arch=arm64; task_asset=604090041
        task_sha=d7189d439d4a6058e39e1b2acd7c8b27acd4856691b408f23b242e4294e1df5b ;;
    x86_64|amd64)
        task_arch=amd64; task_asset=604090073
        task_sha=680859dbb4d881a9c72d4d9a8f510825450849af8567deacd7302c01124416fb ;;
    *) echo 'Unsupported Task build host' >&2; exit 1 ;;
esac
task_archive=task_linux_${task_arch}.tar.gz
task_download=$(mktemp -d)
trap 'rm -rf "$task_download"' EXIT HUP INT TERM
if ! curl --fail --location --silent --show-error --retry 1 --connect-timeout 15 --max-time 90 \
    "https://github.com/go-task/task/releases/download/$task_version/$task_archive" -o "$task_download/$task_archive"; then
    curl --fail --location --silent --show-error --retry 2 --connect-timeout 15 --max-time 90 \
        -H 'Accept: application/octet-stream' \
        "https://api.github.com/repos/go-task/task/releases/assets/$task_asset" -o "$task_download/$task_archive"
fi
printf '%s  %s\n' "$task_sha" "$task_download/$task_archive" | sha256sum --check
mkdir -p "$task_destination"
tar --no-same-owner -xzf "$task_download/$task_archive" -C "$task_destination" task
"$task_destination/task" --version
