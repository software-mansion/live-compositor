#!/bin/bash

function create_helper_apple_bundle {
    local name=$1
    local bundle_id=$2
    local path="$target/video_compositor.app/Contents/Frameworks"

    mkdir -p "$path/$name.app/Contents/MacOS"
    mkdir -p "$path/$name.app/Contents/Resources"
    cp "$target/process_helper" "$path/$name.app/Contents/MacOS/$name"
    cp "$target/resources/helper-Info.plist" "$path/$name.app/Info.plist"
    sed -i '' "s/\${EXECUTABLE_NAME}/$name/g" "$path/$name.app/Info.plist"
    sed -i '' "s/\${BUNDLE_ID_SUFFIX}/$bundle_id/g" "$path/$name.app/Info.plist"
}

if [[ "$OSTYPE" == "darwin"* ]]; then
    # Build compositor
    if ! cargo build "$1" --all; then
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
    cp "$target/resources/Info.plist" "$target/video_compositor.app"

    create_helper_apple_bundle "video_compositor Helper" ""
    create_helper_apple_bundle "video_compositor Helper (Alerts)" ".alerts"
    create_helper_apple_bundle "video_compositor Helper (GPU)" ".gpu"
    create_helper_apple_bundle "video_compositor Helper (Plugin)" ".plugin"
    create_helper_apple_bundle "video_compositor Helper (Renderer)" ".renderer"

   # Run compositor
   ./$target/video_compositor.app/Contents/MacOS/video_compositor ${@:2}
else
    cargo run "$@"
fi
