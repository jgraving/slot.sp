#!/bin/bash
# Build a private, relocatable SMB3-only runtime using maintained Samba sources.
# Run natively on the target architecture; the ARM64 build uses the device's glibc baseline.
set -euo pipefail
script="$(realpath "$0")"
out="${1:?output directory}"
mkdir -p "$out"
out="$(cd "$out" && pwd)"
work="${SLOT_SAMBA_BUILD:-$PWD/.samba-build}"
mkdir -p "$work"
work="$(cd "$work" && pwd)"
sources="${2:-${out}-sources}"
mkdir -p "$sources"
sources="$(cd "$sources" && pwd)"
version=4.25.1
sha=7f367545bca69aa96897628f82f0f04d4b66b3bcb36cb978b9bb952c4b4c428d
wsdd_sha=2b1e7720435a1e067388660ec3edb321a4c91b4f9d0928ba27d0a8d89b7ef3b9
fetch() {
    local url="$1" file="$2" digest="$3"
    if [ ! -f "$file" ]; then
        if ! curl --ipv4 --fail --location --retry 4 --retry-all-errors --connect-timeout 20 --max-time 180 "$url" -o "$file"; then
            rm -f "$file"
            case "$url" in
                https://download.samba.org/pub/samba/stable/*)
                    curl --ipv4 --fail --location --retry 3 --retry-all-errors --connect-timeout 20 --max-time 180 \
                        "https://www.samba.org/samba/ftp/stable/${url##*/}" -o "$file" ;;
                *) return 1 ;;
            esac
        fi
    fi
    echo "$digest  $file" | sha256sum -c -
}
fetch "https://download.samba.org/pub/samba/stable/samba-$version.tar.gz" "$work/samba.tar.gz" "$sha"
fetch 'https://deb.debian.org/debian/pool/main/w/wsdd2/wsdd2_1.8.7+dfsg.orig.tar.xz' "$work/wsdd2.tar.xz" "$wsdd_sha"
build_id="$(sha256sum "$script" | cut -d" " -f1)"
if [ ! -x "$work/install/sbin/smbd" ] || [ "$(cat "$work/install/.slot-build-id" 2>/dev/null || true)" != "$build_id" ]; then
    rm -rf "$work/source" "$work/install"
    mkdir -p "$work/source"
    tar -xzf "$work/samba.tar.gz" -C "$work/source" --strip-components=1
    cd "$work/source"
    ./configure --prefix="$work/install" --with-system-mitkrb5 --without-ad-dc --without-ads --without-winbind \
        --without-ldap --without-pam --without-json --disable-cups --disable-iprint --disable-avahi \
        --disable-python --without-acl-support --without-quotas --without-systemd --without-gpgme \
        --without-libarchive --without-regedit --without-utmp --without-syslog --without-libunwind \
        --without-cluster-support --without-ldb-lmdb --without-smb1-server --without-gettext \
        --without-prometheus-exporter --without-ratelimitd --disable-cephfs --disable-cephrgw \
        --disable-glusterfs --disable-spotlight --disable-wsp \
        --with-static-modules='!DEFAULT,pdb_smbpasswd,pdb_tdbsam,auth_unix,nss_info_template,idmap_tdb,idmap_passdb,idmap_nss' --with-shared-modules='!DEFAULT,!FORCED' \
        --nonshared-binary=smbd/smbd \
        --disable-rpath-install --disable-rpath-private-install
    make -j "${SLOT_BUILD_JOBS:-4}"
    make install
    printf "%s\n" "$build_id" > "$work/install/.slot-build-id"
fi
mkdir -p "$work/wsdd2"
tar -xf "$work/wsdd2.tar.xz" -C "$work/wsdd2" --strip-components=1
make -C "$work/wsdd2" CFLAGS='-O2 -D_FORTIFY_SOURCE=2 -fstack-protector-strong' LDFLAGS='-Wl,-z,relro,-z,now'
mkdir -p "$out/bin" "$out/lib" "$out/licenses"
for name in smbd smbpasswd testparm smbclient samba-dcerpcd rpcd_classic; do
    src="$work/install/bin/$name"
    [ -f "$src" ] || src="$work/install/sbin/$name"
    [ -f "$src" ] || src="$work/install/libexec/samba/$name"
    cp "$src" "$out/bin/$name"
