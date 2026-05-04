#!/bin/sh
cargo build --release

KNOBC=~/Programming/knobc/target/release/knobc
KNV_DIR=~/Programming/knobc/src/knv

GREEN="\033[1;32m"
YELLOW="\033[1;93m"
RESET="\033[0m"

pass=0
fail=0
n=0

run_test() {
    name=$1
    expect=$2
    n=$((n + 1))

    "$KNOBC" build "$KNV_DIR/$name.knv" "$name" >/dev/null 2>&1
    ./"$name"
    got=$?
    rm -f ./"$name" ./*.s

    if [ "$got" = "$expect" ]; then
        printf "${GREEN}TEST %d (%s) PASS${RESET}  expect=%s got=%s\n" "$n" "$name" "$expect" "$got"
        pass=$((pass + 1))
    else
        printf "${YELLOW}TEST %d (%s) FAIL${RESET}  expect=%s got=%s\n" "$n" "$name" "$expect" "$got"
        fail=$((fail + 1))
    fi
}

run_test fib 55
run_test power 128
run_test fact 120
run_test gcd 12
run_test cond 100
run_test fn 100
run_test expr 12

echo
printf "%d passed, %d failed\n" "$pass" "$fail"
[ "$fail" -eq 0 ]
