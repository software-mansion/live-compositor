#!/usr/bin/env bash

set -eo pipefail

source ~/.nvm/nvm.sh
source ~/.cargo/env

sudo dbus-daemon --system
sudo Xvfb :99 -screen 0 640x480x8 -nolisten tcp &

sleep 1

video_compositor
