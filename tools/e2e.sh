#!/usr/bin/env bash
# In-game check for Construction Sticks slice 1 (plane construction).
#
# Three phases, all in survival so the supply is actually paid:
#
#   1. stone wall 5x3: a wooden stick (limit 3) right-clicked on its south face
#      must place exactly 3 stone in the face's plane (z=1) and consume
#      exactly 3 stone items. This is the "vanilla placement + survival cost"
#      phase: the port used to write `default_state` with `native_set_block`.
#   2. stone-brick stairs wall: the three placed stairs must carry
#      `facing=north` (the player looks north). The stairs' default state is
#      `facing=east`, so a default-state write could not produce this: the
#      placement went through `Behaviours::placement_state`.
#   3. stone wall with a cow standing in one plane cell: the cow blocks the
#      building of that cell (vanilla `isUnobstructed`), so the cell stays air
#      and the cow survives; the rest of the plane still fills. A placement
#      that ignored the obstruction would put stone in the cow's cell.
#
#   tools/e2e.sh                     # the wooden stick: 3 blocks, 3 stone
#   COUNTERFACTUAL=1 tools/e2e.sh    # same click, plain minecraft:stick: 0, 0
#
# EMBEDDED=1 (the default) launches with **only the native bundle** and no
# `--dev-mod`: the content comes from the bundle's embedded copy
# (`mixins/constructionstick/data/…`, the AE2 pattern). CONTROL=1 (default with
# EMBEDDED) first launches the same binary with `.horizon-cache` unusable, where
# the stick ids must be unknown.
#
# SELFTEST=1 flips the "3 blocks placed" assertion to 4, which must FAIL: the
# proof the scenario can fail.
#
# The counterfactual is how the check proves it can fail: the scenario and every
# assertion are unchanged, only the port's item is withheld. A plain stick has
# no use, so it must place nothing and consume nothing.
set -euo pipefail

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/constructionstick
# The control binary is shared at the campaign pin; the game is this port's build.
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/constructionstick-e2e-run.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
SELFTEST=${SELFTEST:-0}
EMBEDDED=${EMBEDDED:-1}
CONTROL=${CONTROL:-$EMBEDDED}
DEV_MOD=()
[ "$EMBEDDED" = 1 ] || DEV_MOD=(--dev-mod "$PORT/content")

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/content/horizon-mod.toml" ] || { echo "missing content mod: $PORT/content" >&2; exit 1; }

if [ "$COUNTERFACTUAL" = 1 ]; then
    TOOL=minecraft:stick
    EXPECT_PLACED=0
    EXPECT_CONSUMED=0
else
    TOOL=constructionstick:wooden_stick
    EXPECT_PLACED=3
    EXPECT_CONSUMED=3
fi
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting 4 blocks placed instead of $EXPECT_PLACED)"
    EXPECT_PLACED=4
fi

