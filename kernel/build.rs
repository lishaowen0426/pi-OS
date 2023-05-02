use std::{env, fs, process};

fn main() {
    let ld_script_folder = match env::var("LD_SCRIPT_FOLDER") {
        Ok(var) => var,
        _ => process::exit(0),
    };
    println!("folder {}", ld_script_folder);

    let files = fs::read_dir(ld_script_folder).unwrap();
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
