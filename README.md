# nettui

`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is built as a clean-room project inspired by the UX direction of tools like `impala` and `ethtui`,
but with one app shell and switchable Wi-Fi/Ethernet panels.

## Scope (v0.1)

- One TUI with two transport tabs: `Wi-Fi` and `Ethernet`
- Startup tab policy: prefer active transport (`Ethernet` if active, else `Wi-Fi` if active)
- Wi-Fi (iwd/iwctl):
  - split sections: `Known Networks`, `New Networks`, `Adapter`
  - scan
  - connect/disconnect selected network
  - detail popup for active Wi-Fi interface (`i`)
- Ethernet (systemd-networkd):
  - list interfaces and details
  - DHCP renew (`networkctl renew`)
- Toast/error popups and terminal size guard

## Runtime assumptions

- Linux
- Wi-Fi backend: `iwd` (`iwctl` available)
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

- `Tab` / `Shift+Tab`: switch focus (`Known` / `New` / `Adapter`)
- `s`: scan
- `Enter`: connect/disconnect selected network
- `i`: toggle Wi-Fi details popup

Ethernet tab:

- `n`: renew DHCP on selected interface

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