SCRATCH=$(mktemp -d /tmp/opencode/constructionstick-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
# The game's game folder defaults to the working directory: run from scratch so
# saves/options/caches never land in the port tree.
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
count_item() { ctl inventory | jq -r --arg i "$1" '[.counts[$i] // 0] | add'; }
# The built plane, exactly: the 5x3 face at z=1 (x -3..3, y 100..102). The
# wall itself is at z=0, so every stone in this box is a placed cell.
plane_stone() { ctl blocks -3 100 1 3 102 1 | jq '[.blocks[] | select(.[3] == "minecraft:stone")] | length'; }
plane_stairs() { ctl blocks -3 100 1 3 102 1 | jq '[.blocks[] | select(.[3] | startswith("minecraft:stone_brick_stairs"))] | length'; }
block_state() { ctl block "$1" "$2" "$3" | jq -r '.state // "unknown"'; }
cows() { ctl entities 8 cow | jq -r '.entities | length'; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

# The player must be where the assertions think it is: `ctl tp` sets the client
# position and the server corrects it ("moved wrongly"), so the server-side
# teleport is used and its result checked.
place_player() {
    ctl command "tp @s 0 100 2" >/dev/null
    ctl wait 2
    local pos
    pos=$(ctl state | jq -r '.player.position | @csv')
    echo "player: $pos"
    case "$pos" in
        0.5,100*,2.5* | 0,100*,2*) ;;
        *) fail "the player is at $pos, not at the wall" ;;
    esac
}
# Clear the plane, rebuild the wall of $1 at z=0, and arm the player.
rebuild() {
    ctl command "kill @e[type=!minecraft:player,type=!minecraft:item]" >/dev/null
    ctl fill -6 100 -6 6 104 6 minecraft:air >/dev/null
    ctl fill -2 100 0 2 102 0 "$1" >/dev/null
    ctl command "clear @s" >/dev/null
    # The stick pays with the clicked block's own item.
    ctl give "$1" 64 >/dev/null || fail "cannot give $1"
    ctl give "$TOOL" 1 >/dev/null || fail "the content mod did not provide $TOOL"
    ctl wait 2
    ctl select "$TOOL" >/dev/null || fail "cannot select $TOOL"
    ctl wait 2
    local held
    held=$(ctl inventory | jq -r '.held.item // "none"')
    echo "held: $held"
    [ "$held" = "$TOOL" ] || fail "the held item is $held, expected $TOOL"
}
# Aim at the wall's south face and right-click it.
click_wall() {
    ctl look-at-block 0 101 0 south || fail "cannot aim at the wall's south face"
    ctl wait 2
    ctl use-item
    ctl wait 10
}

launch() { # launch WORLD
    rm -f "$SOCK"
    "$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$1" --seed 20261102 --headless --wait \
        --game-dir "$SCRATCH/game" --resources "$RES" "${DEV_MOD[@]}" >/dev/null
    ctl wait-ready 120
}

# The control for EMBEDDED=1: the same binary (every hook is in it) with the
# embedded content made unavailable — `.horizon-cache` is a plain file, so the
# helper can neither compare nor write its content file and falls back to
# `Continue`. The stick ids must then be unknown; if they resolve, the run proves
# nothing about the embedded file.
control_leg() {
    echo "== control: the same game with no embedded content file =="
    mkdir -p "$HORIZON_HOME"
    rm -rf "$HORIZON_HOME/.horizon-cache"
    : > "$HORIZON_HOME/.horizon-cache"
    launch ConstructionStickControl
    ctl command "clear @s" >/dev/null
    ctl give constructionstick:wooden_stick 1 >/dev/null 2>&1 || true
    ctl wait 3
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "constructionstick:wooden_stick")] | length == 0' >/dev/null \
        || fail "the control game has the construction stick, so the content did not come from the embedded file"
    [ -f "$HORIZON_HOME/.horizon-cache" ] || fail "the control's .horizon-cache was replaced: the helper wrote its content"
    echo "  without the embedded file: no construction stick could be given"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$HORIZON_HOME/.horizon-cache"
}

# EMBEDDED=1: the bundle must have written its content out at startup, byte for
# byte the delivered content mod.
embedded_content_check() {
    local cache=$HORIZON_HOME/.horizon-cache/constructionstick-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/constructionstick.hmod" \
        || fail "the embedded content differs from content/constructionstick.hmod"
    echo "  embedded content: $(wc -c < "$cache" | tr -d ' ') B at $cache, identical to content/constructionstick.hmod"
}

if [ "$CONTROL" = 1 ]; then control_leg; fi

if [ "$EMBEDDED" = 1 ]; then
    echo "== launch (survival world ConstructionStickE2E, one .hnmod, no --dev-mod) =="
else
    echo "== launch (survival world ConstructionStickE2E, content --dev-mod content/) =="
fi
launch ConstructionStickE2E
if [ "$EMBEDDED" = 1 ]; then embedded_content_check; fi

