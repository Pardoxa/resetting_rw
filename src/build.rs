use std::process::Command;

const UNKNOWN: &str = "UNKNOWN_GIT_HASH";

fn main() {
    // note: add error checking yourself.
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output();
    let git_hash = match output{
        Ok(output) => {
            match String::from_utf8(output.stdout){
                Ok(git_hash) => git_hash,
                _ => UNKNOWN.to_owned()
            }
        },
        _ => {
            UNKNOWN.to_owned()
        }
    };
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=BUILD_TIME_CHRONO={}", chrono::offset::Local::now());
}