#!/usr/bin/env bash
set -euo pipefail

BRANCH="${BRANCH:-main}"
BASE_URL="https://raw.githubusercontent.com/davidmegginson/ourairports-data/${BRANCH}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
RESOURCE_DIR="${RESOURCE_DIR:-$(cd "${SCRIPT_DIR}/.." && pwd)/resource}"

if [[ ! -d "${RESOURCE_DIR}" ]]; then
    echo "error: resource directory not found at ${RESOURCE_DIR}" >&2
    exit 1
fi

trap 'rm -f "${RESOURCE_DIR}"/*.tmp 2>/dev/null || true' EXIT

FILES=(
    "airports.csv:\"ident\""
    "runways.csv:\"airport_ident\""
)

echo "Fetching OurAirports data from ${BASE_URL}"

for entry in "${FILES[@]}"; do
    file="${entry%%:*}"
    needle="${entry#*:}"
    dest="${RESOURCE_DIR}/${file}"
    tmp="${dest}.tmp"

    echo "  ${file}"
    curl -fsSL -o "${tmp}" "${BASE_URL}/${file}"

    header="$(head -n 1 "${tmp}")"
    if [[ "${header}" != *"${needle}"* ]]; then
        echo "error: ${file} header mismatch (expected ${needle}):" >&2
        echo "  ${header}" >&2
        exit 1
    fi

    mv "${tmp}" "${dest}"
    mb="$(awk -v b="$(wc -c < "${dest}")" 'BEGIN { printf "%.1f", b / 1048576 }')"
    echo "    ${mb} MB"
done

echo "Done. Run 'cargo build' and 'adb update' to index the new data."
