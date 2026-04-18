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
    preserved_sections: Vec<IwdProfileSection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct IwdProfileSection {
    name: String,
    entries: Vec<(String, String)>,
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
            preserved_sections: Vec::new(),
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
                validate_optional_ini_value("Server domain", &self.server_domain_mask)?;
                validate_optional_absolute_path("CA certificate", &self.ca_cert)?;
                validate_optional_absolute_path("Client certificate", &self.client_cert)?;
                validate_optional_absolute_path("Client key", &self.client_key)?;
                validate_optional_ini_value("Key passphrase", &self.key_passphrase)?;
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
                validate_optional_ini_value("Key passphrase", &self.key_passphrase)?;
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

        let mut sections = self.preserved_sections.clone();
        let security = ensure_section(&mut sections, "Security");
        security
            .entries
            .retain(|(key, _)| !is_managed_security_key(key));
        security.entries.extend(self.render_security_entries());

        if self.preserved_sections.is_empty() {
            let settings = ensure_section(&mut sections, "Settings");
            settings
                .entries
                .retain(|(key, _)| !key.eq_ignore_ascii_case("AutoConnect"));
            settings
                .entries
                .push(("AutoConnect".to_string(), "true".to_string()));
        }

        Ok(render_iwd_profile(&sections))
    }

    pub fn from_iwd_profile(ssid: &str, raw: &str) -> Result<Self> {
        let sections = parse_iwd_profile(raw);
        let security = parse_section_entries(&sections, "Security");
        let method = security
            .get("EAP-Method")
            .map(|value| value.trim().to_ascii_uppercase())
            .unwrap_or_default();

        let mut profile = Self::new_for_ssid(ssid);
        match method.as_str() {
            "PEAP" => {
                let phase2_method = security
                    .get("EAP-PEAP-Phase2-Method")
                    .map(|value| parse_phase2_method(value))
                    .transpose()?
                    .unwrap_or(EnterprisePhase2Method::Mschapv2);
                profile.method = if ssid.eq_ignore_ascii_case("eduroam")
                    && phase2_method == EnterprisePhase2Method::Mschapv2
                {
                    EnterpriseMethod::Eduroam
                } else {
                    EnterpriseMethod::Peap
                };
                profile.identity = get_profile_value(&security, "EAP-Identity");
                profile.server_domain_mask =
                    get_profile_value(&security, "EAP-PEAP-ServerDomainMask");
                profile.ca_cert = get_profile_value(&security, "EAP-PEAP-CACert");
                profile.client_cert = get_profile_value(&security, "EAP-PEAP-ClientCert");
                profile.client_key = get_profile_value(&security, "EAP-PEAP-ClientKey");
                profile.key_passphrase =
                    get_profile_value(&security, "EAP-PEAP-ClientKeyPassphrase");
                profile.phase2_method = phase2_method;
                profile.phase2_identity = get_profile_value(&security, "EAP-PEAP-Phase2-Identity");
                profile.phase2_password = get_profile_value(&security, "EAP-PEAP-Phase2-Password");
            }
            "TTLS" => {
                profile.method = EnterpriseMethod::Ttls;
                profile.identity = get_profile_value(&security, "EAP-Identity");
                profile.server_domain_mask =
                    get_profile_value(&security, "EAP-TTLS-ServerDomainMask");
                profile.ca_cert = get_profile_value(&security, "EAP-TTLS-CACert");
                profile.client_cert = get_profile_value(&security, "EAP-TTLS-ClientCert");
                profile.client_key = get_profile_value(&security, "EAP-TTLS-ClientKey");
                profile.key_passphrase =
                    get_profile_value(&security, "EAP-TTLS-ClientKeyPassphrase");
                profile.phase2_method = security
                    .get("EAP-TTLS-Phase2-Method")
                    .map(|value| parse_phase2_method(value))
                    .transpose()?
                    .unwrap_or(EnterprisePhase2Method::Mschapv2);
                profile.phase2_identity = get_profile_value(&security, "EAP-TTLS-Phase2-Identity");
                profile.phase2_password = get_profile_value(&security, "EAP-TTLS-Phase2-Password");
            }
            "TLS" => {
                profile.method = EnterpriseMethod::Tls;
                profile.ca_cert = get_profile_value(&security, "EAP-TLS-CACert");
                profile.identity = get_profile_value(&security, "EAP-Identity");
                profile.client_cert = get_profile_value(&security, "EAP-TLS-ClientCert");
                profile.client_key = get_profile_value(&security, "EAP-TLS-ClientKey");
                profile.key_passphrase =
                    get_profile_value(&security, "EAP-TLS-ClientKeyPassphrase");
            }
            "PWD" => {
                profile.method = EnterpriseMethod::Pwd;
                profile.identity = get_profile_value(&security, "EAP-Identity");
                profile.password = get_profile_value(&security, "EAP-Password");
            }
            _ => bail!("unsupported EAP method in existing profile"),
        }

        profile.preserved_sections = sections;
        Ok(profile)
    }

    fn render_security_entries(&self) -> Vec<(String, String)> {
        let mut entries = Vec::new();

        match self.method {
            EnterpriseMethod::Eduroam => {
                push_entry(&mut entries, "EAP-Method", "PEAP");
                push_entry(&mut entries, "EAP-Identity", &self.identity);
                push_entry(&mut entries, "EAP-PEAP-Phase2-Method", "MSCHAPV2");
                push_entry(
                    &mut entries,
                    "EAP-PEAP-Phase2-Identity",
                    &self.phase2_identity,
                );
                push_entry(
                    &mut entries,
                    "EAP-PEAP-Phase2-Password",
                    &self.phase2_password,
                );
            }
            EnterpriseMethod::Peap => {
                push_entry(&mut entries, "EAP-Method", "PEAP");
                push_entry(&mut entries, "EAP-Identity", &self.identity);
                push_optional_entry(
                    &mut entries,
                    "EAP-PEAP-ServerDomainMask",
                    &self.server_domain_mask,
                );
                push_optional_entry(&mut entries, "EAP-PEAP-CACert", &self.ca_cert);
                push_optional_entry(&mut entries, "EAP-PEAP-ClientCert", &self.client_cert);
                push_optional_entry(&mut entries, "EAP-PEAP-ClientKey", &self.client_key);
                push_optional_entry(
                    &mut entries,
                    "EAP-PEAP-ClientKeyPassphrase",
                    &self.key_passphrase,
                );
                push_entry(
                    &mut entries,
                    "EAP-PEAP-Phase2-Method",
                    self.phase2_method.label(),
                );
                push_entry(
                    &mut entries,
                    "EAP-PEAP-Phase2-Identity",
                    &self.phase2_identity,
                );
                push_entry(
                    &mut entries,
                    "EAP-PEAP-Phase2-Password",
                    &self.phase2_password,
                );
            }
            EnterpriseMethod::Ttls => {
                push_entry(&mut entries, "EAP-Method", "TTLS");
                push_entry(&mut entries, "EAP-Identity", &self.identity);
                push_optional_entry(
                    &mut entries,
                    "EAP-TTLS-ServerDomainMask",
                    &self.server_domain_mask,
                );
                push_optional_entry(&mut entries, "EAP-TTLS-CACert", &self.ca_cert);
                push_optional_entry(&mut entries, "EAP-TTLS-ClientCert", &self.client_cert);
                push_optional_entry(&mut entries, "EAP-TTLS-ClientKey", &self.client_key);
                push_optional_entry(
                    &mut entries,
                    "EAP-TTLS-ClientKeyPassphrase",
                    &self.key_passphrase,
                );
                push_entry(
                    &mut entries,
                    "EAP-TTLS-Phase2-Method",
                    self.phase2_method.label(),
                );
                push_entry(
                    &mut entries,
                    "EAP-TTLS-Phase2-Identity",
                    &self.phase2_identity,
                );
                push_entry(
                    &mut entries,
                    "EAP-TTLS-Phase2-Password",
                    &self.phase2_password,
                );
            }
            EnterpriseMethod::Tls => {
                push_entry(&mut entries, "EAP-Method", "TLS");
                push_entry(&mut entries, "EAP-TLS-CACert", &self.ca_cert);
                push_entry(&mut entries, "EAP-Identity", &self.identity);
                push_entry(&mut entries, "EAP-TLS-ClientCert", &self.client_cert);
                push_entry(&mut entries, "EAP-TLS-ClientKey", &self.client_key);
                push_optional_entry(
                    &mut entries,
                    "EAP-TLS-ClientKeyPassphrase",
                    &self.key_passphrase,
                );
            }
            EnterpriseMethod::Pwd => {
                push_entry(&mut entries, "EAP-Method", "PWD");
                push_entry(&mut entries, "EAP-Identity", &self.identity);
                push_entry(&mut entries, "EAP-Password", &self.password);
            }
        }

        entries
    }
}

