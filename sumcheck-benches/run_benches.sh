#!/bin/sh

# ATTENTION
# You can pipe the output of this script to a file like so:
#   nohup ./run_benches.sh &> output_file.txt &
# If you need to kill the running process you can find pid with:
#   lsof | grep output_file
# Then:
#   kill <pid>

algorithms="Tradeoff2 VSBW Tradeoff3 Tradeoff4 Tradeoff1 CTY"
fields="Field64 Field128 FieldBn254"

for algorithm in $algorithms; do
    for field in $fields; do
        num_vars=15
        while [ $num_vars -le 30 ]; do
            case "$algorithm" in
                "Tradeoff1") stage_size="1" ;;
                "Tradeoff2") stage_size="2" ;;
                "Tradeoff3") stage_size="3" ;;
                "Tradeoff4") stage_size="4" ;;
                "VSBW") stage_size="1" ;;
                "CTY") stage_size="1" ;;
                *) ;;
            esac
            case "$algorithm" in
                "Tradeoff1") algorithm_label="Tradeoff" ;;
                "Tradeoff2") algorithm_label="Tradeoff" ;;
                "Tradeoff3") algorithm_label="Tradeoff" ;;
                "Tradeoff4") algorithm_label="Tradeoff" ;;
                "VSBW") algorithm_label="VSBW" ;;
                "CTY") algorithm_label="CTY" ;;
                *) ;;
            esac
            output=`(time -v ./target/release/benches $algorithm_label $field $num_vars $stage_size) 2>&1`
            user_time_seconds=$(echo "$output" | grep "User time (seconds):" | awk '{print $4}')
            user_time_ms=$(awk "BEGIN {printf \"%.0f\", $user_time_seconds * 1000}")
            ram_kilobytes=$(echo "$output" | grep "Maximum resident set size (kbytes)" | awk '{print $6}')
            ram_bytes=$(echo "$ram_kilobytes" | awk '{ printf "%.0f", $1 * 1000 }')
            echo "$algorithm, $field, $num_vars, $user_time_ms, $num_vars, $ram_bytes"
            num_vars=$((num_vars + 1))
        done
    done
done
