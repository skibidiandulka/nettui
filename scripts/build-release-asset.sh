#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

version="${1:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)}"
if [[ -z "$version" ]]; then
  echo "Could not detect version from Cargo.toml" >&2
  exit 1
fi

arch="${ARCH:-x86_64}"
dist_dir="${DIST_DIR:-$repo_root/dist}"
bundle_dir="nettui-v${version}-${arch}"
stage_dir="$dist_dir/$bundle_dir"
tarball="$dist_dir/${bundle_dir}.tar.gz"

mkdir -p "$dist_dir"
rm -rf "$stage_dir" "$tarball"

cargo build --release --locked

install -Dm755 "target/release/nettui" "$stage_dir/nettui"
install -Dm644 "README.md" "$stage_dir/README.md"
install -Dm644 "LICENSE" "$stage_dir/LICENSE"
install -Dm644 "config/keybinds.toml.example" "$stage_dir/config/keybinds.toml.example"

tar -C "$dist_dir" -czf "$tarball" "$bundle_dir"

expected_entries=(
  "${bundle_dir}/"
  "${bundle_dir}/nettui"
  "${bundle_dir}/README.md"
  "${bundle_dir}/LICENSE"
  "${bundle_dir}/config/"
  "${bundle_dir}/config/keybinds.toml.example"
)

for entry in "${expected_entries[@]}"; do
  if ! grep -Fxq "$entry" <<< "$(tar -tzf "$tarball")"; then
    echo "Release asset validation failed. Missing: $entry" >&2
    exit 1
  fi
done

echo "Created: $tarball"
sha256sum "$tarball"
