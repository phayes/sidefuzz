#!/usr/bin/env bash
# Build a minimal .deb from an already-compiled binary (no recompilation).
# Usage: build-deb.sh <pkg> <version> <arch-deb> <binary-path> <description> <homepage> <extracted-dir-to-scan-for-LICENSE*>
set -euo pipefail
pkg="$1"; version="$2"; arch="$3"; binpath="$4"; desc="$5"; homepage="$6"; licensedir="$7"

root="$(mktemp -d)"
mkdir -p "$root/DEBIAN" "$root/usr/bin" "$root/usr/share/doc/$pkg"
chmod 0755 "$root"

install -m 0755 "$binpath" "$root/usr/bin/$(basename "$binpath")"

# Bundle whatever LICENSE* files dist packaged into the release archive, if any.
shopt -s nullglob
license_files=("$licensedir"/LICENSE*)
if [ ${#license_files[@]} -gt 0 ]; then
  cat "${license_files[@]}" > "$root/usr/share/doc/$pkg/copyright"
  chmod 0644 "$root/usr/share/doc/$pkg/copyright"
fi

size_kb=$(du -sk "$root/usr" | cut -f1)

cat > "$root/DEBIAN/control" <<EOF
Package: $pkg
Version: $version
Section: utils
Priority: optional
Architecture: $arch
Maintainer: Patrick Hayes <patrick.d.hayes@gmail.com>
Installed-Size: $size_kb
Homepage: $homepage
Description: $desc
EOF

out="${pkg}_${version}_${arch}.deb"
dpkg-deb --build --root-owner-group "$root" "$out"
echo "built: $out"