echo "== a stone platform in open air (y=99), wall 5x3 at z=0 and y=100..102 =="
ctl command "time set noon"
ctl command "difficulty peaceful"
ctl fill -6 99 -6 6 99 6 minecraft:stone
ctl wait 2
place_player

echo "== phase 1: stone wall, wooden stick (limit 3) =="
rebuild minecraft:stone
before=$(count_item minecraft:stone)
plane_before=$(plane_stone)
echo "stone before: $before"
echo "plane stone before: $plane_before"
click_wall
after=$(count_item minecraft:stone)
plane_after=$(plane_stone)
echo "stone after: $after"
echo "plane stone after: $plane_after"
ctl blocks -3 100 1 3 102 1 | jq -c '{counts, blocks}'

[ "$before" -ge "$EXPECT_CONSUMED" ] || fail "not enough stone to consume"
[ $((plane_after - plane_before)) -eq "$EXPECT_PLACED" ] \
    || fail "the click placed $((plane_after - plane_before)) blocks, expected $EXPECT_PLACED"
[ $((before - after)) -eq "$EXPECT_CONSUMED" ] \
    || fail "the click consumed $((before - after)) stone, expected $EXPECT_CONSUMED"

if [ "$COUNTERFACTUAL" = 0 ]; then
    echo "== phase 2: stairs keep a placement facing (default is facing=east) =="
    rebuild minecraft:stone_brick_stairs
    click_wall
    stairs=$(plane_stairs)
    echo "plane stairs after: $stairs"
    [ "$stairs" -eq 3 ] || fail "the click placed $stairs stairs, expected 3"
    for cell in "0 101 1" "1 101 1" "-1 101 1"; do
        # shellcheck disable=SC2086
        state=$(block_state $cell)
        echo "stairs at $cell: $state"
        case "$state" in
            *"facing=north"*) ;;
            *) fail "the stair at $cell is $state, not facing=north (a default-state write leaves facing=east)" ;;
        esac
    done

    echo "== phase 3: a cow in a plane cell blocks that cell but not the plane =="
    rebuild minecraft:stone
    ctl command "summon minecraft:cow 1.5 101.0 1.5 {NoAI:1b,NoGravity:1b,Silent:1b,PersistenceRequired:1b}" >/dev/null
    ctl wait 3
    [ "$(cows)" -eq 1 ] || fail "the cow did not spawn"
    [ "$(ctl block 1 101 1 | jq -r '.block')" = "minecraft:air" ] \
        || fail "the cow's cell is not air before the click"
    before=$(count_item minecraft:stone)
    click_wall
    after=$(count_item minecraft:stone)
    obstructed=$(ctl block 1 101 1 | jq -r '.block')
    placed=$(plane_stone)
    echo "cow cell after: $obstructed"
    echo "plane stone after: $placed"
    echo "stone after: $after (cow entities: $(cows))"
    [ "$obstructed" = "minecraft:air" ] \
        || fail "the click placed $obstructed in the cow's cell: entity obstruction was ignored"
    [ "$(cows)" -eq 1 ] || fail "the cow did not survive the click"
    [ "$placed" -ge 2 ] || fail "the obstructed click placed $placed blocks, expected the rest of the plane to fill"
    [ $((before - after)) -eq "$placed" ] \
        || fail "the obstructed click consumed $((before - after)) stone for $placed placed blocks"
fi

ctl quit >/dev/null 2>&1 || true
trap - EXIT
sleep 1
rm -rf "$SCRATCH" || true
if [ "$COUNTERFACTUAL" = 1 ]; then
    echo "PASS (counterfactual, plain minecraft:stick): 0 blocks placed, 0 stone consumed"
else
    echo "PASS: the wooden stick filled 3 blocks in the clicked face's plane and consumed 3 stone; stairs kept vanilla facing=north; a cow in a plane cell blocked only that cell"
fi
echo "EXIT:0"
