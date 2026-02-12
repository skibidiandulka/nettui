// Copyright (C) 2026 skibidiandulka
// Clean-room implementation inspired by Impala UX by pythops.

use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Keybinds {
    pub quit: char,
    pub refresh: char,
    pub up: char,
    pub down: char,
    pub prev_tab: char,
    pub next_tab: char,
    pub wifi_scan: char,
    pub wifi_show_all: char,
    pub wifi_forget: char,
    pub wifi_autoconnect: char,
    pub wifi_hidden: char,
    pub wifi_details: char,
    pub ethernet_renew: char,
}

impl Default for Keybinds {
    fn default() -> Self {
        Self {
            quit: 'q',
            refresh: 'r',
            up: 'k',
            down: 'j',
            prev_tab: 'h',
            next_tab: 'l',
            wifi_scan: 's',
            wifi_show_all: 'a',
            wifi_forget: 'd',
            wifi_autoconnect: 't',
            wifi_hidden: 'n',
            wifi_details: 'i',
            ethernet_renew: 'n',
        }
    }
}

impl Keybinds {
    pub fn load() -> Self {
        let mut out = Self::default();
        let Some(path) = keybinds_path() else {
            return out;
        };

        ensure_default_config_exists(&path);

        let Ok(raw) = fs::read_to_string(path) else {
            return out;
        };
        let Ok(file) = toml::from_str::<KeybindsFile>(&raw) else {
            return out;
        };
        let Some(keys) = file.keys else {
            return out;
        };

        apply_override(&mut out.quit, keys.quit);
        apply_override(&mut out.refresh, keys.refresh);
        apply_override(&mut out.up, keys.up);
        apply_override(&mut out.down, keys.down);
        apply_override(&mut out.prev_tab, keys.prev_tab);
        apply_override(&mut out.next_tab, keys.next_tab);
        apply_override(&mut out.wifi_scan, keys.wifi_scan);
        apply_override(&mut out.wifi_show_all, keys.wifi_show_all);
        apply_override(&mut out.wifi_forget, keys.wifi_forget);
        apply_override(&mut out.wifi_autoconnect, keys.wifi_autoconnect);
        apply_override(&mut out.wifi_hidden, keys.wifi_hidden);
        apply_override(&mut out.wifi_details, keys.wifi_details);
        apply_override(&mut out.ethernet_renew, keys.ethernet_renew);

        out
    }
}

#[derive(Debug, Deserialize)]
struct KeybindsFile {
    keys: Option<KeybindsPartial>,
}

#[derive(Debug, Default, Deserialize)]
struct KeybindsPartial {
    quit: Option<String>,
    refresh: Option<String>,
    up: Option<String>,
    down: Option<String>,
    prev_tab: Option<String>,
    next_tab: Option<String>,
    wifi_scan: Option<String>,
    wifi_show_all: Option<String>,
    wifi_forget: Option<String>,
    wifi_autoconnect: Option<String>,
    wifi_hidden: Option<String>,
    wifi_details: Option<String>,
    ethernet_renew: Option<String>,
}

fn keybinds_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/nettui/keybinds.toml"))
}

fn ensure_default_config_exists(path: &PathBuf) {
    if path.exists() {
        return;
    }
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }

    let _ = fs::write(path, default_config_template());
}

fn default_config_template() -> &'static str {
    include_str!("../config/keybinds.toml.example")
}

fn apply_override(target: &mut char, value: Option<String>) {
    let Some(raw) = value else {
        return;
    };
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    let mut chars = trimmed.chars();
    let Some(c) = chars.next() else {
        return;
    };
    if chars.next().is_some() {
        return;
    }
    *target = c.to_ascii_lowercase();
}
