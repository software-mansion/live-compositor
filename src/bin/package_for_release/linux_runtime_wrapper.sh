#!/usr/bin/env bash

#
#   Runtime wrapper which provides paths to native libs used by the web renderer
#  

set -eo pipefail

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

export LD_LIBRARY_PATH="$SCRIPT_DIR/lib:$LD_LIBRARY_PATH"
export LIVE_COMPOSITOR_PROCESS_HELPER_PATH="$SCRIPT_DIR/live_compositor_process_helper"

exec "$SCRIPT_DIR/live_compositor_main" "$@"
