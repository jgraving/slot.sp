#!/bin/sh
# Pinned website build tool with a verified alternate route for release-CDN failures.
set -eu
wasm_pack_name=wasm-pack-v0.15.0-x86_64-unknown-linux-musl
wasm_pack_destination=${1:?installation directory required}
wasm_pack_sha=c09f971ecaed9a2efc80fdcea7a00ef6b53c7fadc8c57d1f61b53a6aa66b668a
wasm_pack_download=$(mktemp -d)
trap 'rm -rf "$wasm_pack_download"' EXIT HUP INT TERM
if ! curl --fail --location --silent --show-error --retry 1 --connect-timeout 15 --max-time 90 \
    "https://github.com/wasm-bindgen/wasm-pack/releases/download/v0.15.0/$wasm_pack_name.tar.gz" \
    -o "$wasm_pack_download/wasm-pack.tar.gz"; then
    curl --fail --location --silent --show-error --retry 2 --connect-timeout 15 --max-time 90 \
        -H 'Accept: application/octet-stream' \
        https://api.github.com/repos/wasm-bindgen/wasm-pack/releases/assets/420850440 \
        -o "$wasm_pack_download/wasm-pack.tar.gz"
fi
printf '%s  %s\n' "$wasm_pack_sha" "$wasm_pack_download/wasm-pack.tar.gz" | sha256sum --check
mkdir -p "$wasm_pack_destination"
tar --no-same-owner -xzf "$wasm_pack_download/wasm-pack.tar.gz" -C "$wasm_pack_destination"
"$wasm_pack_destination/$wasm_pack_name/wasm-pack" --version
