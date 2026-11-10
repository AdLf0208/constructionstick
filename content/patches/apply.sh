#!/bin/sh
# Re-apply the hand fixes after re-running the carry.
# Run from anywhere; the fixes are copied over the carried assets/data.
set -eu
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
cp -r "$here/extra/data" "$here/.."
echo "re-applied: 14 data/c/tags/item/*.json and data/constructionstick/horizon/creative_tab/tab.json"
