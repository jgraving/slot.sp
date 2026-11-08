#!/bin/sh
# Pinned build tool: avoid GitHub's mutable latest-release metadata endpoint.
set -eu
task_version=v3.54.0
task_destination=${1:-/usr/local/bin}
case "$(uname -m)" in
    aarch64|arm64) task_arch=arm64 ;;
    x86_64|amd64) task_arch=amd64 ;;
    *) echo 'Unsupported Task build host' >&2; exit 1 ;;
esac
task_archive=task_linux_${task_arch}.tar.gz
task_release=https://github.com/go-task/task/releases/download/${task_version}
task_download=$(mktemp -d)
trap 'rm -rf "$task_download"' EXIT HUP INT TERM
curl --fail --location --silent --show-error --retry 3 --connect-timeout 15 --max-time 120 \
    "$task_release/$task_archive" -o "$task_download/$task_archive"
curl --fail --location --silent --show-error --retry 3 --connect-timeout 15 --max-time 120 \
    "$task_release/task_checksums.txt" -o "$task_download/checksums.txt"
(
    cd "$task_download"
    awk -v archive="$task_archive" '$2 == archive {print}' checksums.txt > selected-checksum.txt
    test -s selected-checksum.txt
    sha256sum --check selected-checksum.txt
)
mkdir -p "$task_destination"
tar -xzf "$task_download/$task_archive" -C "$task_destination" task
"$task_destination/task" --version
