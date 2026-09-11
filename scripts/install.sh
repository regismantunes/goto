#!/usr/bin/env sh
set -eu

repository=${GOTO_REPOSITORY:-@GOTO_REPOSITORY@}
version=${GOTO_VERSION:-latest}
install_dir=${GOTO_INSTALL_DIR:-"$HOME/.local/bin"}

placeholder_prefix='@GOTO_'
if [ "${repository#"$placeholder_prefix"}" != "$repository" ]; then
    echo "error: set GOTO_REPOSITORY to owner/repository when running the source installer" >&2
    exit 1
fi

case "$(uname -s)-$(uname -m)" in
    Linux-x86_64|Linux-amd64)
        target=x86_64-unknown-linux-musl
        ;;
    Darwin-x86_64|Darwin-amd64)
        target=x86_64-apple-darwin
        ;;
    Darwin-arm64|Darwin-aarch64)
        target=aarch64-apple-darwin
        ;;
    *)
        echo "error: this operating system or architecture is not supported by a prebuilt release" >&2
        exit 1
        ;;
esac

asset="goto-$target.tar.gz"
if [ "$version" = "latest" ]; then
    release_url="https://github.com/$repository/releases/latest/download"
else
    case "$version" in
        v*) tag=$version ;;
        *) tag="v$version" ;;
    esac
    release_url="https://github.com/$repository/releases/download/$tag"
fi

temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/goto-install.XXXXXX")
cleanup() {
    case "$temporary_dir" in
        "${TMPDIR:-/tmp}"/goto-install.*) rm -rf -- "$temporary_dir" ;;
        *) echo "Refusing to remove unexpected installer path: $temporary_dir" >&2 ;;
    esac
}
trap cleanup EXIT HUP INT TERM

echo "Downloading $asset..."
curl -fsSL "$release_url/$asset" -o "$temporary_dir/$asset"
curl -fsSL "$release_url/checksums.txt" -o "$temporary_dir/checksums.txt"

expected=$(awk -v asset="$asset" '$2 == asset { print $1 }' "$temporary_dir/checksums.txt")
if [ -z "$expected" ]; then
    echo "error: $asset is missing from checksums.txt" >&2
    exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
    actual=$(sha256sum "$temporary_dir/$asset" | awk '{ print $1 }')
else
    actual=$(shasum -a 256 "$temporary_dir/$asset" | awk '{ print $1 }')
fi
if [ "$actual" != "$expected" ]; then
    echo "error: checksum verification failed for $asset" >&2
    exit 1
fi

tar -xzf "$temporary_dir/$asset" -C "$temporary_dir"
mkdir -p "$install_dir"
install -m 0755 "$temporary_dir/goto" "$install_dir/goto"

echo "Installed goto in $install_dir."
case ":$PATH:" in
    *":$install_dir:"*) ;;
    *) echo "Add $install_dir to PATH before activating goto." ;;
esac
echo "Activate it with one of:"
echo '  Bash: eval "$(goto init bash)"'
echo '  Zsh:  eval "$(goto init zsh)"'
echo '  Fish: goto init fish | source'
