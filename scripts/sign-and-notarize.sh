#!/usr/bin/env bash
# sign-and-notarize.sh — Apple codesign / notarize / verify steps, factored out
# of the three call sites that used to carry byte-identical copies.
#
# WHY A SUBCOMMAND SCRIPT AND NOT ONE FUNCTION (or a composite action):
# each phase runs as its own workflow step so that its secrets stay scoped to
# that step. The notarization credentials are in scope for `notarize` and
# nowhere else; the signing identity is in scope for `sign-binaries` and
# `package` and nowhere else. Collapsing the phases into a single entry point
# would put every secret in scope for every phase, which is a larger change
# than the duplication it removes. So the workflow keeps its step boundaries
# and each step calls one subcommand.
#
# CALL SITES:
#   .github/workflows/sign-and-publish.yml   alpha-sign  (prefix wirerust-a)
#   .github/workflows/sign-and-publish.yml   stable-sign (prefix wirerust)
#   .github/workflows/backfill-release.yml   sign        (prefix wirerust)
# The three differ only in the binary-name prefix. Before this script the
# stable and backfill copies were byte-identical and the alpha copy differed
# by the prefix and one comment, and `set -euo pipefail` had already drifted
# between them.
#
# ENVIRONMENT (supplied by the calling step's `env:`, never read from args):
#   import-certs   APPLE_CERTIFICATE_P12, APPLE_CERTIFICATE_PASSWORD,
#                  APPLE_INSTALLER_CERTIFICATE_P12,
#                  APPLE_INSTALLER_CERTIFICATE_PASSWORD
#   sign-binaries  APPLE_SIGNING_IDENTITY
#   package        APPLE_SIGNING_IDENTITY, APPLE_INSTALLER_IDENTITY
#   notarize       APPLE_NOTARIZATION_APPLE_ID, APPLE_NOTARIZATION_PASSWORD,
#                  APPLE_NOTARIZATION_TEAM_ID
#   verify         (none)
#   checksums      (none)
#
# USAGE:
#   scripts/sign-and-notarize.sh import-certs
#   scripts/sign-and-notarize.sh sign-binaries <prefix>
#   scripts/sign-and-notarize.sh package       <prefix> <version>
#   scripts/sign-and-notarize.sh notarize      <prefix>
#   scripts/sign-and-notarize.sh verify        <prefix>
#   scripts/sign-and-notarize.sh checksums     <prefix>
#
# <prefix> is the binary basename stem: `wirerust` or `wirerust-a`.

set -euo pipefail

usage() {
    sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//;$d' >&2
    exit 2
}

require_prefix() {
    if [ -z "${1:-}" ]; then
        echo "ERROR: subcommand requires a <prefix> argument (wirerust or wirerust-a)" >&2
        exit 2
    fi
}

cmd_import_certs() {
    security create-keychain -p "" build.keychain
    security default-keychain -s build.keychain
    security unlock-keychain -p "" build.keychain

    echo "$APPLE_CERTIFICATE_P12" | base64 --decode > cert.p12
    security import cert.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
    rm cert.p12

    echo "$APPLE_INSTALLER_CERTIFICATE_P12" | base64 --decode > installer-cert.p12
    security import installer-cert.p12 -k build.keychain -P "$APPLE_INSTALLER_CERTIFICATE_PASSWORD" -T /usr/bin/pkgbuild -T /usr/bin/productbuild -T /usr/bin/productsign
    rm installer-cert.p12

    curl -sfo /tmp/DeveloperIDG2CA.cer https://www.apple.com/certificateauthority/DeveloperIDG2CA.cer
    security add-certificates -k build.keychain /tmp/DeveloperIDG2CA.cer
    rm /tmp/DeveloperIDG2CA.cer

    security set-key-partition-list -S apple-tool:,apple: -s -k "" build.keychain
}

cmd_sign_binaries() {
    local prefix="$1"
    codesign --force --options runtime --sign "$APPLE_SIGNING_IDENTITY" --timestamp "${prefix}-darwin-arm64"
    codesign --force --options runtime --sign "$APPLE_SIGNING_IDENTITY" --timestamp "${prefix}-darwin-amd64"
    codesign --verify --deep --strict "${prefix}-darwin-arm64"
    codesign --verify --deep --strict "${prefix}-darwin-amd64"
}

cmd_package() {
    local prefix="$1" version="$2"
    chmod +x scripts/create-app.sh scripts/create-dmg.sh scripts/create-pkg.sh

    for arch in arm64 amd64; do
        ./scripts/create-app.sh "${prefix}-darwin-${arch}" "$version" .
        codesign --force --deep --options runtime --sign "$APPLE_SIGNING_IDENTITY" --timestamp Wirerust.app
        ./scripts/create-dmg.sh Wirerust.app "$version" "${prefix}-${arch}.dmg"
        # Sign the DMG container itself. Required for stapler to attach a
        # Gatekeeper-recognized notarization ticket, and routes the
        # notarytool submission through Apple's fast path (signed-image
        # validation) instead of the slow "discovery" path that hangs at
        # pre-submission under burst load (observed in run 27797831466).
        codesign --force --sign "$APPLE_SIGNING_IDENTITY" --timestamp "${prefix}-${arch}.dmg"
        ./scripts/create-pkg.sh "${prefix}-darwin-${arch}" "$version" "$APPLE_INSTALLER_IDENTITY" "${prefix}-${arch}.pkg"
        rm -rf Wirerust.app
    done
}

