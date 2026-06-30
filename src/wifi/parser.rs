use regex::Regex;
use std::sync::LazyLock;

use super::credentials::{ WifiCredentials, WifiSecurity };

static WIFI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // || {} is a closure that stores the code that runs once (an anonymous
    // function of sorts)
    Regex::new(
        r"^WIFI:T:(?P<wifi_type>[^;]+);S:(?P<wifi_ssid>[^;]+);(?:P:(?P<wifi_pwd>[^;]*);)?.*$" // + means ≥1 chars, * means ≥0 chars
    ).unwrap()
}); // poisoning is where when you create a shared resource, it runs some code inside LazyLock::new the first time it is accessed; if that code panics, then it poisons the the lock and any future attempt to use the data in the lock will panic and crash the program

pub fn parse_wifi_credentials(qr_payloads: &[String]) -> Option<WifiCredentials> {
    // reference to a slice of strings, don't care what container they came from
    // WIFI:T:WPA;S:MyNetworkName;P:MyPassword;;

    for payload in qr_payloads {
        if let Some(caps) = WIFI_REGEX.captures(payload) {
            // Rust auto converts &String to &str
            let ssid = caps["wifi_ssid"].to_string();

            let raw_security_type = caps["wifi_type"].to_string(); // &caps["wifi_type"].to_string(); means to create a new string and then borrow it
            let security_type = match raw_security_type.as_str() {
                "WPA" | "WPA2" | "WPA3" => WifiSecurity::Wpa,
                "WEP" => WifiSecurity::Wep,
                "nopass" | "" => WifiSecurity::Open,
                _ => {
                    // anything else to just continue to next payload
                    continue;
                }
            };

            let password_text = caps // password is optional
                .name("wifi_pwd") // try to get the named capture group, returning an Option
                .map(|m| m.as_str().to_string()) // tiny closure function, converts whatever is inside the Option to a string (if not None)
                .unwrap_or_default(); // returns string or default which is String::new() / ""
            let password = match security_type {
                // Converts password back into an Option, also for checking where if WifiSecurity is given but no password then go to next payload
                WifiSecurity::Wpa | WifiSecurity::Wep => {
                    if password_text.is_empty() {
                        continue;
                    }
                    Some(password_text)
                }
                WifiSecurity::Open => None,
            };

            return Some(WifiCredentials { security_type, ssid, password });
        } else {
            println!("Regex did not match.");
        }
    }
    None
}
