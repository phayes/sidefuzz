#!/usr/bin/env bash
# Generate a Scoop manifest for a dist-built Windows zip already attached to a GitHub Release.
# Usage: gen-scoop-manifest.sh <app> <version> <description> <homepage> <license> <repo> <tag> <zip-name> <bin-name> <sha256>
set -euo pipefail
app="$1"; version="$2"; desc="$3"; homepage="$4"; license="$5"; repo="$6"; tag="$7"; zipname="$8"; binname="$9"; sha="${10}"

url="https://github.com/phayes/${repo}/releases/download/${tag}/${zipname}"

cat > "${app}.json" <<EOF
{
    "version": "${version}",
    "description": "${desc}",
    "homepage": "${homepage}",
    "license": "${license}",
    "architecture": {
        "64bit": {
            "url": "${url}",
            "hash": "${sha}",
            "bin": "${binname}.exe"
        }
    }
}
EOF
echo "wrote ${app}.json"
