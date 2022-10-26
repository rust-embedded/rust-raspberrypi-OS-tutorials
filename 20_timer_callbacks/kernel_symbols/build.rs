use std::{env, path::Path};

fn main() {
    if let Ok(path) = env::var("KERNEL_SYMBOLS_DEMANGLED_RS") {
        if Path::new(&path).exists() {
            println!("cargo:rustc-cfg=feature=\"generated_symbols_available\"")
        }
    }

    println!(
        "cargo:rerun-if-changed={}",
        Path::new("kernel_symbols.ld").display()
    );
}
