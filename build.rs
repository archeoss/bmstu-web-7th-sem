#[allow(unused_imports)]
use std::{path::PathBuf, process::Command};

#[allow(dead_code)]
fn main() {
    #[cfg(feature = "frontend-bundle")]
    {
        let mut dir = PathBuf::from(std::env::var("OUT_DIR").unwrap()); // OUT_DIR == <project_dir>/target/<backend_dir>/build/bob_management-<HASH>/out
        dir.pop();
        dir.pop();
        dir.pop();
        let mut project_dir = dir.clone();
        project_dir.pop();
        project_dir.pop();
        println!("cargo:warning=Moving /dist to: {dir:?}");
        println!("cargo:warning=PROJECT DIR: {project_dir:?}");
        println!(
            "cargo:warning=cp: {:?}",
            Command::new("cp")
                .args([
                    "-rf",
                    &format!("{}/frontend/dist", project_dir.to_string_lossy()),
                    dir.to_str().unwrap()
                ])
                .output()
                .expect("Couldn't move frontend build artifacts")
        );
        println!("cargo:rerun-if-changed=frontend");
    }
    println!("cargo:rerun-if-changed=NULL");
}
