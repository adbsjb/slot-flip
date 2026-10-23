#!/bin/sh
# Refuse to commit, push or publish secrets and personal data.
#
# Run by .githooks/pre-commit, .githooks/pre-push and .githooks/commit-msg, and
# by .github/workflows/secrets.yml. It only reports where a problem is, never the
# value itself, so a leak does not end up in a terminal log or CI output.
#
#   tools/check-sensitive.sh --all          every tracked file (default)
#   tools/check-sensitive.sh --staged       lines added to the index (pre-commit)
#   tools/check-sensitive.sh --range RANGE  commits in a git range (pre-push)
#   tools/check-sensitive.sh --commit REV   one commit's changes (post-commit)
#   tools/check-sensitive.sh --history      every commit reachable from any ref
#   tools/check-sensitive.sh --message FILE a commit-message file (commit-msg)
#   tools/check-sensitive.sh --identity     the author and committer identity
#
# Personal patterns are one extended regular expression per line in
# <git-dir>/info/sensitive-patterns, created by tools/install-hooks.sh. That file
# is inside .git, so it is never committed or pushed. A workflow can point
# SENSITIVE_PATTERNS_FILE at a file built from a repository secret instead.
#
# A line can be let through with the marker `check-sensitive: allow`, for a
# deliberate example such as a fake key in a test. Use it sparingly.

set -eu

usage() {
    sed -n '2,22p' "$0" | sed 's/^# \{0,1\}//'
}

mode=all
range=
commit=HEAD
message=
while [ $# -gt 0 ]; do
    case "$1" in
        --all) mode=all ;;
        --staged) mode=staged ;;
        --history) mode=history ;;
        --range)
            shift
            range=${1:-}
            [ -n "$range" ] || { echo "check-sensitive: --range needs a range" >&2; exit 2; }
            mode=range
            ;;
        --commit)
            shift
            commit=${1:-}
            mode=commit
            ;;
        --message)
            shift
            message=${1:-}
            [ -n "$message" ] || { echo "check-sensitive: --message needs a file" >&2; exit 2; }
            mode=message
            ;;
        --identity) mode=identity ;;
        -h|--help) usage; exit 0 ;;
        *) echo "check-sensitive: unknown argument: $1" >&2; exit 2 ;;
    esac
    shift
done

