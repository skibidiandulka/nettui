<div align="center">
  <h2>🖧 nettui</h2>
  <p>TUI for Wi-Fi and Ethernet</p>
  <p><img alt="CI" src="https://github.com/skibidiandulka/nettui/actions/workflows/rust.yml/badge.svg" /></p>
</div>

![nettui-showcase](https://github.com/user-attachments/assets/3603f7eb-433a-4641-bc38-700d93c67e9f)



`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is heavily inspired by `impala` and `ethtui`, but built as one app shell with switchable `Wi-Fi` and `Ethernet` panels.

It is primarily meant for `Omarchy` and tested on `Omarchy`, but it should also work on other Linux distributions that use the same core stack:

- `iwd`
- `systemd-networkd`
- `networkctl`
- Nerd Fonts

## ✨ Features

- `Wi-Fi` and `Ethernet`
- Hidden SSID connect
- Enterprise Wi-Fi setup and edit
- Explicit multi-adapter Wi-Fi view
- QR Wi-Fi sharing
- Wi-Fi power toggle
- Access point mode
- DHCP renew
- Configurable keybinds

## 💡 Prerequisites

- Linux
- `iwd` running on D-Bus
- `systemd-networkd`
- `networkctl`
- Nerd Fonts recommended

> [!IMPORTANT]
> `nettui` works best when `iwd` is the only active Wi-Fi manager. Avoid overlapping managers like `NetworkManager` or `wpa_supplicant`.

## 🚀 Installation

### crates.io

```bash
cargo install nettui
```

### Arch Linux (AUR source)

```bash
yay -S nettui
```

### Arch Linux (AUR binary)

```bash
yay -S nettui-bin
```

### Other Linux distributions

`nettui` is not Arch-specific — it runs on any distribution using the `iwd` + `systemd-networkd` stack. Install the binary with `cargo install nettui` (see above), then make sure that stack is the active network manager:

```bash
# Use iwd for Wi-Fi and systemd-networkd for addressing.
# Disable overlapping managers first (skip any that aren't installed).
sudo systemctl disable --now NetworkManager wpa_supplicant 2>/dev/null || true
sudo systemctl enable --now iwd systemd-networkd systemd-resolved
```

Config is created on first launch at `~/.config/nettui/keybinds.toml`. Nerd Fonts are recommended for the full UI.

## 🤝 Contributing

See `CONTRIBUTING.md` for development setup, required checks, and pull request expectations.

## 🪄 Usage

```bash
nettui
```

On first launch, `nettui` creates:

```bash
~/.config/nettui/keybinds.toml
```

## 🧩 Omarchy

Official Omarchy currently launches `impala` for Wi-Fi:

```bash
rfkill unblock wifi
omarchy-launch-or-focus-tui impala
```

To switch your local Omarchy install to `nettui`:

```bash
sed -i 's/omarchy-launch-or-focus-tui impala/omarchy-launch-or-focus-tui nettui/g' ~/.local/share/omarchy/bin/omarchy-launch-wifi
```

Verify:

```bash
sed -n '1,120p' ~/.local/share/omarchy/bin/omarchy-launch-wifi
```

By default, Omarchy only floats the window classes shipped in its own list, so `nettui` opens **tiled**. To make it float and center like `impala` and the other Omarchy TUIs, add this rule to the personal section at the bottom of `~/.config/hypr/hyprland.conf`:

```bash
windowrule = tag +floating-window, match:class org.omarchy.nettui
```

Then reload:

```bash
hyprctl reload
```

`nettui` no longer enforces a hard minimum terminal size, so no fixed-size rule is needed — the `floating-window` tag already applies Omarchy's standard float, center, and size.

## ⌨️ Controls

### Global

- `h/l` or `←/→`: switch tab
- `j/k` or `↓/↑`: move
- `r`: refresh
- `q` or `Esc`: quit

### Wi-Fi

- `Tab` / `Shift+Tab`: switch section
- `s`: scan
- `Enter`: connect or disconnect
- `a`: show all
- `d`: forget
- `y`: share
- `t`: autoconnect
- `n`: hidden network
- `i`: details
- `e`: edit 802.1x
- `o`: power
- `p`: access point

### Ethernet

- `Enter`: link up/down
- `n`: renew DHCP

## 🩺 Notes

- `Access point` mode is hardware-dependent.
- Some adapters can scan and connect normally, but still fail in AP mode.
- `nettui` warns when the Wi-Fi radio is blocked by `rfkill`.
- For DHCP in AP mode, `/etc/iwd/main.conf` should enable:

```ini
[General]
EnableNetworkConfiguration=true
```

## 🔄 Restart

```bash
pkill -x nettui || true
omarchy-launch-or-focus-tui nettui
```

## 🛠️ Build

```bash
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## 📦 Release Asset

Always build the GitHub release tarball with:

```bash
./scripts/build-release-asset.sh
```

## ⚖️ License

`nettui` is licensed under `GPL-3.0-only`. See `LICENSE`.
