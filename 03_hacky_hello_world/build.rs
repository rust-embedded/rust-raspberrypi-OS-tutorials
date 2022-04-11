use std::{env, fs};

fn main() {
    let ld_script_path = env::var("LD_SCRIPT_PATH").unwrap_or_default();

    let files = fs::read_dir(ld_script_path).unwrap();
    files
        .filter_map(Result::ok)
        .filter(|d| {
            if let Some(e) = d.path().extension() {
                e == "ld"
            } else {
                false
            }
        })
        .for_each(|f| println!("cargo:rerun-if-changed={}", f.path().display()));
}
