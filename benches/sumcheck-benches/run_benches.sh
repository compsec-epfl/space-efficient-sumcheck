#!/bin/sh

algorithms="Blendy2 VSBW" # "Blendy1 Blendy2 VSBW Blendy3 Blendy4 CTY"
fields="Field64 Field128 FieldBn254"

for algorithm in $algorithms; do
    for field in $fields; do
        num_vars=16
        while [ $num_vars -le 30 ]; do
            case "$algorithm" in
                "Blendy1") stage_size="1" ;;
                "Blendy2") stage_size="2" ;;
                "Blendy3") stage_size="3" ;;
                "Blendy4") stage_size="4" ;;
                "VSBW") stage_size="1" ;;
                "CTY") stage_size="1" ;;
                *) ;;
            esac
            case "$algorithm" in
                "Blendy1") algorithm_label="Blendy" ;;
                "Blendy2") algorithm_label="Blendy" ;;
                "Blendy3") algorithm_label="Blendy" ;;
                "Blendy4") algorithm_label="Blendy" ;;
                "VSBW") algorithm_label="VSBW" ;;
                "CTY") algorithm_label="CTY" ;;
                *) ;;
            esac
            # NOTE: mac users change "time" --> "gtime" on next line
            output=`(gtime -v ./target/release/benches $algorithm_label $field $num_vars $stage_size) 2>&1`
            user_time_seconds=$(echo "$output" | grep "User time (seconds):" | awk '{print $4}')
            user_time_ms=$(awk "BEGIN {printf \"%.0f\", $user_time_seconds * 1000}")
            ram_kilobytes=$(echo "$output" | grep "Maximum resident set size (kbytes)" | awk '{print $6}')
            ram_bytes=$(echo "$ram_kilobytes" | awk '{ printf "%.0f", $1 * 1000 }')
            echo "$algorithm, $field, $num_vars, $user_time_ms, $num_vars, $ram_bytes"
            num_vars=$((num_vars + 2))
        done
    done
done

# NOTE: helpful Unix commands
#
# 1) You can run this shell in the background while piping the output to a file like so:
#   nohup ./run_benches.sh &> output_file.txt &
#
# 2) If you need to kill the running process you can find pid with:
#   lsof | grep output_file
#  Then:
#     kill <pid>
