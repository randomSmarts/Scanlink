use std::process::{ Command, Output };

use super::connector::{ WifiConnectError, WifiConnector };
use super::credentials::{ WifiCredentials, WifiSecurity };

pub struct LinuxNetworkManagerConnector;
// interface_name: String,
// unit struct, needed when you only need a type to attach behavior to

// impl LinuxNetworkManagerConnector {
//     fn new(interface_name: String) -> Self {
//         LinuxNetworkManagerConnector {}
//     }
// }

impl LinuxNetworkManagerConnector {
    // no self is needed because it belongs to the type, not to the instance
    fn check_output_status(output: Output) -> Result<(), WifiConnectError> {
        if output.status.success() {
            Ok(())
        } else {
            Err(
                WifiConnectError::CommandFailed(String::from_utf8_lossy(&output.stderr).to_string())
            )
        }
    }
    fn delete_network_profile(&self, credentials: &WifiCredentials) {
        let _ = Command::new("nmcli") // deletes old broken profiles with the corresponding SSID name, don't care about errors
            .args(["connection", "delete", credentials.ssid.as_str()])
            .output();
    }
}

impl WifiConnector for LinuxNetworkManagerConnector {
    fn connect(&self, credentials: &WifiCredentials) -> Result<(), WifiConnectError> {
        match credentials.security_type {
            WifiSecurity::Wpa => {
                let password = credentials.password
                    .as_ref() // we do .as_ref() for all because credentials, where the password comes from, is borrowed
                    .ok_or(WifiConnectError::MissingPassword)?;

                self.delete_network_profile(credentials); // deletes old broken profiles with the corresponding SSID name, don't care about errors

                // needed b/c what type of security management needed to use isn't stated w/reg. nmcli command
                let add_output = Command::new("nmcli") // create a proper WPA profile
                    .args([
                        "connection",
                        "add",
                        "type", // wifi connection this is
                        "wifi",
                        "ifname", // use any available wi-fi interface
                        "*",
                        "con-name", // name saved connection profile after the SSID
                        credentials.ssid.as_str(), // as_str() because that's how the terminal likes it
                        "ssid", // actual wi-fi SSID name to connect to
                        credentials.ssid.as_str(),
                        "wifi-sec.key-mgmt", // these are the missing settings (this and below)
                        "wpa-psk",
                        "wifi-sec.psk", // This is WPA/WPA-2 Personal using a pre-shared key
                        password.as_str(),
                    ])
                    .output()?; // ? used because it returns Result, where if the command failed then it returns the error immediately

                Self::check_output_status(add_output)?; // if returns Ok(()) continue, else return immediately the error from the current function

                let up_output = Command::new("nmcli") // connect the profile just created (activating it)
                    .args(["connection", "up", credentials.ssid.as_str()])
                    .output()?;

                Self::check_output_status(up_output)
            }
            WifiSecurity::Wep => { Err(WifiConnectError::UnsupportedSecurity) }
            WifiSecurity::Open => {
                self.delete_network_profile(credentials);

                let add_output = Command::new("nmcli")
                    .args([
                        "connection",
                        "add",
                        "type",
                        "wifi",
                        "ifname",
                        "*",
                        "con-name",
                        credentials.ssid.as_str(),
                        "ssid",
                        credentials.ssid.as_str(),
                    ])
                    .output()?;

                Self::check_output_status(add_output)?;

                let up_output = Command::new("nmcli")
                    .args(["connection", "up", credentials.ssid.as_str()])
                    .output()?;

                Self::check_output_status(up_output)
            }
        }
    }
}
