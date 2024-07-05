fn main() {
    if !cfg!(target_os = "linux") {
        return
    }
    let profile = std::env::var("PROFILE").unwrap();

    let mut bridge = cxx_build::bridges(["src/api.rs", "src/enums.rs"]);

    bridge
        .file("cpp/api.cpp")
        .file("cpp/enums.cpp")
        .file("cpp/callback.cpp")
        .file("decklink_sdk/include/DeckLinkAPIDispatch.cpp");

    if profile == "debug" {
        bridge.flag("-O");
    }

    bridge.std("c++20").compile("decklink-bridge");

    println!("cargo:rerun-if-changed=src/api.rs");
    println!("cargo:rerun-if-changed=src/enums.rs");
    println!("cargo:rerun-if-changed=cpp/api.h");
    println!("cargo:rerun-if-changed=cpp/api.cpp");
    println!("cargo:rerun-if-changed=cpp/callback.h");
    println!("cargo:rerun-if-changed=cpp/callback.cpp");
    println!("cargo:rerun-if-changed=cpp/enums.h");
    println!("cargo:rerun-if-changed=cpp/enums.cpp");
}
