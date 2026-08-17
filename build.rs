fn main() {
    let liblcf_path = "C:/path/to/liblcf/install"; // Change to your liblcf install path

    cxx_build::bridge("src/lcf_bridge.rs")
        .file("cpp/bridge.cpp")
        .include("cpp")
        .include(format!("{}/include", liblcf_path))
        .flag_if_supported("/std:c++17") // MSVC flag
        .compile("lcf_bridge");

    println!("cargo:rustc-link-search=native={}/lib", liblcf_path);
    println!("cargo:rustc-link-lib=static=lcf");
}
