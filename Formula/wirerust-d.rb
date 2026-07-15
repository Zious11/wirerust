class WirerustD < Formula
  desc "Fast PCAP forensics and network triage CLI tool written in Rust (dev ch)"
  homepage "https://github.com/REPO_PLACEHOLDER"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  if Hardware::CPU.arm?
    url "https://github.com/REPO_PLACEHOLDER/releases/download/TAG_PLACEHOLDER/wirerust-darwin-arm64"
    sha256 "SHA256_ARM64_PLACEHOLDER"
  else
    url "https://github.com/REPO_PLACEHOLDER/releases/download/TAG_PLACEHOLDER/wirerust-darwin-amd64"
    sha256 "SHA256_AMD64_PLACEHOLDER"
  end

  def install
    binary_name = Hardware::CPU.arm? ? "wirerust-darwin-arm64" : "wirerust-darwin-amd64"
    bin.install binary_name => "wirerust-d"
  end

  def caveats
    <<~EOS
      wirerust-d is the dev channel. Updates on every v*-dev.* tag.
      For stable: brew install TAP_PLACEHOLDER/wirerust
    EOS
  end

  test do
    assert_match "wirerust", shell_output("#{bin}/wirerust-d --version 2>&1")
  end
end
