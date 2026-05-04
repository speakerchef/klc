#!/bin/sh
cargo build --release
KNOBC=~/Programming/knobc/target/release/knobc
MODE=$1
pmode=-p
tmode=-t
if [ "$MODE" = "$pmode" ] || [ "$MODE" = "$tmode" ]; then
    KNV_FILENAME=~/Programming/knobc/src/knv/"$2".knv
else
    KNV_FILENAME=~/Programming/knobc/src/knv/"$1".knv
fi

if [ "$MODE" = "$pmode" ]; then
    $KNOBC build "$KNV_FILENAME" out && ./out
    EC=$?
elif [ "$MODE" = "$tmode" ]; then
    $KNOBC build "$KNV_FILENAME" out >/dev/null && ./out
    EC=$?
    ~/Programming/knobc/test.sh
else
    $KNOBC build "$KNV_FILENAME" out >/dev/null && ./out
    EC=$?
fi
rm ./out
printf "Exit code: %d\n" $EC
