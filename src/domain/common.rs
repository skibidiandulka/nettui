use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Wifi,
    Ethernet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WifiFocus {
    KnownNetworks,
    NewNetworks,
    Adapter,
}

#[derive(Debug, Clone, Copy)]
pub enum StartupTabPolicy {
    PreferActive,
    ForceWifi,
    ForceEthernet,
}

#[derive(Debug, Clone, Copy)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub kind: ToastKind,
    pub msg: String,
    pub until: Instant,
}
