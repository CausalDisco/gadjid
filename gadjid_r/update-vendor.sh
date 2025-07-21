#!/usr/bin/env bash
set -euo pipefail
trap 's=$?; echo >&2 "$0: Error on line "$LINENO": $BASH_COMMAND"; exit $s' ERR

# cd relative to folder of this .sh file
cd "$(dirname "$(readlink -f "$0")")/src/rust"

SOURCE_EPOCH=$(date +%Y-%m-%dT%H:%M:%SZ)
TARFLAGS="
  --sort=name --format=posix
  --pax-option=exthdr.name=%d/PaxHeaders/%f
  --pax-option=delete=atime,delete=ctime
  --clamp-mtime --mtime=$SOURCE_EPOCH
  --numeric-owner --owner=0 --group=0
  --mode=go+u,go-w
  --no-acls --no-selinux --no-xattrs
"

cargo vendor --versioned-dirs
LC_ALL=C tar -cJ --no-xattrs -f vendor.tar.xz vendor

cd -
