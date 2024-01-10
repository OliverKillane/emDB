#!/usr/bin/env bash

# Get all drawio images, convert them to pdf and add to the output directory 
# with the same path. 

# from the reports directory
TOOLS_DIR=$(realpath $(dirname "$0"))
WORK_DIR=$(realpath $TOOLS_DIR/..)
DRAWIO_DIR="$WORK_DIR/_drawio"
CORES=32

# Remove old directory
echo "Removing old directory $DRAWIO_DIR" &&
rm -rf $DRAWIO_DIR &&
mkdir $DRAWIO_DIR &&

# Search for drawio files and convert them in parallel
find $WORK_DIR -type f -name "*.drawio" -not -name "_*" -print | xargs --max-procs $CORES -I {} /bin/bash -c "$TOOLS_DIR/transform.sh $DRAWIO_DIR {}"