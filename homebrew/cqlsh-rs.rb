# Homebrew formula for cqlsh-rs
# To use: brew tap scylladb/cqlsh-rs && brew install cqlsh-rs
# Or copy this formula to your own homebrew-tap repository.
#
# After each release, update the `url`, `version`, and `sha256` fields
# for each platform. A CI job or script can automate this via:
#   brew bump-formula-pr --url=<new_url> --sha256=<new_sha>

class CqlshRs < Formula
  desc "A Rust re-implementation of the Apache Cassandra cqlsh shell"
  homepage "https://github.com/scylladb/cqlsh-rs"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/scylladb/cqlsh-rs/releases/download/v#{version}/cqlsh-rs-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/scylladb/cqlsh-rs/releases/download/v#{version}/cqlsh-rs-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/scylladb/cqlsh-rs/releases/download/v#{version}/cqlsh-rs-#{version}-aarch64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_ARM64"
    else
      url "https://github.com/scylladb/cqlsh-rs/releases/download/v#{version}/cqlsh-rs-#{version}-x86_64-unknown-linux-musl.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end
  end

  def install
    bin.install "cqlsh-rs"
  end

  test do
    assert_match "cqlsh-rs", shell_output("#{bin}/cqlsh-rs --version")
  end
end
