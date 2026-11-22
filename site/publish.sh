#!/usr/bin/env bash
# Publish the built site tree to gh-pages without touching the current index.
set -euo pipefail

site_dir=$(realpath "$1")
git_dir=$(git rev-parse --absolute-git-dir)
branch=refs/heads/gh-pages
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT

# Build the tree from an empty index so removed site files disappear on the next publish.
export GIT_INDEX_FILE="$scratch/index"
site_git=(git -C "$site_dir" --git-dir="$git_dir" --work-tree="$site_dir")
test -f "$site_dir/index.html"
test -f "$site_dir/zh/index.html"
"${site_git[@]}" add --all --force
tree=$("${site_git[@]}" write-tree)
unset GIT_INDEX_FILE

parent=()
remote_head=$(git ls-remote origin "$branch")
if [[ -n $remote_head ]]; then
    git fetch --no-tags origin "$branch"
    previous=$(git rev-parse FETCH_HEAD)
    if [[ $tree == "$(git rev-parse "$previous^{tree}")" ]]; then
        printf '%s\n' "$previous"
        exit 0
    fi
    parent=(-p "$previous")
fi

commit=$(git commit-tree "$tree" "${parent[@]}" -m "Publish Robot Bus website")
git push origin "$commit:$branch"
printf '%s\n' "$commit"
