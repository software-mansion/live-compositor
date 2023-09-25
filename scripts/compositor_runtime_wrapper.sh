#!/usr/bin/env bash

set -eo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

export LD_LIBRARY_PATH="$SCRIPT_DIR/lib:$LD_LIBRARY_PATH"
export MEMBRANE_VIDEO_COMPOSITOR_PROCESS_HELPER_PATH="$SCRIPT_DIR/video_compositor_process_helper"

exec "$SCRIPT_DIR/video_compositor_main" "$@"
