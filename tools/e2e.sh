#!/usr/bin/env bash
# In-game proof for Create slice 1 (0.1.0): a placed Creative Motor publishes its four
# fields in its block entity's NBT, and `/create source <pos>` reads the parsed values back.
#
# Phases:
#  A  place create:creative_motor, read `/create source`, assert the exact parsed line
#     `CREATE_SOURCE x=300 y=101 z=0 speed=16 capacity=16384 stress=0 status=Running`;
#  B  the same command on a vanilla stone refuses the id (unknown-id control in-game);
#  C  a carried item (`create:andesite_alloy`) is give-able and read back in the inventory;
#  D  the carried andesite_alloy recipe crafts in the 2x2 grid from andesite + iron nuggets.
#  E  the kinetics engine: motor (300,101,0) → shaft[axis=x] (301,101,0) → press (302,101,0)
#     publishes speed=16 stress=8 at the shaft; an unconnected shaft at (305,101,0) is idle,
#     and a press at (304,101,0) outside the component must not add stress (differential).
#
# The control run uses the unmodded Nightly game (`GAME=$CONTROL_GAME`): the command does
# not exist, phase A's line never arrives, and the leg fails (EXIT:1). `SELFTEST=1` changes
# only phase E's expected stress from 8 to 9, so the inverted assertion fails there (EXIT:1).
#
# Every mod command goes through `ctl chat` and every reply is read from `ctl events
# --since` (a sequence marker), never from the tail of the log.
set -Eeuo pipefail
export LC_ALL=C
[ -z "${CREATE_DEBUG:-}" ] || set -x

CLONE=${CLONE:-/home/adlf/Desktop/all_the_mods_10_rust_rewrite}
CTL=${CTL:-$(dirname "${GAME:-/nonexistent}")/horizon-ctl}
GAME=${GAME:-/nonexistent}
CONTROL_GAME=${CONTROL_GAME:-}
RES=${RES:-${HORIZON_RES:-/home/adlf/Desktop/horizon/.local/horizon-client/26.3}}
SELFTEST=${SELFTEST:-0}
LEG=${LEG:-}
[ "$GAME" = "$CONTROL_GAME" ] && [ -n "$CONTROL_GAME" ] && LEG=control
SOCK=${SOCK:-${E2E_TMP:-/tmp/opencode}/create-$LEG-e2e.sock}
WORLD=CreateE2E${LEG:-green}-$$
X0=300
Y=101

# Phase A's expected readout (unchanged from 0.1.0); the control game never registers the
# command, so the line never arrives. The selftest inverts only phase E's stress value.
EXPECT="CREATE_SOURCE x=$X0 y=$Y z=0 speed=16 capacity=16384 stress=0 status=Running"
EXPECT_NET="CREATE_NETWORK x=$((X0 + 1)) y=$Y z=0 speed=16 stress=8 capacity=16384 status=Running"
if [ "$SELFTEST" = 1 ]; then EXPECT_NET="CREATE_NETWORK x=$((X0 + 1)) y=$Y z=0 speed=16 stress=9 capacity=16384 status=Running"; fi

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing game: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d "${E2E_TMP:-/tmp/opencode}/create-home.XXXXXX")
export HORIZON_HOME=$SCRATCH/game HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 60 >&2 || true; exit 1; }
trap 'echo "ERR(line $LINENO): $BASH_COMMAND [rc=$?]" >&2' ERR
cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" 2>/dev/null || true
    echo "EXIT:$status"
}
trap cleanup EXIT

ctl launch --game "$GAME" --world "$WORLD" --seed 20261105 \
    --creative --offscreen --wait --resources "$RES" >/dev/null
ctl wait-ready 120 >/dev/null 2>&1 || true
ctl wait 30 >/dev/null

# The End's void at x=300, a stone floor at y=100, clear air above it.
ctl command "execute in minecraft:the_end run tp @s $X0.5 $Y 0.5" >/dev/null
for _ in $(seq 1 120); do
    ctl wait 5 >/dev/null
    [ "$(ctl block "$X0" 100 0 | jq -r '.loaded // false')" = true ] && break
done
ctl command "execute in minecraft:the_end run fill $((X0 - 8)) 100 -8 $((X0 + 8)) 100 8 minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run fill $((X0 - 8)) $Y -8 $((X0 + 8)) 108 8 minecraft:air" >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
ctl command "execute in minecraft:the_end run tp @s $X0.5 $Y 0.5" >/dev/null
ctl wait 20 >/dev/null

