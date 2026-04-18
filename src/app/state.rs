use super::*;
use crate::domain::wifi::WifiDeviceInfo;

impl App {
    pub(super) fn known_total_len(&self) -> usize {
        if self.wifi_access_point_active() {
            return 1;
        }
        self.wifi.known_networks.len()
            + if self.show_unavailable_known_networks {
                self.wifi.unavailable_known_networks.len()
            } else {
                0
            }
    }

    pub(super) fn new_total_len(&self) -> usize {
        if self.wifi_access_point_active() {
            return self.wifi.access_point_clients.len().max(1);
        }
        self.wifi.new_networks.len()
            + if self.show_hidden_networks {
                self.wifi.hidden_networks.len()
            } else {
                0
            }
    }

    pub(super) fn device_total_len(&self) -> usize {
        self.wifi.devices.len()
    }

    pub(super) fn selected_known_network(&self) -> Option<&WifiNetwork> {
        if self.wifi_access_point_active() {
            return None;
        }
        let idx = self.wifi_known_state.selected()?;
        if idx < self.wifi.known_networks.len() {
            return self.wifi.known_networks.get(idx);
        }
        if !self.show_unavailable_known_networks {
            return None;
        }
        let hidden_idx = idx.saturating_sub(self.wifi.known_networks.len());
        self.wifi.unavailable_known_networks.get(hidden_idx)
    }

    pub(super) fn selected_new_network(&self) -> Option<&WifiNetwork> {
        if self.wifi_access_point_active() {
            return None;
        }
        let idx = self.wifi_new_state.selected()?;
        if idx < self.wifi.new_networks.len() {
            return self.wifi.new_networks.get(idx);
        }
        if !self.show_hidden_networks {
            return None;
        }
        let hidden_idx = idx.saturating_sub(self.wifi.new_networks.len());
        self.wifi.hidden_networks.get(hidden_idx)
    }

    pub(super) fn selected_wifi_device(&self) -> Option<&WifiDeviceInfo> {
        let idx = self.wifi_adapter_state.selected()?;
        self.wifi.devices.get(idx)
    }

    pub(super) fn init_wifi_states(&mut self) {
        let known_len = self.known_total_len();
        let new_len = self.new_total_len();
        select_first_if_any(&mut self.wifi_known_state, known_len);
        select_first_if_any(&mut self.wifi_new_state, new_len);
        self.restore_wifi_device_selection(None);
        self.ensure_valid_wifi_focus();
    }

    pub(super) fn init_ethernet_state(&mut self) {
        select_first_if_any(&mut self.ethernet_state, self.ethernet.ifaces.len());
    }

    pub(super) fn selected_known_ssid(&self) -> Option<String> {
        self.selected_known_network().map(|n| n.ssid.clone())
    }

    pub(super) fn selected_new_ssid(&self) -> Option<String> {
        self.selected_new_network().map(|n| n.ssid.clone())
    }

    pub(super) fn restore_wifi_selection(
        &mut self,
        known_ssid: Option<String>,
        new_ssid: Option<String>,
        selected_device_iface: Option<String>,
    ) {
        if let Some(ssid) = known_ssid {
            if let Some(idx) = self.wifi.known_networks.iter().position(|n| n.ssid == ssid) {
                self.wifi_known_state.select(Some(idx));
            } else if self.show_unavailable_known_networks {
                if let Some(idx) = self
                    .wifi
                    .unavailable_known_networks
                    .iter()
                    .position(|n| n.ssid == ssid)
                {
                    self.wifi_known_state
                        .select(Some(self.wifi.known_networks.len() + idx));
                } else {
                    let len = self.known_total_len();
                    select_first_if_any(&mut self.wifi_known_state, len);
                }
            } else {
                let len = self.known_total_len();
                select_first_if_any(&mut self.wifi_known_state, len);
            }
        } else {
            let len = self.known_total_len();
            select_first_if_any(&mut self.wifi_known_state, len);
        }

        if let Some(ssid) = new_ssid {
            if let Some(idx) = self.wifi.new_networks.iter().position(|n| n.ssid == ssid) {
                self.wifi_new_state.select(Some(idx));
            } else if self.show_hidden_networks {
                if let Some(idx) = self
                    .wifi
                    .hidden_networks
                    .iter()
                    .position(|n| n.ssid == ssid)
                {
                    self.wifi_new_state
                        .select(Some(self.wifi.new_networks.len() + idx));
                } else {
                    let len = self.new_total_len();
                    select_first_if_any(&mut self.wifi_new_state, len);
                }
            } else {
                let len = self.new_total_len();
                select_first_if_any(&mut self.wifi_new_state, len);
            }
        } else {
            let len = self.new_total_len();
            select_first_if_any(&mut self.wifi_new_state, len);
        }

        self.restore_wifi_device_selection(selected_device_iface);
    }

