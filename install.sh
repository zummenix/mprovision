#!/usr/bin/env bash

cargo install --path .

MPROVISION_PATH=$(which mprovision)
echo "Binary size before strip:"
du -h ${MPROVISION_PATH}
strip ${MPROVISION_PATH}
echo "Binary size after strip:"
du -h ${MPROVISION_PATH}