fn parse_iwd_profile(raw: &str) -> Vec<IwdProfileSection> {
    let mut sections = Vec::new();
    let mut current_section: Option<usize> = None;

    for line in raw.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            sections.push(IwdProfileSection {
                name: line
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .trim()
                    .to_string(),
                entries: Vec::new(),
            });
            current_section = Some(sections.len() - 1);
            continue;
        }
        let Some(current_section) = current_section else {
            continue;
        };
        if let Some((key, value)) = line.split_once('=') {
            sections[current_section]
                .entries
                .push((key.trim().to_string(), value.to_string()));
        }
    }

    sections
}

fn parse_section_entries(
    sections: &[IwdProfileSection],
    section_name: &str,
) -> std::collections::HashMap<String, String> {
    sections
        .iter()
        .find(|section| section.name.eq_ignore_ascii_case(section_name))
        .map(|section| {
            section
                .entries
                .iter()
                .map(|(key, value)| (key.clone(), value.trim().to_string()))
                .collect()
        })
        .unwrap_or_default()
}

fn ensure_section<'a>(
    sections: &'a mut Vec<IwdProfileSection>,
    name: &str,
) -> &'a mut IwdProfileSection {
    if let Some(index) = sections
        .iter()
        .position(|section| section.name.eq_ignore_ascii_case(name))
    {
        return &mut sections[index];
    }

    sections.push(IwdProfileSection {
        name: name.to_string(),
        entries: Vec::new(),
    });
    sections.last_mut().expect("section was just pushed")
}