case "$message" in
    ""|/*) ;;
    *) message="$PWD/$message" ;;
esac

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
    echo "check-sensitive: not inside a git repository" >&2
    exit 2
}
cd "$root"

gitdir=$(git rev-parse --absolute-git-dir 2>/dev/null || git rev-parse --git-dir)
patterns_file=${SENSITIVE_PATTERNS_FILE:-$gitdir/info/sensitive-patterns}

work=$(mktemp -d "${TMPDIR:-/tmp}/check-sensitive.XXXXXX")
trap 'rm -rf "$work"' EXIT INT TERM

# kind <TAB> extended regular expression. Matched case-insensitively.
: >"$work/patterns"
cat >>"$work/patterns" <<'PATTERNS'
private-key	-----BEGIN [A-Z0-9 ]*PRIVATE KEY-----
putty-private-key	PuTTY-User-Key-File-[0-9]+:
age-secret-key	AGE-SECRET-KEY-1[0-9A-Z]{20,}
aws-access-key	(AKIA|ASIA)[0-9A-Z]{16}
github-token	(gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{50,})
gitlab-token	glpat-[A-Za-z0-9_-]{20,}
slack-token	xox[abprs]-[A-Za-z0-9-]{10,}
stripe-live-key	(sk|rk)_live_[A-Za-z0-9]{16,}
google-api-key	AIza[0-9A-Za-z_-]{35}
openai-key	sk-[A-Za-z0-9_-]{20,}
anthropic-key	sk-ant-[A-Za-z0-9_-]{20,}
npm-token	npm_[A-Za-z0-9]{36}
sendgrid-key	SG\.[A-Za-z0-9_-]{20,}\.[A-Za-z0-9_-]{20,}
generic-secret	(api[_-]?key|apikey|secret|token|passwd|password|client[_-]?secret|access[_-]?key|auth[_-]?token)["' ]*[:=][[:space:]]*["']?[A-Za-z0-9/+_.-]{20,}
local-home-path	/(Users|home)/[A-Za-z0-9._-]{2,}/
windows-home-path	[A-Za-z]:\\Users\\[A-Za-z0-9._-]{2,}
private-ip	(^|[^0-9])(192\.168\.[0-9]{1,3}\.[0-9]{1,3}|10\.[0-9]{1,3}\.[0-9]{1,3}\.[0-9]{1,3}|172\.(1[6-9]|2[0-9]|3[01])\.[0-9]{1,3}\.[0-9]{1,3})([^0-9]|$)
PATTERNS

if [ -f "$patterns_file" ]; then
    # One personal pattern per line; comments and blanks ignored.
    grep -v '^[[:space:]]*#' "$patterns_file" 2>/dev/null |
        grep -v '^[[:space:]]*$' |
        while IFS= read -r pattern; do
            printf 'personal\t%s\n' "$pattern" >>"$work/patterns"
        done || true
fi

TAB=$(printf '\t')

# True when a match is a deliberate placeholder rather than a real value.
skip() { # kind, text
    case "$1" in
        generic-secret)
            printf '%s' "$2" | grep -qiE 'example|dummy|fake|sample|placeholder|changeme|xxxx|your[_-]|redacted|<[a-z_]+>|\$\{?[A-Za-z_]+\}?' && return 0
            ;;
        private-ip)
            # Canonical examples, plus the fixed 10.42.0.0/24 the slot link uses.
            printf '%s' "$2" | grep -qE '(^|[^0-9])(192\.168\.(0|1)\.1|10\.0\.0\.1|172\.16\.0\.1|10\.42\.0\.[0-9]+)([^0-9]|$)' && return 0
            ;;
        local-home-path)
            printf '%s' "$2" | grep -qiE '/(Users|home)/(user|users|you|your|name|someone|example|runner|ubuntu|vscode|pi|deck|root)/' && return 0
            ;;
    esac
    return 1
}

# Scan a file of `path <TAB> line <TAB> text` records, one pattern at a time.
: >"$work/found"
scan_records() { # records file
    while IFS="$TAB" read -r kind pattern; do
        [ -n "${pattern:-}" ] || continue
        grep -iE -e "$pattern" "$1" 2>/dev/null | while IFS= read -r hit; do
            file=${hit%%"$TAB"*}
            rest=${hit#*"$TAB"}
            line=${rest%%"$TAB"*}
            text=${rest#*"$TAB"}
            printf '%s' "$text" | grep -q 'check-sensitive: allow' && continue
            skip "$kind" "$text" && continue
            printf '%s:%s: %s\n' "$file" "$line" "$kind" >>"$work/found"
        done || true
    done <"$work/patterns"
}

# Scan the full contents of tracked files.
scan_tracked() {
    while IFS="$TAB" read -r kind pattern; do
        [ -n "${pattern:-}" ] || continue
        git grep -inIE -e "$pattern" -- . 2>/dev/null | while IFS= read -r hit; do
            file=${hit%%:*}
            rest=${hit#*:}
            line=${rest%%:*}
            text=${rest#*:}
            printf '%s' "$text" | grep -q 'check-sensitive: allow' && continue
            skip "$kind" "$text" && continue
            printf '%s:%s: %s\n' "$file" "$line" "$kind" >>"$work/found"
        done || true
    done <"$work/patterns"
}

# Turn `git log -p` output into added-line records.
log_added_lines() { # arguments to git log
    git log --no-color --format='commit %H' -p -U0 "$@" 2>/dev/null |
        awk '
            /^commit / { file = ""; next }
            /^\+\+\+ / {
                file = substr($0, 7)
                sub(/^b\//, "", file)
                next
            }
            /^@@ / {
                if (file == "/dev/null") { next }
                match($0, /\+[0-9]+/)
                line = substr($0, RSTART + 1, RLENGTH - 1) + 0
                next
            }
            /^\+/ && file != "" && file != "/dev/null" {
                print file "\t" line "\t" substr($0, 2)
                line++
            }
        '
}

# Reject files that are secrets by name alone.
: >"$work/badnames"
name_is_bad() {
    printf '%s' "$1" | grep -qE '(^|/)(id_(rsa|dsa|ecdsa|ed25519)|\.env(\..*)?|\.netrc|\.pgpass|credentials|[^/]*\.(pem|key|p12|pfx|jks|keystore|kdbx|ovpn|ppk))$' || return 1
    case "$1" in
        *.pub|.env.example|*.env.example) return 1 ;;
    esac
    return 0
}

case "$mode" in
    all)
        scan_tracked
        git ls-files -z | tr '\0' '\n' >"$work/names"
        ;;
    staged)
        git diff --cached --no-color -U0 --diff-filter=ACMR |
            awk '
                /^\+\+\+ / {
                    file = substr($0, 7)
                    sub(/^b\//, "", file)
                    next
                }
                /^@@ / {
                    match($0, /\+[0-9]+/)
                    line = substr($0, RSTART + 1, RLENGTH - 1) + 0
                    next
                }
                /^\+/ && file != "" && file != "/dev/null" {
                    print file "\t" line "\t" substr($0, 2); line++
                }
            ' >"$work/records"
        scan_records "$work/records"
        git diff --cached --name-only --diff-filter=ACMR -z | tr '\0' '\n' >"$work/names"
        ;;
    range)
        # Deliberately unquoted: a range may carry git-log options of its own.
        # shellcheck disable=SC2086
        log_added_lines $range >"$work/records"
        scan_records "$work/records"
        git log --format= --name-only --no-color $range 2>/dev/null | sort -u >"$work/names"
        ;;
    history)
        git log --no-color --format='commit %H' -p -U0 --all |
            awk -F '\t' '
                /^commit / { file = ""; next }
                /^\+\+\+ / {
                    file = substr($0, 7); sub(/^b\//, "", file); next
                }
                /^@@ / {
                    match($0, /\+[0-9]+/)
                    line = substr($0, RSTART + 1, RLENGTH - 1) + 0; next
                }
                /^\+/ && file != "" && file != "/dev/null" {
                    print file "\t" line "\t" substr($0, 2); line++
                }
            ' >"$work/records"
        scan_records "$work/records"
        git log --format= --name-only --no-color --all | sort -u >"$work/names"
        ;;
    commit)
        git show --no-color --format= -U0 "$commit" 2>/dev/null |
            awk '
                /^\+\+\+ / {
                    file = substr($0, 7); sub(/^b\//, "", file); next
                }
                /^@@ / {
                    match($0, /\+[0-9]+/)
                    line = substr($0, RSTART + 1, RLENGTH - 1) + 0; next
                }
                /^\+/ && file != "" && file != "/dev/null" {
                    print file "\t" line "\t" substr($0, 2); line++
                }
            ' >"$work/records"
        scan_records "$work/records"
        ;;
    message)
        awk -v f="(commit message)" '{ print f "\t" NR "\t" $0 }' "$message" >"$work/records"
        scan_records "$work/records"
        ;;
    identity)
        {
            git var GIT_AUTHOR_IDENT 2>/dev/null | awk -v f="(author)" '{ print f "\t1\t" $0 }' || true
            git var GIT_COMMITTER_IDENT 2>/dev/null | awk -v f="(committer)" '{ print f "\t1\t" $0 }' || true
        } >"$work/records"
        scan_records "$work/records"
        ;;
esac

if [ -f "$work/names" ]; then
    while IFS= read -r name; do
        [ -n "$name" ] || continue
        name_is_bad "$name" && printf '%s: forbidden file name (secret or private data)\n' "$name" >>"$work/found"
    done <"$work/names"
fi

sort -u "$work/found" -o "$work/found"
count=$(wc -l <"$work/found" | tr -d ' ')

if [ "$count" -eq 0 ]; then
    exit 0
fi

echo "check-sensitive: $count problem(s) found. Nothing was committed, pushed or published." >&2
echo >&2
sed 's/^/  /' "$work/found" >&2
echo >&2
echo "Remove the secret or personal detail. If it was a real secret, rotate it now:" >&2
echo "deleting it here does not remove it from a commit that already exists." >&2
echo "A deliberate example can be marked with \`check-sensitive: allow\` on its line." >&2
echo "Your own names, emails and paths go in \"$patterns_file\" (never committed)." >&2
exit 1
