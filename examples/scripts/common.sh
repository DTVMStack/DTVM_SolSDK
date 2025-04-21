#!/bin/bash

function run_cmd_and_grep() {
    # run command, echo result, and grep $grep. if exit, not run continue
    local exit_code=$?
    local output="$1"
    local grep_pattern="$2"

    # Echo the output
    echo "$output"

    # Check if the command was successful
    if [ $exit_code -ne 0 ]; then
        echo "Command failed with exit code $exit_code"
        exit $exit_code
    fi

    # Check if the output matches the grep pattern
#    echo "$output" | grep -E -zo "$grep_pattern"
    echo "matching pattern: $grep_pattern"
    echo "$output" | awk -v pattern="$grep_pattern" 'BEGIN { RS="\0" } $0 ~ pattern { found=1 } END { if (!found) exit 1 }'
    echo "grep pattern matched"
}

function grep_output_last_result_address() {
    # grep the last result address from the output
    local output="$1"
    # Find the last line with the result hex using awk
    local last_line=$(echo "$output" | awk '/evm (finish|revert) with result hex:/ {last_match = $0} END {print last_match}')
    # Extract the address (last field, skipping first 24 chars)
    local result_hex=$(echo "$last_line" | awk '{print substr($NF, 25)}')
    echo "$result_hex"
}

function grep_output_last_result_hex() {
    # grep the last result hex from the output
    local output="$1"
    # Find the last line with the result hex using awk
    local last_line=$(echo "$output" | awk '/evm (finish|revert) with result hex:/ {last_match = $0} END {print last_match}')
    # Extract the hex value (last field) from that line
    local result_hex=$(echo "$last_line" | awk '{print $NF}')
    echo "$result_hex"
}