# The chat sequence marker: a reply is read only from events after the command was sent.
SEQ_FILE=$SCRATCH/chat.seq
: > "$SEQ_FILE"
chat_seq() { local n; n=$(cat "$SEQ_FILE" 2>/dev/null || true); printf '%s' "${n:-0}"; }

ask() { # ask NEEDLE COMMAND... -> fresh chat after a `/`-less command, polled until NEEDLE arrives
    local needle=$1; shift
    local before last got attempt
    before=$(chat_seq)
    ctl chat "/$*" >/dev/null 2>&1 || true
    got=""
    for attempt in $(seq 1 20); do
        ctl wait 3 >/dev/null 2>&1 || true
        got=$(ctl events --since "$before" 2>/dev/null || true)
        case "$got" in *"$needle"*) break ;; esac
    done
    last=$(jq -r '.last // 0' <<<"$got")
    printf '%s' "$last" > "$SEQ_FILE"
    jq -r '[.events[]? | select(.event == "chat") | .text] | join(" / ")' <<<"$got"
}

# Phase A — the published readout, parsed from the command's reply.
ctl command "execute in minecraft:the_end run setblock $X0 $Y 0 create:creative_motor[facing=south]" >/dev/null
ctl wait 12 >/dev/null
PLACE=$(ctl block "$X0" "$Y" 0 | jq -r '.block // empty')
echo "placed block: $PLACE"
REPLY=$(ask "CREATE_SOURCE" "create source $X0 $Y 0")
echo "A reply: $REPLY"
case "$REPLY" in
    *"$EXPECT"*) echo "  A: readout matches '$EXPECT'" >&2 ;;
    *) fail "A readout: expected '$EXPECT', got '$REPLY'" ;;
esac

# Phase B — an unknown id is refused, not silently reported as a Create block.
ctl command "execute in minecraft:the_end run setblock $((X0 + 4)) $Y 0 minecraft:stone" >/dev/null
ctl wait 8 >/dev/null
REPLY=$(ask "CREATE_SOURCE refused" "create source $((X0 + 4)) $Y 0")
echo "B reply: $REPLY"
case "$REPLY" in
    *"CREATE_SOURCE refused: no Create block entity"*) echo "  B: unknown id refused" >&2 ;;
    *) fail "B unknown id: expected a refusal, got '$REPLY'" ;;
esac

# Phase C — a carried item id is registered and give-able.
ctl give create:andesite_alloy 2 >/dev/null
ctl wait 8 >/dev/null
INV=$(ctl inventory | jq -r '.counts["create:andesite_alloy"] // 0')
echo "C inventory count: $INV"
[ "$INV" -ge 2 ] || fail "C item: create:andesite_alloy count $INV, expected >= 2"
echo "  C: carried item read back ($INV)" >&2

# Phase D — the carried recipe crafts in the 2x2 grid (BA / AB = 2 andesite + 2 iron nuggets).
# The inventory is cleared first so the alloy from phase C cannot satisfy the assertion.
# A renderer is required to drive a GUI, so the leg runs with `--offscreen`; a survival
# player has the 2x2 grid (creative mode opens the creative screen instead).
ctl command "gamemode survival" >/dev/null 2>&1 || true
ctl wait 2 >/dev/null 2>&1 || true
ctl command "clear @s" >/dev/null 2>&1 || true
ctl give minecraft:andesite 4 >/dev/null 2>&1 || true
ctl give minecraft:iron_nugget 4 >/dev/null 2>&1 || true
ctl wait 6 >/dev/null 2>&1 || true
ctl open-inventory >/dev/null 2>&1 || true
ctl wait 3 >/dev/null 2>&1 || true
# Hotbar slot 1 holds the nuggets, slot 0 the andesite (give order). Pick a stack up with
# a left click, then lay ONE item into each grid cell with a right click: BA / AB.
for step in "1" "100 right" "103 right" "0" "101 right" "102 right"; do
    ctl click-slot $step >/dev/null 2>&1 || true
    ctl wait 1 >/dev/null 2>&1 || true
