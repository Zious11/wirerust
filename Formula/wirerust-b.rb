class WirerustB < Formula
  desc "Fast PCAP forensics and network triage CLI (beta)"
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
    bin.install binary_name => "wirerust-b"
  end

  def caveats
    <<~EOS
      wirerust-b is the beta channel. Updates on every v*-beta.* tag.
      For stable: brew install TAP_PLACEHOLDER/wirerust
    EOS
  end

  test do
    assert_match "wirerust", shell_output("#{bin}/wirerust-b --version 2>&1")
  end
end
