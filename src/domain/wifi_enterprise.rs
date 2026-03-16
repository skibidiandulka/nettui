use anyhow::{Result, bail};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnterpriseMethod {
    Eduroam,
    Peap,
    Ttls,
    Tls,
    Pwd,
}

impl EnterpriseMethod {
    pub const ALL: [Self; 5] = [Self::Eduroam, Self::Peap, Self::Ttls, Self::Tls, Self::Pwd];

    pub fn label(self) -> &'static str {
        match self {
            Self::Eduroam => "Eduroam",
            Self::Peap => "PEAP",
            Self::Ttls => "TTLS",
            Self::Tls => "TLS",
            Self::Pwd => "PWD",
        }
    }

    pub fn default_for_ssid(ssid: &str) -> Self {
        if ssid.eq_ignore_ascii_case("eduroam") {
            Self::Eduroam
        } else {
            Self::Peap
        }
    }

    pub fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        Self::ALL[(index + 1) % Self::ALL.len()]
    }

    pub fn prev(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|candidate| *candidate == self)
            .unwrap_or(0);
        Self::ALL[(index + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnterprisePhase2Method {
    Mschapv2,
    Sim,
    Gtc,
    TunneledChap,
    TunneledMschap,
    TunneledMschapv2,
    TunneledPap,
}

impl EnterprisePhase2Method {
    pub const PEAP_ALL: [Self; 3] = [Self::Mschapv2, Self::Sim, Self::Gtc];
    pub const TTLS_ALL: [Self; 5] = [
        Self::Mschapv2,
        Self::TunneledChap,
        Self::TunneledMschap,
        Self::TunneledMschapv2,
        Self::TunneledPap,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Self::Mschapv2 => "MSCHAPV2",
            Self::Sim => "SIM",
            Self::Gtc => "GTC",
            Self::TunneledChap => "Tunneled-CHAP",
            Self::TunneledMschap => "Tunneled-MSCHAP",
            Self::TunneledMschapv2 => "Tunneled-MSCHAPv2",
            Self::TunneledPap => "Tunneled-PAP",
        }
    }

    pub fn next_for_method(self, method: EnterpriseMethod) -> Self {
        cycle_phase2(method, self, true)
    }

    pub fn prev_for_method(self, method: EnterpriseMethod) -> Self {
        cycle_phase2(method, self, false)
    }
}

fn cycle_phase2(
    method: EnterpriseMethod,
    current: EnterprisePhase2Method,
    forward: bool,
) -> EnterprisePhase2Method {
    let values = match method {
        EnterpriseMethod::Peap => &EnterprisePhase2Method::PEAP_ALL[..],
        EnterpriseMethod::Ttls => &EnterprisePhase2Method::TTLS_ALL[..],
        EnterpriseMethod::Eduroam => &[EnterprisePhase2Method::Mschapv2][..],
        _ => &[EnterprisePhase2Method::Mschapv2][..],
    };
    let index = values
        .iter()
        .position(|candidate| *candidate == current)
        .unwrap_or(0);
    if forward {
        values[(index + 1) % values.len()]
    } else {
        values[(index + values.len() - 1) % values.len()]
    }
}

#[derive(Debug, Clone)]
pub struct WifiEnterpriseProfile {
    pub method: EnterpriseMethod,
    pub identity: String,
    pub server_domain_mask: String,
    pub ca_cert: String,
    pub client_cert: String,
    pub client_key: String,
    pub key_passphrase: String,
    pub phase2_method: EnterprisePhase2Method,
    pub phase2_identity: String,
    pub phase2_password: String,
    pub password: String,
}

impl WifiEnterpriseProfile {
    pub fn new_for_ssid(ssid: &str) -> Self {
        Self {
            method: EnterpriseMethod::default_for_ssid(ssid),
            identity: String::new(),
            server_domain_mask: String::new(),
            ca_cert: String::new(),
            client_cert: String::new(),
            client_key: String::new(),
            key_passphrase: String::new(),
            phase2_method: EnterprisePhase2Method::Mschapv2,
            phase2_identity: String::new(),
            phase2_password: String::new(),
            password: String::new(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self.method {
            EnterpriseMethod::Eduroam => {
                require("Identity", &self.identity)?;
                require("Phase 2 identity", &self.phase2_identity)?;
                require("Phase 2 password", &self.phase2_password)?;
            }
            EnterpriseMethod::Peap | EnterpriseMethod::Ttls => {
                require("Identity", &self.identity)?;
                validate_optional_absolute_path("CA certificate", &self.ca_cert)?;
                validate_optional_absolute_path("Client certificate", &self.client_cert)?;
                validate_optional_absolute_path("Client key", &self.client_key)?;
                require("Phase 2 identity", &self.phase2_identity)?;
                require("Phase 2 password", &self.phase2_password)?;
            }
            EnterpriseMethod::Tls => {
                require("CA certificate", &self.ca_cert)?;
                validate_required_absolute_path("CA certificate", &self.ca_cert)?;
                require("Identity", &self.identity)?;
                require("Client certificate", &self.client_cert)?;
                validate_required_absolute_path("Client certificate", &self.client_cert)?;
                require("Client key", &self.client_key)?;
                validate_required_absolute_path("Client key", &self.client_key)?;
            }
            EnterpriseMethod::Pwd => {
                require("Identity", &self.identity)?;
                require("Password", &self.password)?;
            }
        }
        Ok(())
    }

    pub fn to_iwd_profile(&self) -> Result<String> {
        self.validate()?;

        let mut text = String::from("[Security]\n");
        match self.method {
            EnterpriseMethod::Eduroam => {
                push_line(&mut text, "EAP-Method", "PEAP");
                push_line(&mut text, "EAP-Identity", &self.identity);
                push_line(&mut text, "EAP-PEAP-Phase2-Method", "MSCHAPV2");
                push_line(&mut text, "EAP-PEAP-Phase2-Identity", &self.phase2_identity);
                push_line(&mut text, "EAP-PEAP-Phase2-Password", &self.phase2_password);
            }
            EnterpriseMethod::Peap => {
                push_line(&mut text, "EAP-Method", "PEAP");
                push_line(&mut text, "EAP-Identity", &self.identity);
                push_optional_line(
                    &mut text,
                    "EAP-PEAP-ServerDomainMask",
                    &self.server_domain_mask,
                );
                push_optional_line(&mut text, "EAP-PEAP-CACert", &self.ca_cert);
                push_optional_line(&mut text, "EAP-PEAP-ClientCert", &self.client_cert);
                push_optional_line(&mut text, "EAP-PEAP-ClientKey", &self.client_key);
                push_optional_line(
                    &mut text,
                    "EAP-PEAP-ClientKeyPassphrase",
                    &self.key_passphrase,
                );
                push_line(
                    &mut text,
                    "EAP-PEAP-Phase2-Method",
                    self.phase2_method.label(),
                );
                push_line(&mut text, "EAP-PEAP-Phase2-Identity", &self.phase2_identity);
                push_line(&mut text, "EAP-PEAP-Phase2-Password", &self.phase2_password);
            }
            EnterpriseMethod::Ttls => {
                push_line(&mut text, "EAP-Method", "TTLS");
                push_line(&mut text, "EAP-Identity", &self.identity);
                push_optional_line(
                    &mut text,
                    "EAP-TTLS-ServerDomainMask",
                    &self.server_domain_mask,
                );
                push_optional_line(&mut text, "EAP-TTLS-CACert", &self.ca_cert);
                push_optional_line(&mut text, "EAP-TTLS-ClientCert", &self.client_cert);
                push_optional_line(&mut text, "EAP-TTLS-ClientKey", &self.client_key);
                push_optional_line(
                    &mut text,
                    "EAP-TTLS-ClientKeyPassphrase",
                    &self.key_passphrase,
                );
                push_line(
                    &mut text,
                    "EAP-TTLS-Phase2-Method",
                    self.phase2_method.label(),
                );
                push_line(&mut text, "EAP-TTLS-Phase2-Identity", &self.phase2_identity);
                push_line(&mut text, "EAP-TTLS-Phase2-Password", &self.phase2_password);
            }
            EnterpriseMethod::Tls => {
                push_line(&mut text, "EAP-Method", "TLS");
                push_line(&mut text, "EAP-TLS-CACert", &self.ca_cert);
                push_line(&mut text, "EAP-Identity", &self.identity);
                push_line(&mut text, "EAP-TLS-ClientCert", &self.client_cert);
                push_line(&mut text, "EAP-TLS-ClientKey", &self.client_key);
                push_optional_line(
                    &mut text,
                    "EAP-TLS-ClientKeyPassphrase",
                    &self.key_passphrase,
                );
            }
            EnterpriseMethod::Pwd => {
                push_line(&mut text, "EAP-Method", "PWD");
                push_line(&mut text, "EAP-Identity", &self.identity);
                push_line(&mut text, "EAP-Password", &self.password);
            }
        }

        text.push_str("\n[Settings]\nAutoConnect=true\n");
        Ok(text)
    }
}

fn require(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("{label} cannot be empty");
    }
    Ok(())
}

fn validate_optional_absolute_path(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Ok(());
    }
    validate_required_absolute_path(label, value)
}

fn validate_required_absolute_path(label: &str, value: &str) -> Result<()> {
    let path = Path::new(value.trim());
    if !path.is_absolute() {
        bail!("{label} must be an absolute path");
    }
    if !path.exists() {
        bail!("{label} file was not found");
    }
    Ok(())
}

fn push_line(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push('=');
    out.push_str(value.trim());
    out.push('\n');
}

fn push_optional_line(out: &mut String, key: &str, value: &str) {
    if !value.trim().is_empty() {
        push_line(out, key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eduroam_profile_renders_expected_lines() {
        let profile = WifiEnterpriseProfile {
            method: EnterpriseMethod::Eduroam,
            identity: "user@school".to_string(),
            server_domain_mask: String::new(),
            ca_cert: String::new(),
            client_cert: String::new(),
            client_key: String::new(),
            key_passphrase: String::new(),
            phase2_method: EnterprisePhase2Method::Mschapv2,
            phase2_identity: "user".to_string(),
            phase2_password: "secret".to_string(),
            password: String::new(),
        };

        let rendered = profile.to_iwd_profile().unwrap();
        assert!(rendered.contains("EAP-Method=PEAP"));
        assert!(rendered.contains("EAP-PEAP-Phase2-Password=secret"));
    }

    #[test]
    fn pwd_profile_requires_password() {
        let profile = WifiEnterpriseProfile {
            method: EnterpriseMethod::Pwd,
            identity: "user".to_string(),
            server_domain_mask: String::new(),
            ca_cert: String::new(),
            client_cert: String::new(),
            client_key: String::new(),
            key_passphrase: String::new(),
            phase2_method: EnterprisePhase2Method::Mschapv2,
            phase2_identity: String::new(),
            phase2_password: String::new(),
            password: String::new(),
        };

        assert!(profile.validate().is_err());
    }
}