fn render_iwd_profile(sections: &[IwdProfileSection]) -> String {
    let mut out = String::new();

    for (index, section) in sections.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push('[');
        out.push_str(&section.name);
        out.push_str("]\n");
        for (key, value) in &section.entries {
            out.push_str(key);
            out.push('=');
            out.push_str(value);
            out.push('\n');
        }
    }

    out
}

fn is_managed_security_key(key: &str) -> bool {
    [
        "EAP-Method",
        "EAP-Identity",
        "EAP-Password",
        "EAP-PEAP-ServerDomainMask",
        "EAP-PEAP-CACert",
        "EAP-PEAP-ClientCert",
        "EAP-PEAP-ClientKey",
        "EAP-PEAP-ClientKeyPassphrase",
        "EAP-PEAP-Phase2-Method",
        "EAP-PEAP-Phase2-Identity",
        "EAP-PEAP-Phase2-Password",
        "EAP-TTLS-ServerDomainMask",
        "EAP-TTLS-CACert",
        "EAP-TTLS-ClientCert",
        "EAP-TTLS-ClientKey",
        "EAP-TTLS-ClientKeyPassphrase",
        "EAP-TTLS-Phase2-Method",
        "EAP-TTLS-Phase2-Identity",
        "EAP-TTLS-Phase2-Password",
        "EAP-TLS-CACert",
        "EAP-TLS-ClientCert",
        "EAP-TLS-ClientKey",
        "EAP-TLS-ClientKeyPassphrase",
    ]
    .iter()
    .any(|managed| key.eq_ignore_ascii_case(managed))
}

fn get_profile_value(security: &std::collections::HashMap<String, String>, key: &str) -> String {
    security.get(key).cloned().unwrap_or_default()
}

fn parse_phase2_method(value: &str) -> Result<EnterprisePhase2Method> {
    match value.trim().to_ascii_uppercase().as_str() {
        "MSCHAPV2" => Ok(EnterprisePhase2Method::Mschapv2),
        "SIM" => Ok(EnterprisePhase2Method::Sim),
        "GTC" => Ok(EnterprisePhase2Method::Gtc),
        "TUNNELED-CHAP" => Ok(EnterprisePhase2Method::TunneledChap),
        "TUNNELED-MSCHAP" => Ok(EnterprisePhase2Method::TunneledMschap),
        "TUNNELED-MSCHAPV2" => Ok(EnterprisePhase2Method::TunneledMschapv2),
        "TUNNELED-PAP" => Ok(EnterprisePhase2Method::TunneledPap),
        _ => bail!("unsupported phase 2 method in existing profile"),
    }
}

fn require(label: &str, value: &str) -> Result<()> {
    validate_optional_ini_value(label, value)?;
    if value.trim().is_empty() {
        bail!("{label} cannot be empty");
    }
    Ok(())
}

