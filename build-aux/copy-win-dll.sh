#!/bin/env bash

target=$1
OUTPUT_DIR=$2

echo "Starting DLL copy"
echo "Target: $target"
echo "Destination: $OUTPUT_DIR"

ldd $target \
  | grep -iv "windows" \
  | awk '{print $3}' \
  | while read -r dll; do
filename=$(basename "$dll")
echo "Copying: $filename"
cp "$dll" "$OUTPUT_DIR/"
done

