#!/usr/bin/env bash
set -euo pipefail
exec zig cc -target x86_64-macos "$(cat <<'ZIGENV'
CFLAGS+= -mmacosx-version-min=10.7
ZIGENV
) "$@"
