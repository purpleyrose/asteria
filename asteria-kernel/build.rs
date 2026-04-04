fn main() {
    cc::Build::new()
        .file("src/boot.s")
        .compile("boot");
    println!("cargo:rustc-link-arg=-Tkernel.ld");
}
