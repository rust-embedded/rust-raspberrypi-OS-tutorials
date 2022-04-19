use std::{env, fs, process};

fn main() {
    let ld_script_path = match env::var("LD_SCRIPT_PATH") {
        Ok(var) => var,
        _ => process::exit(0),
    };

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
