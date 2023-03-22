fn main() {
    println!("cargo:rustc-link-lib=z");
    println!("cargo:rustc-link-lib=zen");
    println!("cargo:rustc-link-lib=mediainfo");
}
