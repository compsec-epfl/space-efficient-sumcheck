#!/bin/sh

# ATTENTION
# You can pipe the output of this script to a file like so:
#   nohup ./run_benches.sh &> output_file.txt &
# If you need to kill the running process you can find pid with:
#   lsof | grep output_file
# Then:
#   kill <pid>

num_vars=15
while [ $num_vars -le 30 ]; do
    # NOTE: if you're on mac you might install gnu-time and change next line to "gtime"
    output=`(gtime -v ./target/release/lag-poly-benches $num_vars) 2>&1`
    user_time_seconds=$(echo "$output" | grep "User time (seconds):" | awk '{print $4}')
    user_time_ms=$(awk "BEGIN {printf \"%.0f\", $user_time_seconds * 1000}")
    ram_kilobytes=$(echo "$output" | grep "Maximum resident set size (kbytes)" | awk '{print $6}')
    ram_bytes=$(echo "$ram_kilobytes" | awk '{ printf "%.0f", $1 * 1000 }')
    echo "graycode, $num_vars, $user_time_ms, $ram_bytes"
    num_vars=$((num_vars + 1))
done
