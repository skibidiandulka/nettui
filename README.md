<p align="center"> TUI for managing Wi-Fi and Ethernet</p>

# nettui

`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is built as a clean-room project inspired by the UX direction of tools like `impala` and `ethtui`,
but with one app shell and switchable Wi-Fi/Ethernet panels. It was built mainly for OMARCHY arch linux.
This project was inspired by and builds upon ideas from Impala by pythops.

## Runtime assumptions

- Linux
- Wi-Fi backend: `iwd` service available on D-Bus
- Ethernet backend: `systemd-networkd` (`networkctl` available)

## Installation

From AUR:

```bash
yay -S nettui
```

or prebuilt binary package:

```bash
yay -S nettui-bin
```

From crates.io:

```bash
cargo install nettui
```

## Controls

Global:

- `h/l` or `←/→`: switch transport tab (`Wi-Fi` / `Ethernet`)
- `j/k` or `↓/↑`: move selection
- `r`: refresh
- `q` (or `Esc`): quit

Wi-Fi tab:

- `Tab` / `Shift+Tab`: switch focus (`Known` / `New` / `Device`)
- `s`: scan
- `Enter`: connect/disconnect selected network
- `Enter` in passphrase popup: connect with passphrase
- `a`: show/hide additional entries (`Known`: unavailable, `New`: hidden)
- `d`: forget selected known network
- `t`: toggle autoconnect for selected known network
- `n`: connect hidden network (from `New` section)
- `i`: toggle Wi-Fi details popup
- `New Networks` shows `- no new networks -` when no entries are discovered
- `refresh` and `show all` actions display info toast feedback

Ethernet tab:

- `Enter`: toggle selected interface link (`up/down`)
- `n`: renew DHCP on selected interface

## Keybind configuration

`nettui` reads optional key overrides from:

`~/.config/nettui/keybinds.toml`

On first launch, `nettui` automatically creates this file with defaults if it does not exist.

You can still reset it manually from template:

```bash
mkdir -p ~/.config/nettui
cp /usr/share/doc/nettui/keybinds.toml.example ~/.config/nettui/keybinds.toml
```

Edit your keybinds directly in:

```bash
~/.config/nettui/keybinds.toml
```

After changing keybinds, restart `nettui`.

## Restart / control

`nettui` is not a systemd service, so `systemctl` is not used.

Quick restart:

```bash
pkill -x nettui || true
omarchy-launch-or-focus-tui nettui
```

From inside the app:

- press `q` to quit
- launch again from Waybar network icon (or run `omarchy-launch-wifi`)

## Build

```bash
cargo build
cargo test
```

## License

`nettui` is licensed under `GPL-3.0-only`. See `LICENSE`.

## Omarchy integration (optional)

Current Omarchy launcher behavior:

```bash
omarchy-launch-wifi
```

To force `nettui` as default network TUI:

1. Install `nettui`:

```bash
yay -S nettui-bin
```

2. Verify launcher script prefers `nettui`:

```bash
grep -n "nettui" ~/.local/share/omarchy/bin/omarchy-launch-wifi
grep -n "nettui" ~/.local/share/omarchy/bin/omarchy-launch-ethernet
```

3. If needed, patch both launchers:

```bash
sed -i 's/omarchy-launch-or-focus-tui impala/omarchy-launch-or-focus-tui nettui/g' ~/.local/share/omarchy/bin/omarchy-launch-wifi
sed -i 's/omarchy-launch-or-focus-tui ethtui/omarchy-launch-or-focus-tui nettui/g' ~/.local/share/omarchy/bin/omarchy-launch-ethernet
```

4. Set dedicated floating window size for `nettui` (Hyprland):

```bash
grep -q "match:class org.omarchy.nettui" ~/.config/hypr/apps/system.conf || echo "windowrule = size 1190 735, match:class org.omarchy.nettui" >> ~/.config/hypr/apps/system.conf
hyprctl reload
```

5. Click network module in Waybar to verify `nettui` opens.
