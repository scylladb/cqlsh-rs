/// Build script that captures the git revision at compile time.
///
/// Sets the following environment variables for use with `env!()`:
/// - `CQLSH_GIT_SHA`  — short (7-char) commit hash, or "unknown"
/// - `CQLSH_GIT_DIRTY` — "true" if the working tree has uncommitted changes
use std::process::Command;

fn main() {
    // Re-run when the git HEAD changes (new commit, checkout, branch switch, etc.)
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
    println!("cargo:rerun-if-changed=.git/index");

    let sha = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Only tracked-file changes are considered "dirty" (matching the standard
    // convention used by `git describe --dirty`). Untracked files are ignored.
    let dirty = Command::new("git")
        .args(["diff-index", "--quiet", "HEAD", "--"])
        .status()
        .map(|s| !s.success())
        .unwrap_or(false);

    println!("cargo:rustc-env=CQLSH_GIT_SHA={sha}");
    println!("cargo:rustc-env=CQLSH_GIT_DIRTY={dirty}");
}
