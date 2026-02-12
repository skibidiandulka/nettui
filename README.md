# nettui

`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is built as a clean-room project inspired by the UX direction of tools like `impala` and `ethtui`,
but with one app shell and switchable Wi-Fi/Ethernet panels.

## Scope (v0.1)

- One TUI with two tabs: `Wi-Fi` and `Ethernet`
- Startup tab policy: prefer active transport (`Ethernet` if active, else `Wi-Fi` if active)
- Wi-Fi (iwd/iwctl):
  - list networks
  - scan
  - connect/disconnect selected network
- Ethernet (systemd-networkd):
  - list interfaces and details
  - DHCP renew (`networkctl renew`)
- Toast/error popups and terminal size guard

## Runtime assumptions

- Linux
- Wi-Fi backend: `iwd` (`iwctl` available)
- Ethernet backend: `systemd-networkd` (`networkctl` available)

## Controls

Global:

- `Tab` / `Shift+Tab` or `h/l`: switch tab
- `j/k` or `↓/↑`: move selection
- `r`: refresh
- `q` (or `Esc`): quit

Wi-Fi tab:

- `s`: scan
- `Enter`: connect/disconnect selected network

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
