#!/usr/bin/env sh
set -eu

project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
test_root=$(mktemp -d "${TMPDIR:-/tmp}/goto-shell-test.XXXXXX")
cleanup() {
    case "$test_root" in
        "${TMPDIR:-/tmp}"/goto-shell-test.*) rm -rf -- "$test_root" ;;
        *) echo "Refusing to remove unexpected test path: $test_root" >&2 ;;
    esac
}
trap cleanup EXIT HUP INT TERM

export PATH="$project_dir/target/debug:$PATH"
export GOTO_CONFIG="$test_root/workplaces.json"
target="$test_root/folder with spaces"
mkdir -p "$target"
goto -s "$target" --name ci

bash -c 'eval "$(goto init bash)"; goto ci; test "$PWD" = "'"$target"'"'

if [ "${GOTO_TEST_BASH_ONLY:-0}" = "1" ]; then
    exit 0
fi

command -v zsh >/dev/null
command -v fish >/dev/null
zsh -c 'eval "$(goto init zsh)"; goto ci; test "$PWD" = "'"$target"'"'
fish -c 'goto init fish | source; goto ci; test "$PWD" = "'"$target"'"'