    pub(super) fn restore_ethernet_selection(&mut self, selected_iface: Option<String>) {
        if let Some(name) = selected_iface
            && let Some(idx) = self.ethernet.ifaces.iter().position(|i| i.name == name)
        {
            self.ethernet_state.select(Some(idx));
            return;
        }
        select_first_if_any(&mut self.ethernet_state, self.ethernet.ifaces.len());
    }

    pub(super) fn ensure_valid_wifi_focus(&mut self) {
        if self.focus_has_items(self.wifi_focus) {
            return;
        }
        for candidate in [
            WifiFocus::KnownNetworks,
            WifiFocus::NewNetworks,
            WifiFocus::Adapter,
        ] {
            if self.focus_has_items(candidate) {
                self.wifi_focus = candidate;
                return;
            }
        }
    }

    pub(super) fn focus_has_items(&self, focus: WifiFocus) -> bool {
        match focus {
            WifiFocus::KnownNetworks => self.known_total_len() > 0,
            WifiFocus::NewNetworks => self.new_total_len() > 0,
            WifiFocus::Adapter => self.device_total_len() > 0,
        }
    }

    pub(crate) fn displayed_wifi_iface(&self) -> Option<&str> {
        self.selected_wifi_device()
            .or(self.wifi.device.as_ref())
            .map(|device| device.iface.as_str())
    }

    pub(crate) fn displayed_wifi_device(&self) -> Option<&WifiDeviceInfo> {
        self.selected_wifi_device().or(self.wifi.device.as_ref())
    }

    pub(super) fn selected_wifi_device_iface(&self) -> Option<String> {
        self.selected_wifi_device()
            .map(|device| device.iface.clone())
    }

    fn restore_wifi_device_selection(&mut self, selected_device_iface: Option<String>) {
        if let Some(iface) = selected_device_iface
            && let Some(idx) = self
                .wifi
                .devices
                .iter()
                .position(|device| device.iface == iface)
        {
            self.wifi_adapter_state.select(Some(idx));
            return;
        }

        if let Some(iface) = self
            .wifi
            .device
            .as_ref()
            .map(|device| device.iface.as_str())
            && let Some(idx) = self
                .wifi
                .devices
                .iter()
                .position(|device| device.iface == iface)
        {
            self.wifi_adapter_state.select(Some(idx));
            return;
        }

        let len = self.device_total_len();
        select_first_if_any(&mut self.wifi_adapter_state, len);
    }

    pub(super) fn preferred_station_iface(&self) -> Option<&str> {
        self.displayed_wifi_iface()
            .or(self.wifi.station_iface.as_deref())
            .or_else(|| self.wifi.ifaces.first().map(String::as_str))
    }

    pub(super) fn preferred_access_point_iface(&self) -> Option<&str> {
        self.displayed_wifi_iface()
            .or(self.wifi.access_point_iface.as_deref())
            .or_else(|| self.wifi.ifaces.first().map(String::as_str))
    }

    pub(super) fn preferred_wifi_iface(&self) -> Option<&str> {
        self.displayed_wifi_iface()
            .or(self.wifi.station_iface.as_deref())
            .or(self.wifi.access_point_iface.as_deref())
            .or_else(|| self.wifi.ifaces.first().map(String::as_str))
    }
}
