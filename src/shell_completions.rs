//! Shell completion generation for cqlsh-rs CLI arguments.
//!
//! Generates completion scripts for bash, zsh, fish, elvish, and PowerShell
//! using clap_complete. Usage:
//!
//! ```sh
//! # Generate bash completions
//! cqlsh-rs --completions bash > /etc/bash_completion.d/cqlsh-rs
//!
//! # Generate zsh completions
//! cqlsh-rs --completions zsh > ~/.zfunc/_cqlsh-rs
//!
//! # Generate fish completions
//! cqlsh-rs --completions fish > ~/.config/fish/completions/cqlsh-rs.fish
//! ```

use clap::CommandFactory;
use clap_complete::{generate as gen_complete, Shell};

use crate::cli::CliArgs;

/// Generate shell completion script for the given shell and write to stdout.
pub fn generate(shell: Shell) {
    let mut cmd = CliArgs::command();
    gen_complete(shell, &mut cmd, "cqlsh-rs", &mut std::io::stdout());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_bash_completions_does_not_panic() {
        let mut cmd = CliArgs::command();
        let mut buf = Vec::new();
        gen_complete(Shell::Bash, &mut cmd, "cqlsh-rs", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("cqlsh-rs"));
    }

    #[test]
    fn generate_zsh_completions_does_not_panic() {
        let mut cmd = CliArgs::command();
        let mut buf = Vec::new();
        gen_complete(Shell::Zsh, &mut cmd, "cqlsh-rs", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("cqlsh-rs"));
    }

    #[test]
    fn generate_fish_completions_does_not_panic() {
        let mut cmd = CliArgs::command();
        let mut buf = Vec::new();
        gen_complete(Shell::Fish, &mut cmd, "cqlsh-rs", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("cqlsh-rs"));
    }

    #[test]
    fn generate_powershell_completions_does_not_panic() {
        let mut cmd = CliArgs::command();
        let mut buf = Vec::new();
        gen_complete(Shell::PowerShell, &mut cmd, "cqlsh-rs", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("cqlsh-rs"));
    }

    #[test]
    fn generate_elvish_completions_does_not_panic() {
        let mut cmd = CliArgs::command();
        let mut buf = Vec::new();
        gen_complete(Shell::Elvish, &mut cmd, "cqlsh-rs", &mut buf);
        let output = String::from_utf8(buf).unwrap();
        assert!(output.contains("cqlsh-rs"));
    }
}
