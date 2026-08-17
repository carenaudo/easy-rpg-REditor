#[cxx::bridge]
mod ffi {
    extern "C++" {
        include!("bridge.h");

        fn load_project(path: &str) -> Vec<String>;
        fn get_map_chipset(path: &str, map_id: i32) -> Vec<u8>;
    }
}

pub use ffi::*;
