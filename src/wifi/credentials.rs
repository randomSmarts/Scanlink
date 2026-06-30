#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WifiCredentials {
    pub security_type: WifiSecurity, // makes fields public, aren't by default for structs
    pub ssid: String,
    pub password: Option<String>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)] // generates some common behavior for this type
pub enum WifiSecurity {
    Wpa,
    Wep,
    Open,
}
