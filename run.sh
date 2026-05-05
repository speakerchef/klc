#!/bin/bash
if [ $# == 0 ]; then
    echo "run: too little arguments"
    exit 1
fi

#check if any build errors
fin=Finished
out=$(cargo build --release 2>&1)
if [[ "$out" != *"$fin"* ]]; then
    printf "run: \n%s" "$out"
    exit 1
fi

KNOBC=~/Programming/knobc/target/release/knobc
MODE=$1
pmode=-p
tmode=-t
if [ "$MODE" == "$pmode" ] || [ "$MODE" == "$tmode" ]; then
    KNV_FILENAME=~/Programming/knobc/tests/valid/"$2".knv
else
    KNV_FILENAME=~/Programming/knobc/tests/valid/"$1".knv
fi

if [ "$MODE" == "$pmode" ]; then
    $KNOBC build "$KNV_FILENAME" out && ./out
    ec=$?
elif [ "$MODE" == "$tmode" ]; then
    $KNOBC build "$KNV_FILENAME" out >/dev/null 2>&1 && ./out
    ec=$?
    ~/Programming/knobc/test.sh
else
    $KNOBC build "$KNV_FILENAME" out >/dev/null 2>&1 && ./out
    ec=$?
fi

rm ./out
rm $KNOBC
printf "Exit code: %d\n" $ec
