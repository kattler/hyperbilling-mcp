use std::path::Path;
use std::process::Command;

fn output(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

fn main() {
    let commit =
        output("git", &["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "unknown".to_string());
    let date = output("date", &["-u", "+%Y-%m-%d"]).unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_COMMIT={commit}");
    println!("cargo:rustc-env=BUILD_DATE={date}");

    let git_path = |name: &str| {
        output(
            "git",
            &["rev-parse", "--path-format=absolute", "--git-path", name],
        )
    };
    let mut watched = vec![git_path("HEAD")];
    if let Some(branch) = output("git", &["symbolic-ref", "-q", "HEAD"]) {
        watched.push(git_path(&branch));
    }
    for path in watched.into_iter().flatten() {
        if Path::new(&path).exists() {
            println!("cargo:rerun-if-changed={path}");
        }
    }
    println!("cargo:rerun-if-changed=build.rs");

    for asset in ["web/dist/app.js", "web/dist/app.css"] {
        println!("cargo:rerun-if-changed={asset}");
        assert!(
            Path::new(asset).exists(),
            "{asset} is missing, build the documentation page with `make web`"
        );
    }
}
