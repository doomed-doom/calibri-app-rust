use std::fs::{read_to_string, write};

fn main() {
    println!("cargo:rustc-link-lib=neurosdk2");
    println!("cargo:rustc-link-search=native=/usr/lib");

    let out_path = "src/core/bindings.rs";
    let allows: String = String::from(
        "#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, dead_code, unused)]\n",
    );

    bindgen::builder()
        .header("/usr/include/libneurosdk2/sdk_api.h")
        .clang_arg("-x")
        .clang_arg("c++")
        .clang_arg("-std=c++17")
        .clang_arg("-I/usr/include/libneurosdk2")
        .generate()
        .expect("bindgen failed")
        .write_to_file(out_path)
        .unwrap();

    let mut contents = read_to_string(out_path).expect("Failed to read bindings.rs");

    if !contents.starts_with(&allows) {
        contents = allows + &contents;
        write(out_path, contents).expect("Failed to write modified bindings.rs");
    }
}
