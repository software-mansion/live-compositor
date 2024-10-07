fn main() {
    cfg_aliases::cfg_aliases! {
            vulkan: {
                any(
                    windows,
                    all(
                        unix,
                        not(any(target_os = "macos", target_os = "ios", target_os = "emscripten"))
                    )
                )
        },
    }
}
