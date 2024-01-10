#!/usr/bin/env bash

# convert a drawio file to pdf
# ./transform.sh <PDF DIR> <FILE>

PDF_DIR=$1
FILE=$2
SUBPATH=$(realpath --relative-to=$PDF_DIR/.. $FILE)


NEW_DIR="$PDF_DIR/$(dirname $SUBPATH)"
NEW_FILE="$PDF_DIR/$SUBPATH.pdf"

mkdir -p $NEW_DIR &&  
drawio $FILE -x --format=pdf -t --crop -o $NEW_FILE > /dev/null 2>&1 &&
echo "✅ $FILE"
