#!/usr/bin/env bash

set -eo pipefail
set -x

sleep 2

exec "$MEMBRANE_VIDEO_COMPOSITOR_MAIN_EXECUTABLE_PATH"