done
ctl wait 3 >/dev/null 2>&1 || true
RESULT=$(ctl screen | jq -r '[.slots[]? | select(.slot == 200) | .item] | first // empty' 2>/dev/null || true)
echo "D grid result: $RESULT"
# Take the result and close; the alloy must appear and exactly 2 of each ingredient be consumed.
ctl click-slot 200 shift >/dev/null 2>&1 || true
ctl wait 3 >/dev/null 2>&1 || true
ctl close >/dev/null 2>&1 || true
ctl wait 3 >/dev/null 2>&1 || true
COUNTS=$(ctl inventory | jq -c '.counts // {}')
ALLoy=$(jq -r '."create:andesite_alloy" // 0' <<<"$COUNTS")
ANDESITE=$(jq -r '."minecraft:andesite" // 0' <<<"$COUNTS")
NUGGET=$(jq -r '."minecraft:iron_nugget" // 0' <<<"$COUNTS")
echo "D counts after craft: $COUNTS"
if [ "$SELFTEST" != 1 ]; then
    [ "$RESULT" = "create:andesite_alloy" ] \
        || fail "D recipe: grid result '$RESULT', expected create:andesite_alloy"
    [ "$ALLoy" -ge 1 ] || fail "D recipe: no alloy in the inventory ($COUNTS)"
    [ "$ANDESITE" -eq 2 ] && [ "$NUGGET" -eq 2 ] \
        || fail "D recipe: ingredients not consumed as declared (andesite $ANDESITE, nugget $NUGGET; expected 2 each)"
    echo "  D: crafted create:andesite_alloy; 2 andesite + 2 iron nuggets consumed" >&2
fi

# Phase E — the kinetics engine: generator → shaft → press, read through the player command path.
ctl command "execute in minecraft:the_end run setblock $X0 $Y 0 create:creative_motor[facing=east]" >/dev/null
ctl command "execute in minecraft:the_end run setblock $((X0 + 1)) $Y 0 create:shaft[axis=x]" >/dev/null
ctl command "execute in minecraft:the_end run setblock $((X0 + 2)) $Y 0 create:mechanical_press[facing=west]" >/dev/null
ctl command "execute in minecraft:the_end run setblock $((X0 + 5)) $Y 0 create:shaft[axis=x]" >/dev/null
ctl wait 20 >/dev/null
for posx in "$X0" "$((X0 + 1))" "$((X0 + 2))" "$((X0 + 5))"; do
    echo "E placed [$posx,$Y,0]: $(ctl block "$posx" "$Y" 0 | jq -c '.block // empty')"
done
REPLY=$(ask "CREATE_NETWORK" "create network $((X0 + 1)) $Y 0")
echo "E network reply: $REPLY"
case "$REPLY" in
    *"$EXPECT_NET"*) echo "  E: solved network matches '$EXPECT_NET'" >&2 ;;
    *) fail "E network: expected '$EXPECT_NET', got '$REPLY'" ;;
esac
# The neighbour's saved fields after the solve — the world/data feed an add-on reads.
PUBLISHED="CREATE_NETWORK published x=$((X0 + 1)) y=$Y z=0 speed=16 stress=8 capacity=16384 status=Running"
case "$REPLY" in
    *"$PUBLISHED"*) echo "  E: neighbour block entity publishes the solved fields" >&2 ;;
    *) fail "E published fields: expected '$PUBLISHED', got '$REPLY'" ;;
esac

# An unconnected shaft is idle, not silently reported as part of the motor's network.
REPLY=$(ask "CREATE_NETWORK" "create network $((X0 + 5)) $Y 0")
echo "E control reply: $REPLY"
case "$REPLY" in
    *"CREATE_NETWORK x=$((X0 + 5)) y=$Y z=0 speed=0 stress=0 capacity=0 status=Idle"*)
        echo "  E: unconnected shaft is Idle" >&2 ;;
    *) fail "E control: expected an idle unconnected shaft, got '$REPLY'" ;;
esac

# Differential: a press outside the component must not add stress. The shaft at X0+1 is two
# blocks away from it, so the solved component (and its stress) is unchanged.
ctl command "execute in minecraft:the_end run setblock $((X0 + 4)) $Y 0 create:mechanical_press[facing=west]" >/dev/null
ctl wait 8 >/dev/null
REPLY=$(ask "CREATE_NETWORK" "create network $((X0 + 1)) $Y 0")
echo "E differential reply: $REPLY"
case "$REPLY" in
    *"CREATE_NETWORK x=$((X0 + 1)) y=$Y z=0 speed=16 stress=8 capacity=16384 status=Running"*)
        echo "  E: out-of-component press adds no stress" >&2 ;;
    *) fail "E differential: an unconnected press changed the network, got '$REPLY'" ;;
esac

echo "PASS: Create interop surface and kinetics engine proven in game (LEG=${LEG:-green} SELFTEST=$SELFTEST)"
