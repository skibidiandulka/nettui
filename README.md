# nettui

`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is built as a clean-room project inspired by the UX direction of tools like `impala` and `ethtui`,
but with one app shell and switchable Wi-Fi/Ethernet panels.

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
- Toast/error popups and terminal size guard

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

Ethernet tab:

- `Enter`: toggle selected interface link (`up/down`)
- `n`: renew DHCP on selected interface

## Keybind configuration

`nettui` reads optional key overrides from:

`~/.config/nettui/keybinds.toml`

Use the example template:

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

## Omarchy integration (optional)

Recommended launch command for Omarchy-style app-id handling:

```bash
omarchy-launch-or-focus-tui nettui
```

If Omarchy chooses to make `nettui` the default network TUI later, this command can become the single
launcher entry for both Wi-Fi and Ethernet workflows.
