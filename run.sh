#!/bin/bash

if [[ "$OSTYPE" == "darwin"* ]]; then
    # Build compositor
    if ! cargo build "$1"; then
        exit 1
    fi
    target="target/debug"
    if [[ "$1" == "--release" || "$1" == "-r" ]]; then
        target="target/release"
    fi

    # Create a bundle
    mkdir -p "$target/video_compositor.app/Contents/MacOS"
    mkdir -p "$target/video_compositor.app/Contents/Resources"
    cp -r "$target/Frameworks" "$target/video_compositor.app/Contents"
    cp "$target/video_compositor" "$target/video_compositor.app/Contents/MacOS"
    cp "$target/Info.plist" "$target/video_compositor.app"

   # Run compositor
   ./$target/video_compositor.app/Contents/MacOS/video_compositor ${@:2}
else
    cargo run "$@"
fi