done
cp "$work/wsdd2/wsdd2" "$out/bin/"
# Samba invokes RPC workers itself; a small wrapper keeps those children on the private loader too.
mkdir -p "$out/libexec"
cat > "$out/libexec/rpcd_classic" <<'WRAPPER'
#!/bin/sh
set -eu
TOOLS="$(cd "$(dirname "$0")/.." && pwd)"
exec "$TOOLS/lib/ld-linux-aarch64.so.1" --library-path "$TOOLS/lib" "$TOOLS/bin/rpcd_classic" "$@"
WRAPPER
chmod +x "$out/libexec/rpcd_classic"
triplet="$(gcc -print-multiarch)"
cp -aL "/usr/lib/$triplet/gconv" "$out/gconv"

# Preserve Samba's private libraries if a binary is not eligible for nonshared linkage.
find "$work/install/lib" -type f -name '*.so*' -exec cp -L '{}' "$out/lib/" \;
# Copy every system runtime dependency, including the matching loader, into this private tree.
# No installed system library is replaced and no LD_LIBRARY_PATH is exported by the launcher.
python3 - "$out" "$work/install/lib" "$sources" <<'PYCODE'
import os,pathlib,re,shutil,subprocess,sys
out=pathlib.Path(sys.argv[1]);lib=out/'lib';sources=pathlib.Path(sys.argv[3]);packages=set()
env=dict(os.environ,LD_LIBRARY_PATH=str(lib)+':'+sys.argv[2]+':'+sys.argv[2]+'/private')
queue=list((out/'bin').iterdir())+list(lib.iterdir());seen=set()
while queue:
    binary=queue.pop()
    if binary in seen:continue
    seen.add(binary)
    result=subprocess.run(['ldd',str(binary)],env=env,text=True,capture_output=True)
    if 'not found' in result.stdout:raise SystemExit(result.stdout)
    for path in re.findall(r'(/[^\s()]+)',result.stdout):
        src=pathlib.Path(path)
        if not src.is_file():continue
        dst=lib/src.name
        if not dst.exists():
            shutil.copyfile(src,dst);dst.chmod(0o755);queue.append(dst)
            owner=subprocess.run(['dpkg-query','-S',str(src.resolve())],text=True,capture_output=True).stdout.split(': /')[0]
            if owner:packages.add(owner)
manifest=[]
for package in sorted(packages):
    info=subprocess.check_output(['dpkg-query','-W','-f=${source:Package}\t${source:Version}',package],text=True).split('\t')
    name,version=info
    manifest.append(f'{package}\t{name}\t{version}\n')
    notice=pathlib.Path('/usr/share/doc')/package.split(':')[0]/'copyright'
    if notice.is_file():
        dest=out/'licenses/dependencies'/f'{package.replace(":","-")}.txt';dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(notice,dest)
    subprocess.run(['apt-get','source','--download-only','--only-source',f'{name}={version}'],cwd=sources,check=True)
(sources/'dependency-sources.tsv').write_text(''.join(manifest))
(out/'licenses/dependency-sources.tsv').write_text(''.join(manifest))
PYCODE

cp "$work/samba.tar.gz" "$sources/samba-$version.tar.gz"
cp "$work/source/COPYING" "$out/licenses/samba-GPL-3.0.txt"
cp "$work/wsdd2.tar.xz" "$sources/wsdd2-1.8.7-source.tar.xz"
cp "$work/wsdd2/LICENSE" "$out/licenses/wsdd2-LICENSE.txt"
printf 'Samba %s\nsource_sha256=%s\nwsdd2_source_sha256=%s\n' "$version" "$sha" "$wsdd_sha" > "$out/build-info.txt"
for binary in "$out"/bin/*; do strip "$binary"; done

cp "$script" "$sources/build-samba.sh"
printf 'Corresponding source: slot.sp-network-sources.zip from the same build/release.\n' > "$out/licenses/SOURCES.txt"