fn validate_optional_ini_value(label: &str, value: &str) -> Result<()> {
    if value.contains('\n') || value.contains('\r') || value.contains('\0') {
        bail!("{label} cannot contain line breaks");
    }
    Ok(())
}

fn validate_optional_absolute_path(label: &str, value: &str) -> Result<()> {
    validate_optional_ini_value(label, value)?;
    if value.trim().is_empty() {
        return Ok(());
    }
    validate_required_absolute_path(label, value)
}

fn validate_required_absolute_path(label: &str, value: &str) -> Result<()> {
    validate_optional_ini_value(label, value)?;
    let path = Path::new(value.trim());
    if !path.is_absolute() {
        bail!("{label} must be an absolute path");
    }
    if !path.exists() {
        bail!("{label} file was not found");
    }
    Ok(())
}

fn push_entry(out: &mut Vec<(String, String)>, key: &str, value: &str) {
    out.push((key.to_string(), value.to_string()));
}

fn push_optional_entry(out: &mut Vec<(String, String)>, key: &str, value: &str) {
    if !value.trim().is_empty() {
        push_entry(out, key, value);
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
            preserved_sections: Vec::new(),
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
            preserved_sections: Vec::new(),
        };

        assert!(profile.validate().is_err());
    }

    #[test]
    fn parse_existing_peap_profile_round_trips() {
        let raw = "\
[Security]
EAP-Method=PEAP
EAP-Identity=user@example.com
EAP-PEAP-ServerDomainMask=example.com
EAP-PEAP-CACert=/etc/ssl/certs/example.pem
EAP-PEAP-Phase2-Method=GTC
EAP-PEAP-Phase2-Identity=user
EAP-PEAP-Phase2-Password=secret

[Settings]
AutoConnect=true
";

        let profile = WifiEnterpriseProfile::from_iwd_profile("corp", raw).unwrap();
        assert_eq!(profile.method, EnterpriseMethod::Peap);
        assert_eq!(profile.identity, "user@example.com");
        assert_eq!(profile.server_domain_mask, "example.com");
        assert_eq!(profile.ca_cert, "/etc/ssl/certs/example.pem");
        assert_eq!(profile.phase2_method, EnterprisePhase2Method::Gtc);
        assert_eq!(profile.phase2_identity, "user");
        assert_eq!(profile.phase2_password, "secret");
    }

    #[test]
    fn parse_existing_eduroam_profile_sets_eduroam_mode() {
        let raw = "\
[Security]
EAP-Method=PEAP
EAP-Identity=user@school.edu
EAP-PEAP-Phase2-Method=MSCHAPV2
EAP-PEAP-Phase2-Identity=user
EAP-PEAP-Phase2-Password=secret
";

        let profile = WifiEnterpriseProfile::from_iwd_profile("eduroam", raw).unwrap();
        assert_eq!(profile.method, EnterpriseMethod::Eduroam);
        assert_eq!(profile.phase2_identity, "user");
    }

    #[test]
    fn existing_profile_preserves_unmanaged_entries_and_settings() {
        let raw = "\
[Security]
EAP-Method=PEAP
EAP-Identity=user@example.com
EAP-PEAP-Phase2-Method=MSCHAPV2
EAP-PEAP-Phase2-Identity=user
EAP-PEAP-Phase2-Password=secret
VendorOption=keep-me

[Settings]
AutoConnect=false
Hidden=true
";

        let mut profile = WifiEnterpriseProfile::from_iwd_profile("corp", raw).unwrap();
        profile.phase2_password = "updated".to_string();

        let rendered = profile.to_iwd_profile().unwrap();
        assert!(rendered.contains("VendorOption=keep-me"));
        assert!(rendered.contains("AutoConnect=false"));
        assert!(rendered.contains("Hidden=true"));
        assert!(rendered.contains("EAP-PEAP-Phase2-Password=updated"));
        assert!(!rendered.contains("AutoConnect=true"));
    }

    #[test]
    fn profile_rejects_line_breaks_in_values() {
        let mut profile = WifiEnterpriseProfile::new_for_ssid("corp");
        profile.method = EnterpriseMethod::Pwd;
        profile.identity = "user".to_string();
        profile.password = "bad\nvalue".to_string();

        assert_eq!(
            profile.validate().unwrap_err().to_string(),
            "Password cannot contain line breaks"
        );
    }
}
