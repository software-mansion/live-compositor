#!/usr/bin/env bash

set -eo pipefail
set -x

source ~/.cargo/env

sudo dbus-daemon --system
sudo Xvfb :99 -screen 0 640x480x8 -nolisten tcp &

sleep 2

exec "$MEMBRANE_VIDEO_COMPOSITOR_MAIN_EXECUTABLE_PATH"
