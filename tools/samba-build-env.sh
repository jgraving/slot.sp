#!/bin/bash
# Dedicated native ARM64 container: newer SMB dependencies never change the launcher build baseline.
set -euo pipefail
if [ -f /etc/apt/sources.list.d/debian.sources ]; then
    sed -i 's/^Types: deb$/Types: deb deb-src/' /etc/apt/sources.list.d/debian.sources
fi
apt-get update
apt-get install -y --no-install-recommends build-essential flex bison python3 python3-dev pkg-config \
    libgnutls28-dev libacl1-dev libattr1-dev libpopt-dev libbsd-dev libicu-dev \
    perl libparse-yapp-perl libjansson-dev liblmdb-dev libcap-dev libtasn1-6-dev libtasn1-bin \
    libtirpc-dev libkrb5-dev curl ca-certificates xz-utils zip
bash tools/build-samba.sh vendor/samba vendor/samba-sources
