fn main() {
    cc::Build::new()
        .file("src/boot.s")
        .compile("boot");
    println!("cargo:rustc-link-arg=-T{}/kernel.ld", std::env::var("CARGO_MANIFEST_DIR").unwrap());
}
