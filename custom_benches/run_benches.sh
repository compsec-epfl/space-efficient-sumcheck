#!/bin/sh

algorithms="VSBW Tradeoff2 Tradeoff3 Tradeoff4 CTY"
fields="Field64 Field128 FieldBn254"

for algorithm in $algorithms; do
    for field in $fields; do
        num_vars=15
        while [ $num_vars -le 30 ]; do
            case "$algorithm" in
                "Tradeoff2") stage_size="2" ;;
                "Tradeoff3") stage_size="3" ;;
                "Tradeoff4") stage_size="4" ;;
                "VSBW") stage_size="1" ;;
                "CTY") stage_size="1" ;;
                *) ;;
            esac
            output=`(gtime ./target/release/benches $algorithm $field $num_vars $stage_size) 2>&1`
            user_time_seconds=$(echo "$output" | awk '/user/ {printf "%.2f", $2}')
            user_time_ms=$(awk "BEGIN {printf \"%.0f\", $user_time_seconds * 1000}")
            ram_bytes=$(expr "$output" : '.* \([0-9]*\)maxresident')
            echo "$algorithm, $field, $num_vars, $user_time_ms, $num_vars, $ram_bytes"
            num_vars=$((num_vars + 1))
        done
    done
done
