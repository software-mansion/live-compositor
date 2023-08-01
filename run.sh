#!/bin/bash

target="target/debug"
if [[ "$1" == "--release" || "$1" == "-r" ]]; then
    target="target/release"
fi

# Download CEF
if [[ -z "$CEF_ROOT" || ! -d "$CEF_ROOT" ]]; then
    export CEF_ROOT=$(pwd)/$target/cef_root
    
    if [[ ! -d "$CEF_ROOT" ]]; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            if [[ "uname -m" == "x86_64" ]]; then
                platform="macosx64"
            else
                platform="macosarm64"
            fi
        elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
            platform="linux64"
        else
            echo "Unsupported platform"
            exit 1
        fi

        echo "Downloading CEF..."
        wget https://cef-builds.spotifycdn.com/cef_binary_115.3.11%2Bga61da9b%2Bchromium-115.0.5790.114_$platform.tar.bz2 -O $target/cef.tar.bz2
        mkdir -p $target/cef_root

        echo "Extracting..."
        tar -xvf $target/cef.tar.bz2 -C $target/cef_root --strip-components=1
    fi
fi

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

# Build compositor
if ! cargo build "$1" --all; then
    exit 1
fi

if [[ "$OSTYPE" == "darwin"* ]]; then
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
elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "TODO: Bundle linux"
else
    echo "Platform not supported"
    exit 1
fi
