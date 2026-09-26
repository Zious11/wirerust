class Wirerust < Formula
  # Homebrew desc audit: <= 80 chars (incl. any channel suffix), capitalized,
  # no leading article, must not start with the formula name, no trailing period.
  desc "Fast PCAP forensics and network triage CLI"
  homepage "https://github.com/REPO_PLACEHOLDER"
  # No `version` here: the stable tag is v<version>, so brew scans the
  # version from the URL and `brew audit --strict` rejects the duplicate
  # ("`version 0.13.2` is redundant with version scanned from URL").
  # The prerelease formulae keep theirs: their tags do not scan to the
  # declared value.
  license "MIT"

  # These formulae ship only Mach-O binaries: the release pipeline builds
  # darwin-arm64 and darwin-amd64 and nothing else. Without this guard a
  # Linux `brew install` falls through the Hardware::CPU.arm? else-branch,
  # downloads the darwin-amd64 Mach-O, and fails at exec time with no
  # explanation. Declaring the dependency makes brew refuse up front.
  depends_on :macos

  if Hardware::CPU.arm?
    url "https://github.com/REPO_PLACEHOLDER/releases/download/TAG_PLACEHOLDER/wirerust-darwin-arm64"
    sha256 "SHA256_ARM64_PLACEHOLDER"
  else
    url "https://github.com/REPO_PLACEHOLDER/releases/download/TAG_PLACEHOLDER/wirerust-darwin-amd64"
    sha256 "SHA256_AMD64_PLACEHOLDER"
  end

  def install
    binary_name = Hardware::CPU.arm? ? "wirerust-darwin-arm64" : "wirerust-darwin-amd64"
    bin.install binary_name => "wirerust"
  end

  test do
    assert_match "wirerust", shell_output("#{bin}/wirerust --version 2>&1")
  end
end
