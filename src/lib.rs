#[cfg(feature = "qr")]
pub mod camera;

#[cfg(feature = "qr")] // cfg means only compile this code if this Cargo feature is enabled
pub mod qr_scanner;

#[cfg(feature = "wifi")]
pub mod wifi;

#[cfg(feature = "qr")]
pub use camera::{save_camera_preview, setup_camera};

#[cfg(feature = "qr")]
pub use qr_scanner::{build_qr_scanner, scan_qr_payloads_from_camera};

#[cfg(feature = "wifi")]
pub use wifi::{
    connect_to_wifi_from_qr_payloads,
    parse_wifi_credentials,
    WifiConnectError,
    WifiConnector,
    WifiCredentials,
    WifiSecurity,
};

#[cfg(feature = "linux-network-manager")]
pub use wifi::LinuxNetworkManagerConnector;