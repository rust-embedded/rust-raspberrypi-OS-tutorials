use std::env;

fn main() {
    let linker_file = env::var("LINKER_FILE").unwrap();

    println!("cargo:rerun-if-changed={}", linker_file);
}
