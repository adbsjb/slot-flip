#!/bin/sh
# Tests for tools/check-sensitive.sh. Builds a throwaway repository so the real
# one is never touched, and assembles every fake secret at runtime so this file
# is not itself a finding.
#
#   sh tools/check-sensitive-test.sh
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
scan="$here/check-sensitive.sh"
tmp=$(mktemp -d "${TMPDIR:-/tmp}/check-sensitive-test.XXXXXX")
trap 'rm -rf "$tmp"' EXIT INT TERM

cd "$tmp"
git init -q
git config user.name test
git config user.email test@example.com

out="$tmp/out"
failures=0

fail() {
    echo "FAIL: $*" >&2
    [ -f "$out" ] && sed 's/^/  /' "$out" >&2 || true
    failures=$((failures + 1))
}

expect_pass() { # description, arguments...
    desc=$1
    shift
    if ! sh "$scan" "$@" >"$out" 2>&1; then
        fail "$desc (should have passed)"
    fi
}

expect_fail() { # description, kind, arguments...
    desc=$1
    kind=$2
    shift 2
    if sh "$scan" "$@" >"$out" 2>&1; then
        fail "$desc (should have failed)"
        return
    fi
    grep -q "$kind" "$out" || fail "$desc (failed, but not as $kind)"
}

# A personal pattern the scanner only knows from the private file.
patterns="$tmp/patterns"
printf '%s\n' 'secretname42' >"$patterns"
export SENSITIVE_PATTERNS_FILE="$patterns"

printf 'ordinary text\n' >clean.txt
git add clean.txt
expect_pass "a clean tree" --all
git commit -q -m "clean"
expect_pass "a clean commit" --commit HEAD

token='ghp_'
i=0
while [ "$i" -lt 36 ]; do
    token="${token}a"
    i=$((i + 1))
done
key='-----BEGIN'" RSA"' PRIVATE KEY-----'
home='/Users/'"realperson"'/notes'

printf 'token = %s\n' "$token" >token.txt
git add token.txt
expect_fail "a GitHub token" "github-token" --all
expect_fail "a GitHub token in a commit message" "github-token" --message token.txt
git rm -q --cached token.txt && rm token.txt

printf '%s\n' "$key" >key.txt
git add key.txt
expect_fail "a private key" "private-key" --all
git rm -q --cached key.txt && rm key.txt

printf '%s\n' "$home" >home.txt
git add home.txt
expect_fail "a home directory path" "local-home-path" --all
git rm -q --cached home.txt && rm home.txt

printf '%s\n' 'owned by secretname42' >personal.txt
git add personal.txt
expect_fail "a private denylist entry" "personal" --all
if env GIT_AUTHOR_EMAIL=secretname42@example.com GIT_COMMITTER_EMAIL=secretname42@example.com \
    sh "$scan" --identity >"$out" 2>&1; then
    fail "the same entry as a commit author (should have failed)"
elif ! grep -q "personal" "$out"; then
    fail "the same entry as a commit author (failed, but not as personal)"
fi
git rm -q --cached personal.txt && rm personal.txt

printf 'token = %s  # check-sensitive: allow\n' "$token" >allowed.txt
git add allowed.txt
expect_pass "an explicitly allowed example" --all
git rm -q --cached allowed.txt && rm allowed.txt

printf 'x\n' >id_rsa
git add -f id_rsa
expect_fail "a file that is a key by name" "forbidden file name" --all
git rm -q --cached id_rsa && rm id_rsa

printf 'token = %s\n' "$token" >staged.txt
git add staged.txt
expect_fail "a staged token" "github-token" --staged
git rm -q --cached staged.txt && rm staged.txt

printf 'token = %s\n' "$token" >committed.txt
git add committed.txt
git commit -q -m "a commit with a token"
expect_fail "a commit that was just made" "github-token" --commit HEAD
git reset -q --hard 'HEAD^'
rm -f committed.txt

if [ "$failures" -ne 0 ]; then
    echo "check-sensitive-test: $failures failure(s)" >&2
    exit 1
fi
echo "check-sensitive-test: all checks passed"
