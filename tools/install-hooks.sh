#!/bin/sh
# Install this repository's git hooks and seed the private denylist of your own
# names, email addresses and paths.
#
#   sh tools/install-hooks.sh
#
# The hooks live in .githooks (committed, so they can be reviewed) and are
# enabled with a repository-local core.hooksPath. The denylist is written to
# <git-dir>/info/sensitive-patterns, which is inside .git and is never committed
# or pushed.
#
# Run this once per clone. It is safe to run again: it only adds missing lines.
set -eu

root=$(git rev-parse --show-toplevel 2>/dev/null) || {
    echo "install-hooks: run this inside the repository" >&2
    exit 1
}
cd "$root"

if [ ! -d .githooks ]; then
    echo "install-hooks: .githooks is missing; are you on the right branch?" >&2
    exit 1
fi

chmod +x .githooks/* tools/check-sensitive.sh

previous=$(git config --local --get core.hooksPath 2>/dev/null || true)
git config --local core.hooksPath .githooks

gitdir=$(git rev-parse --absolute-git-dir 2>/dev/null || git rev-parse --git-dir)
patterns="$gitdir/info/sensitive-patterns"
mkdir -p "$gitdir/info"
if [ ! -f "$patterns" ]; then
    cat >"$patterns" <<'HEADER'
# Personal data that must never be committed or pushed, as extended regular
# expressions, one per line. This file lives inside .git: it is never tracked.
# tools/install-hooks.sh fills in your email, user name, home directory and
# machine names; add anything else that identifies you (phone number, an old
# email address, a device serial).
HEADER
fi

# Escape extended-regex metacharacters so a literal string matches literally.
escape() {
    printf '%s' "$1" | sed 's#[][\\.^$*+?(){}|]#\\&#g'
}

added=0
add() {
    [ -n "${1:-}" ] || return 0
    [ "${#1}" -ge 3 ] || return 0
    if ! grep -qxF -- "$1" "$patterns" 2>/dev/null; then
        printf '%s\n' "$1" >>"$patterns"
        added=$((added + 1))
    fi
}

email=$(git config user.email 2>/dev/null || true)
case "$email" in
    ""|*@users.noreply.github.com) ;;
    *) add "$(escape "$email")" ;;
esac

add "$(escape "$(id -un 2>/dev/null || whoami 2>/dev/null || true)")"

home=${HOME:-}
[ -n "$home" ] && [ "$home" != "/" ] && add "$(escape "$home")"
home_name=${home##*/}
[ -n "$home_name" ] && [ "$home_name" != "$home" ] && add "$(escape "$home_name")"

short=$(hostname -s 2>/dev/null || true)
[ -n "$short" ] && add "$(escape "$short")"
full=$(hostname 2>/dev/null || true)
[ -n "$full" ] && [ "$full" != "$short" ] && add "$(escape "$full")"
if command -v scutil >/dev/null 2>&1; then
    add "$(escape "$(scutil --get ComputerName 2>/dev/null || true)")"
fi

echo "install-hooks: hooks enabled (core.hooksPath=.githooks)"
if [ -n "$previous" ] && [ "$previous" != ".githooks" ]; then
    echo "install-hooks: was '$previous'; post-checkout still runs ~/.git-hooks/post-checkout if present"
fi
echo "install-hooks: $added personal pattern(s) added to $patterns"

case "$email" in
    ""|*@users.noreply.github.com) ;;
    *)
        echo >&2
        echo "warning: your git email ($email) is personal and is now refused." >&2
        echo "Set a no-reply address before committing, for example:" >&2
        echo "  git config --global user.email '<ID>+<USER>@users.noreply.github.com'" >&2
        echo "(GitHub shows the exact one under Settings -> Emails.)" >&2
        ;;
esac

echo "install-hooks: dry run:"
if sh tools/check-sensitive.sh --all; then
    echo "install-hooks: the current tree is clean"
else
    echo "install-hooks: the tree has findings above; fix them before committing" >&2
fi
