#!/bin/bash
#check if any build errors
out=$(cargo build --release 2>&1) ec=$?
if [[ $ec != 0 ]]; then
    printf "run failed with code [ $ec ]: \n%s" "$out"
    exit 1
fi

KNOBC=~/Programming/knobc/target/release/knobc
KNV_DIR=~/Programming/knobc/tests/valid
ERR_DIR=~/Programming/knobc/tests/errors

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

    output=$("$KNOBC" build "$KNV_DIR/$name.knv" _valid_out 2>&1) rc=$?
    if [[ $rc != 0 ]]; then
        printf "Compilation failed with errors: \n%s\n" "$output"
        exit 1
    fi

    ./_valid_out
    got=$?
    rm -f ./_valid_out ./*.s

    if [ "$got" == "$expect" ]; then
        printf "${GREEN}TEST %d (%s) PASS${RESET}  expect=%s got=%s\n" "$n" "$name" "$expect" "$got"
        pass=$((pass + 1))
    else
        printf "${YELLOW}TEST %d (%s) FAIL${RESET}  expect=%s got=%s\n" "$n" "$name" "$expect" "$got"
        fail=$((fail + 1))
    fi
}

run_err_test() {
    name=$1
    expect=$2
    n=$((n + 1))

    out=$("$KNOBC" build "$ERR_DIR/$name.knv" _err_out 2>&1)
    rm -f ./_err_out ./*.s

    if [[ "$out" == *"$expect"* ]]; then
        printf "${GREEN}TEST %d (err: %s) PASS${RESET}  matched \"%s\"\n" "$n" "$name" "$expect"
        pass=$((pass + 1))
    else
        printf "${YELLOW}TEST %d (err: %s) FAIL${RESET}  wanted \"%s\"\n" "$n" "$name" "$expect"
        fail=$((fail + 1))
    fi
}

run_test fib 55
run_test operators 54
run_test exponent 128
run_test fact 120
run_test gcd 12
run_test cond 100
run_test loop 50
run_test fn 100
run_test expr 12
run_test line-comment 20
run_test block-comment 20

echo
echo "--- error tests ---"
run_err_test undeclared "use of undeclared identifier"
run_err_test undeclared-fn "undeclared function identifier"
run_err_test let-const "cannot re-assign"
run_err_test redefinition "cannot re-assign"
run_err_test extraneous-rparen "extraneous \`)\`"
run_err_test bare-elif "expected accompanying"
run_err_test bare-else "expected accompanying"
run_err_test void-in-expr "returns type \`void\`"

rm $KNOBC

echo
printf "%d passed, %d failed\n" "$pass" "$fail"
[ "$fail" -eq 0 ]
