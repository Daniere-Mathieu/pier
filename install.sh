#!/bin/sh
set -eu

REPO="Daniere-Mathieu/pier"
BINARY="pier"
INSTALL_DIR="${PIER_INSTALL_DIR:-${HOME}/.local/bin}"

main() {
    os="$(uname -s)"
    arch="$(uname -m)"

    case "${os}" in
        Linux)  target_os="unknown-linux-gnu" ;;
        Darwin) target_os="apple-darwin" ;;
        *)
            echo "Error: unsupported OS '${os}'" >&2
            exit 1
            ;;
    esac

    case "${arch}" in
        x86_64|amd64)   target_arch="x86_64" ;;
        aarch64|arm64)   target_arch="aarch64" ;;
        *)
            echo "Error: unsupported architecture '${arch}'" >&2
            exit 1
            ;;
    esac

    target="${target_arch}-${target_os}"

    if [ -n "${1:-}" ]; then
        tag="$1"
    else
        tag="$(get_latest_tag)"
    fi

    echo "Installing ${BINARY} ${tag} for ${target}..."

    archive="pier-${tag}-${target}.tar.gz"
    url="https://github.com/${REPO}/releases/download/${tag}/${archive}"

    tmpdir="$(mktemp -d)"
    trap 'rm -rf "${tmpdir}"' EXIT

    echo "Downloading ${url}..."
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "${url}" -o "${tmpdir}/${archive}"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "${url}" -O "${tmpdir}/${archive}"
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi

    tar xzf "${tmpdir}/${archive}" -C "${tmpdir}"

    mkdir -p "${INSTALL_DIR}"
    cp "${tmpdir}/pier-${tag}-${target}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
    chmod +x "${INSTALL_DIR}/${BINARY}"

    echo ""
    echo "${BINARY} ${tag} installed to ${INSTALL_DIR}/${BINARY}"

    if ! echo "${PATH}" | tr ':' '\n' | grep -qx "${INSTALL_DIR}"; then
        echo ""
        echo "Add ${INSTALL_DIR} to your PATH:"
        echo "  export PATH=\"${INSTALL_DIR}:\${PATH}\""
    fi
}

get_latest_tag() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" |
            grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" |
            grep '"tag_name"' | head -1 | sed 's/.*"tag_name": *"\([^"]*\)".*/\1/'
    else
        echo "Error: curl or wget is required" >&2
        exit 1
    fi
}

main "$@"
