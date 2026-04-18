# Contributing

## Scope

`nettui` is an `iwd`-first network TUI built primarily for `Omarchy`.
Keep changes aligned with that scope:

- prefer `iwd` over adding another Wi-Fi backend
- keep the Wi-Fi flow close to `impala`
- keep Ethernet support integrated, not bolted on
- prefer small, reviewable changes

## Development Setup

Requirements:

- Rust stable
- `iwd`
- `systemd-networkd`
- `networkctl`
- Nerd Fonts recommended for the full UI

Clone and run:

```bash
git clone https://github.com/skibidiandulka/nettui.git
cd nettui
cargo run --release
```

## Required Checks

Run these before opening a pull request:

```bash
cargo fmt --all
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## Code Style

- use `rustfmt`; do not hand-format around it
- prefer explicit, descriptive names
- keep modules small when a file starts mixing unrelated concerns
- keep user-facing text short and plain
- add comments only when they explain intent, not mechanics
- do not introduce a second Wi-Fi manager backend

## Repo Notes

- `README.md`: user-facing overview and setup
- `config/keybinds.toml.example`: default keybind reference
- `scripts/build-release-asset.sh`: release archive layout check
- `.github/workflows/rust.yml`: CI checks

## Pull Requests

A good pull request should include:

- a short summary of the change
- why the change is needed
- how it was tested
- screenshots or terminal output for visible UI changes when relevant

## Releases

Release work should keep these paths in sync:

- `Cargo.toml`
- `Cargo.lock`
- Git tag and GitHub release
- `crates.io`
- AUR `nettui`
- AUR `nettui-bin`

Do not publish a release unless the GitHub release asset, `crates.io`, and AUR packages all match the same version.
