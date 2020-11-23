use std::env;

fn main() {
    let linker_file = env::var("LINKER_FILE").unwrap();

    // Tells Cargo to run again if the file or directory at $path changes.
    println!("cargo:rerun-if-changed={}", linker_file);
}
