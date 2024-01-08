#!/usr/bin/env bash

# Get all drawio images, convert them to pdf and add to the output directory 
# with the same path. 

WORK_DIR=$(dirname "$0")

function transform() {
    OUT_DIR="_drawio"
    file=$1
    directory="$OUT_DIR/$(dirname $file)"
    out_file="$OUT_DIR/$file.pdf"
    mkdir -p $directory &&
    echo "Created Directory for $file" && 
    drawio $file -x --format=pdf -t --crop -o $out_file > /dev/null 2>&1 &&
    echo "Converted to PDF $file $out_file> "
}
export -f transform

# Note exporting transform to subshell used by find -exec
find $WORK_DIR -type f -name "*.drawio" -not -name "_*" -exec bash -c 'transform "$@"' bash {} \;
