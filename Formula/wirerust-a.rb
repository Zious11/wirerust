class WirerustA < Formula
  desc "Fast PCAP forensics and network triage CLI (alpha)"
  homepage "https://github.com/REPO_PLACEHOLDER"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  if Hardware::CPU.arm?
    url "https://github.com/REPO_PLACEHOLDER/releases/download/TAG_PLACEHOLDER/wirerust-a-darwin-arm64"
    sha256 "SHA256_ARM64_PLACEHOLDER"
  else
    url "https://github.com/REPO_PLACEHOLDER/releases/download/TAG_PLACEHOLDER/wirerust-a-darwin-amd64"
    sha256 "SHA256_AMD64_PLACEHOLDER"
  end

  def install
    binary_name = Hardware::CPU.arm? ? "wirerust-a-darwin-arm64" : "wirerust-a-darwin-amd64"
    bin.install binary_name => "wirerust-a"
  end

  def caveats
    <<~EOS
      wirerust-a is the alpha channel. Updates on every push to develop.
      For stable: brew install TAP_PLACEHOLDER/wirerust
    EOS
  end

  test do
    assert_match "wirerust", shell_output("#{bin}/wirerust-a --version 2>&1")
  end
end
