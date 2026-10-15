#!/usr/bin/env bash
#
# Supply-chain gates (Section 20.10, [BA-SUPPLY-001]).
#
# CI runs this exact script, so a local run and the merge gate cannot disagree.
# Every gate is blocking: the script exits non-zero on the first failure and the
# CI step that calls it fails with it. Nothing here reports a warning and then
# passes.
#
#   cargo deny check        advisories, bans, licenses and sources policy
#   cargo audit             RustSec advisories for the locked Rust graph
#   cargo cyclonedx         Rust dependency inventory (CycloneDX 1.5 JSON)
#   npm audit               advisories for the locked JavaScript graph
#   npm sbom                JavaScript dependency inventory (CycloneDX 1.5 JSON)
#
# Required tools: cargo, cargo-deny, cargo-audit, cargo-cyclonedx, npm.

set -euo pipefail

cd "$(dirname "$0")/.."

required_tools=(cargo npm cargo-deny cargo-audit cargo-cyclonedx)

missing=()
for tool in "${required_tools[@]}"; do
  if ! command -v "$tool" >/dev/null 2>&1; then
    missing+=("$tool")
  fi
done
if [ ${#missing[@]} -gt 0 ]; then
  echo "supply-chain: missing tool(s): ${missing[*]}" >&2
  echo "Install the cargo tools, for example:" >&2
  echo "  cargo install cargo-deny cargo-audit cargo-cyclonedx" >&2
  exit 127
fi

echo "== cargo deny check"
cargo deny check

echo "== cargo audit"
# Warnings fail too: an unmaintained or yanked dependency is a real finding.
cargo audit --deny warnings

echo "== npm audit"
# Any advisory at any severity fails; the JavaScript graph has its own job here
# rather than being folded into the Rust report.
npm audit

echo "== dependency inventory (CycloneDX 1.5 JSON)"
sbom_dir="target/sbom"
rm -rf "$sbom_dir"
mkdir -p "$sbom_dir"

# A fixed source-date epoch keeps the inventory byte-stable for a given commit.
if epoch=$(git log -1 --format=%ct 2>/dev/null) && [ -n "$epoch" ]; then
  :
else
  epoch=1700000000
fi

SOURCE_DATE_EPOCH="$epoch" cargo cyclonedx --format json --spec-version 1.5

# `cargo cyclonedx` writes one document per workspace member beside its manifest;
# they are build artifacts, so they are collected into the ignored target tree.
while IFS= read -r -d '' file; do
  mv "$file" "$sbom_dir/$(basename "$file")"
done < <(find crates -name '*.cdx.json' -print0)

npm sbom --sbom-format cyclonedx > "$sbom_dir/npm.cdx.json"

produced=$(find "$sbom_dir" -name '*.cdx.json' | wc -l | tr -d ' ')
if [ "$produced" -lt 2 ]; then
  echo "supply-chain: expected a Rust and a JavaScript SBOM, found $produced" >&2
  exit 1
fi

python3 - "$sbom_dir" <<'PY'
import json
import pathlib
import sys

directory = pathlib.Path(sys.argv[1])
components = {}
for document in sorted(directory.glob('*.cdx.json')):
    payload = json.loads(document.read_text())
    if payload.get('bomFormat') != 'CycloneDX':
        raise SystemExit(f'{document.name}: not a CycloneDX document')
    if payload.get('specVersion') != '1.5':
        raise SystemExit(f'{document.name}: unexpected spec version {payload.get("specVersion")}')
    entries = payload.get('components') or []
    if not entries:
        raise SystemExit(f'{document.name}: no components, so the inventory is empty')
    source = 'npm' if document.name == 'npm.cdx.json' else 'rust'
    for entry in entries:
        name = entry.get('name', '')
        version = entry.get('version', '')
        ids = []
        for license_entry in entry.get('licenses') or []:
            expression = license_entry.get('expression')
            if expression:
                ids.append(expression)
                continue
            license_object = license_entry.get('license', {})
            ids.append(license_object.get('id') or license_object.get('name') or 'unknown')
        key = f'{name} {version}'
        label = ', '.join(sorted(set(ids))) or 'no license declared'
        # A package used by both graphs keeps the first concrete license found.
        components.setdefault(key, (source, label))

lines = [f'{key}: {source}, {label}' for key, (source, label) in sorted(components.items())]
(directory / 'licenses.txt').write_text('\n'.join(lines) + '\n')
declared = sum(1 for _, (_, label) in components.items() if label != 'no license declared')
print(
    f'supply-chain: {len(components)} components inventoried '
    f'({declared} with a declared license of {len(components)})'
)
PY
