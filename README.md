<p align="center"> TUI for managing Wi-Fi and Ethernet</p>

# nettui

`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is built as a clean-room project inspired by the UX direction of tools like `impala` and `ethtui`,
but with one app shell and switchable Wi-Fi/Ethernet panels. It was built mainly for OMARCHY arch linux.
This project was inspired by and builds upon ideas from Impala by pythops.

## Scope (v0.1)

- One TUI with two transport tabs: `Wi-Fi` and `Ethernet`
- Startup tab policy: prefer active transport (`Ethernet` if active, else `Wi-Fi` if active)
- Wi-Fi (iwd via D-Bus / `iwdrs`):
  - split sections: `Known Networks`, `New Networks`, `Device`
  - non-blocking scan/connect with spinner in section titles
  - connect/disconnect selected network
  - fallback passphrase prompt when iwd reports `No Agent registered`
  - forget known network
  - toggle autoconnect
  - show/hide unavailable known and hidden network entries
  - connect hidden network by SSID prompt
  - detail popup for active Wi-Fi interface (`i`)
- Ethernet (systemd-networkd):
  - list interfaces and details
  - link up/down toggle on selected interface
  - toggle link admin state up/down (`ip link set`)
  - DHCP renew (`networkctl renew`)
- Toast/error popups and terminal size guard (`119x35` minimum)

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

or from repository:

```bash
cp config/keybinds.toml.example ~/.config/nettui/keybinds.toml
```

After changing keybinds, restart `nettui`.

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

On recent Omarchy, this already prefers `nettui` when installed.

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