cmd_notarize() {
    local prefix="$1"
    for ARTIFACT in "${prefix}-arm64.pkg" "${prefix}-amd64.pkg" "${prefix}-arm64.dmg" "${prefix}-amd64.dmg"; do
        echo "Notarizing $ARTIFACT..."
        xcrun notarytool submit "$ARTIFACT" \
            --apple-id "$APPLE_NOTARIZATION_APPLE_ID" \
            --password "$APPLE_NOTARIZATION_PASSWORD" \
            --team-id "$APPLE_NOTARIZATION_TEAM_ID" \
            --wait --timeout 14400
        xcrun stapler staple "$ARTIFACT"
    done
}

cmd_verify() {
    local prefix="$1"
    local CS_OUT SPCTL_OUT
    CS_OUT=$(mktemp)
    SPCTL_OUT=$(mktemp)
    trap 'rm -f "$CS_OUT" "$SPCTL_OUT"' EXIT
    # Bare Mach-O binaries: stapler can't attach to a bare binary
    # (Apple TN3147), so `spctl --assess --type execute` would
    # report "Unnotarized Developer ID". Verify the load-bearing
    # properties directly via codesign: Developer ID Application
    # identity, stable Team Identifier, and hardened runtime flag.
    for BIN in "${prefix}-darwin-arm64" "${prefix}-darwin-amd64"; do
        echo "::group::Verify $BIN"
        codesign -dvv "$BIN" 2>&1 | tee "$CS_OUT"
        # GHA log-masks the leaf cert CN (it matches APPLE_SIGNING_IDENTITY
        # secret), so `Authority=Developer ID Application: ...` becomes
        # `Authority=***`. Anchor instead on the intermediate cert in the
        # Developer ID chain, which is a public Apple CA name and never
        # masked. Its presence proves the chain.
        grep -q "^Authority=Developer ID Certification Authority$" "$CS_OUT" \
            || { echo "::error::$BIN missing Developer ID chain (intermediate CA)"; exit 1; }
        # TeamIdentifier value matches APPLE_NOTARIZATION_TEAM_ID and is
        # masked to "***" — accept either a real team-id format or the
        # masked marker, but reject "not set" (the ad-hoc sentinel).
        grep -qE "^TeamIdentifier=([A-Z0-9]{6,}|\*+)$" "$CS_OUT" \
            || { echo "::error::$BIN: TeamIdentifier missing, ad-hoc, or unexpected format"; exit 1; }
        grep -qE "^CodeDirectory.*flags=0x[0-9a-f]+\(.*runtime.*\)" "$CS_OUT" \
            || { echo "::error::$BIN missing hardened runtime (--options runtime) flag"; exit 1; }
        echo "::endgroup::"
    done
    # Stapled containers: spctl --assess returns "accepted
    # source=Notarized Developer ID" when signed + notarized +
    # stapled. .pkg → --type install; .dmg → --type open.
    for PKG in "${prefix}-arm64.pkg" "${prefix}-amd64.pkg"; do
        echo "::group::Verify $PKG"
        spctl --assess --type install --verbose=4 "$PKG" 2>&1 | tee "$SPCTL_OUT"
        grep -q "source=Notarized Developer ID" "$SPCTL_OUT" \
            || { echo "::error::$PKG not notarized (spctl source unexpected)"; exit 1; }
        echo "::endgroup::"
    done
    # `spctl --assess --type open` on a notarized .dmg returns
    # "rejected: source=Insufficient Context" on macOS 15+ — Gatekeeper
    # assesses the *mounted* contents, not the .dmg file. `stapler
    # validate` is the canonical check: it verifies the notarization
    # ticket is locally attached to the .dmg and references a valid
    # Apple notarization record. Non-zero exit on missing/invalid
    # staple fails the step.
    for DMG in "${prefix}-arm64.dmg" "${prefix}-amd64.dmg"; do
        echo "::group::Verify $DMG"
        xcrun stapler validate "$DMG"
        echo "::endgroup::"
    done
}

cmd_checksums() {
    local prefix="$1"
    for f in "${prefix}-darwin-arm64" "${prefix}-darwin-amd64" \
             "${prefix}-arm64.pkg" "${prefix}-amd64.pkg" \
             "${prefix}-arm64.dmg" "${prefix}-amd64.dmg"; do
        shasum -a 256 "$f" > "${f}.sha256"
    done
}

case "${1:-}" in
    import-certs)  cmd_import_certs ;;
    sign-binaries) require_prefix "${2:-}"; cmd_sign_binaries "$2" ;;
    package)       require_prefix "${2:-}"
                   [ -n "${3:-}" ] || { echo "ERROR: package requires <prefix> <version>" >&2; exit 2; }
                   cmd_package "$2" "$3" ;;
    notarize)      require_prefix "${2:-}"; cmd_notarize "$2" ;;
    verify)        require_prefix "${2:-}"; cmd_verify "$2" ;;
    checksums)     require_prefix "${2:-}"; cmd_checksums "$2" ;;
    *)             usage ;;
esac
