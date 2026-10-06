#!/usr/bin/env bash
# update-homebrew-formula.sh — render a formula template into the tap repo and
# push it, factored out of the three call sites that used to carry near-identical
# copies (alpha-homebrew and stable-homebrew in sign-and-publish.yml, and the
# homebrew job in backfill-release.yml).
#
# The three differed only in which formula and which binary prefix they used,
# and `set -euo pipefail` had already drifted between them: two had it, one
# did not.
#
# ENVIRONMENT (supplied by the calling step's `env:`):
#   HOMEBREW_TAP_TOKEN   push credential for the tap repo
#   HOMEBREW_TAP_REPO    owner/homebrew-<tap> of the tap to update
#   GITHUB_REPOSITORY    set by the runner; substituted for REPO_PLACEHOLDER
#
# USAGE:
#   scripts/update-homebrew-formula.sh <formula> <binary-prefix> <version> <tag> [label]
#
#   <formula>        wirerust | wirerust-a | wirerust-d | wirerust-b | wirerust-rc
#   <binary-prefix>  basename stem of the signed binaries in the CWD
#   <version>        value for VERSION_PLACEHOLDER
#   <tag>            value for TAG_PLACEHOLDER (the release tag)
#   [label]          optional channel word for the commit message, e.g. "alpha"

set -euo pipefail

if [ "$#" -lt 4 ]; then
    echo "Usage: $0 <formula> <binary-prefix> <version> <tag> [label]" >&2
    exit 2
fi

FORMULA="$1"
BIN_PREFIX="$2"
VERSION="$3"
TAG="$4"
LABEL="${5:-}"

: "${HOMEBREW_TAP_TOKEN:?HOMEBREW_TAP_TOKEN is required}"
: "${HOMEBREW_TAP_REPO:?HOMEBREW_TAP_REPO is required}"
: "${GITHUB_REPOSITORY:?GITHUB_REPOSITORY is required}"

TAP_SHORT="${HOMEBREW_TAP_REPO#*/}"
TAP_NAME="${HOMEBREW_TAP_REPO%%/*}/${TAP_SHORT#homebrew-}"

SHA256_ARM64=$(shasum -a 256 "${BIN_PREFIX}-darwin-arm64" | cut -d' ' -f1)
SHA256_AMD64=$(shasum -a 256 "${BIN_PREFIX}-darwin-amd64" | cut -d' ' -f1)

git clone "https://x-access-token:${HOMEBREW_TAP_TOKEN}@github.com/${HOMEBREW_TAP_REPO}.git" homebrew-tap-repo
cd homebrew-tap-repo

mkdir -p Formula
cp "../Formula/${FORMULA}.rb" "Formula/${FORMULA}.rb"

sed -i "s|REPO_PLACEHOLDER|${GITHUB_REPOSITORY}|g" "Formula/${FORMULA}.rb"
sed -i "s|TAP_PLACEHOLDER|${TAP_NAME}|g" "Formula/${FORMULA}.rb"
# The stable formula carries no `version` line: its tag is v<version>, so brew
# scans the version from the URL and `brew audit --strict` rejects an explicit
# duplicate. This substitution is a no-op there and load-bearing for the
# prerelease formulae, whose tags do not scan to the declared value.
sed -i "s/VERSION_PLACEHOLDER/$VERSION/g" "Formula/${FORMULA}.rb"
sed -i "s/TAG_PLACEHOLDER/$TAG/g" "Formula/${FORMULA}.rb"
sed -i "s/SHA256_ARM64_PLACEHOLDER/$SHA256_ARM64/g" "Formula/${FORMULA}.rb"
sed -i "s/SHA256_AMD64_PLACEHOLDER/$SHA256_AMD64/g" "Formula/${FORMULA}.rb"

git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"
git add "Formula/${FORMULA}.rb"
git diff --cached --quiet && echo "No changes to commit" && exit 0

if [ -n "$LABEL" ]; then
    git commit -m "Update ${FORMULA} (${LABEL}) to $VERSION"
else
    git commit -m "Update ${FORMULA} to $VERSION"
fi
git push
