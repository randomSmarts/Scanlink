use core::error;
use std::{ io, process::Output };

use regex::Regex;
use std::sync::LazyLock; // from Rust standard library; lets you create something globally, only initializing it the first time it is used
use std::process::Command; // Lets Rust run system commands

use nokhwa::{
    Camera,
    pixel_format::{ LumaFormat },
    utils::{ CameraIndex, RequestedFormat, RequestedFormatType },
};

use zedbar::config::*; // Bring everything public from zedbar::config into scope
use zedbar::{ DecoderConfig, Scanner };

static WIFI_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // || {} is a closure that stores the code that runs once (an anonymous
    // function of sorts)
    Regex::new(
        r"^WIFI:T:(?P<wifi_type>[^;]+);S:(?P<wifi_ssid>[^;]+);(?:P:(?P<wifi_pwd>[^;]*);)?.*$" // + means ≥1 chars, * means ≥0 chars
    ).unwrap()
}); // poisoning is where when you create a shared resource, it runs some code inside LazyLock::new the first time it is accessed; if that code panics, then it poisons the the lock and any future attempt to use the data in the lock will panic and crash the program

fn setup_camera(index: CameraIndex, requested: RequestedFormat) -> Camera {
    let mut camera = Camera::new(index, requested).expect("Failed to create camera"); // mut because opening/using the camera changes its internal state
    camera.open_stream().expect("Failed to open camera stream."); // starts the camera, changes internal state; use expect because it returns a Result

    warm_up_camera(&mut camera); // removing unnecessary flame flushing from hot path because camera acquisition latency dominates QR decode latency
    camera
}

fn main() {
    let mut user_input: String = String::new();

    let index = CameraIndex::Index(0); // CameraIndex is a type, and so using CameraIndex::Index(0) means ot use camera number 0

    let requested = RequestedFormat::new::<LumaFormat>(
        // creates a camera format request, uses turbofish syntax telling rust
        // call new and use RgbFormat as the format type with highest frame rate
        RequestedFormatType::AbsoluteHighestFrameRate
    );

    let mut camera = setup_camera(index, requested);

    let config = DecoderConfig::new() // settings object
        .enable(QrCode) // tell scanner to look for QR codes
        .test_inverted(true) // if normal scan finds nothing, try scanning an inverted image (white squares on black background)
        .retry_undecoded_regions(true) // If zedbar sees QR finder patterns but can't decode the QR, auto crop/upscale the sus regions and try again
        .scan_density(2, 2); // density control, about scanning every x rows and every y columns

    let mut scanner = Scanner::with_config(config);

    loop {
        // loop can return a value to a var; runs ≥ 1
        user_input.clear();

        println!("Type command: p = preview, c = capture, q = quit");

        io::stdin()
            .read_line(&mut user_input) // mutable reference to fill in user_input since it's a String buffer
            .expect("failed to read input"); // expect says it should succeed, if it doesn't crash program and print this message; unwraps the Result and if Ok(...) then returns inside value of how many bytes it was read (which we don't care about); if Ok(...) fills in String buffer

        let command = user_input.trim().to_lowercase(); // when I press enter in terminal, it appens \n so need to get rid of it, and converting to lowercase

        if command == "q" {
            println!("Thank you for using this service. Good bye.");
            break;
        } else if command == "p" {
            save_camera_preview(&mut camera);
        } else if command == "c" {
            let data_result = fallback_detect_qr_code_loop(&mut camera, &mut scanner);
            println!("Data detected: {:?}", data_result);
            let connector = LinuxNetworkManagerConnector;
            let result = connect_to_wifi_from_qr_payloads(&data_result, &connector);

            match result {
                Ok(()) => {
                    println!("Connected to Wi-Fi successfully.");
                }
                Err(error) => {
                    println!("Failed to connect to Wi-Fi: {:?}", error); // prints real error directly
                }
            }
            // println!("The data in the QR code is: {}",
            // fallback_detect_qr_code_loop(&mut camera, &mut scanner));
        }
    }
}

fn save_camera_preview(camera: &mut Camera) {
    let output_path = "camera_preview.png";

    for _ in 0..10 {
        let _ = camera.frame(); // getting rid of buffers
    }
    let frame = match camera.frame() {
        Ok(frame) => frame,
        Err(error) => {
            println!("Failed to capture preview frame: {:?}", error);
            return;
        }
    };

    let width = frame.resolution().width();
    let height = frame.resolution().height();

    let decoded_frame = match frame.decode_image::<LumaFormat>() {
        Ok(decoded_frame) => decoded_frame,
        Err(error) => {
            println!("Failed to decode preview frame: {:?}", error);
            return;
        }
    };

    match decoded_frame.save(output_path) {
        Ok(()) => println!("Saved camera preview to {} ({}x{}).", output_path, width, height),
        Err(error) => println!("Failed to save preview frame: {:?}", error),
    }
}

fn fallback_detect_qr_code_loop(camera: &mut Camera, scanner: &mut Scanner) -> Vec<String> {
    let mut qr_code_data: Vec<String> = Vec::new();
    while qr_code_data.is_empty() {
        let frame = camera.frame().expect("Failed to capture frame.");
        let decoded_frame = frame.decode_image::<LumaFormat>().expect("Failed to decode frame.");

        let width = frame.resolution().width();
        let height = frame.resolution().height();

        let mut img = zedbar::Image
            ::from_gray(&decoded_frame, width, height)
            .expect("Failed to create zedbar image.");

        let symbols = scanner.scan(&mut img);

        for symbol in symbols {
            let data = symbol.data_string().unwrap_or("");
            if !data.is_empty() {
                qr_code_data.push(data.to_string());
                println!("Data found: {}", data);
            }
        }
    }
    println!("QR Code Data: {:?}", qr_code_data);
    qr_code_data
}

fn warm_up_camera(camera: &mut Camera) {
    for _ in 0..5 {
        let _ = camera.frame();
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)] // generates some common behavior for this type
enum WifiSecurity {
    Wpa,
    Wep,
    Open,
}

#[derive(Debug)]
enum WifiConnectError {
    MissingPassword,
    UnsupportedSecurity,
    CommandFailed(String),
    Io(std::io::Error),
    NoCredentialsFound,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WifiCredentials {
    security_type: WifiSecurity,
    ssid: String,
    password: Option<String>,
}

fn parse_wifi_credentials(qr_payloads: &[String]) -> Option<WifiCredentials> {
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

fn connect_to_wifi_from_qr_payloads(
    qr_payloads: &[String],
    connector: &impl WifiConnector
) -> Result<(), WifiConnectError> {
    // &impl WifiConnector means give me a reference to any type that implements
    // WifiConnector (abstracting away what exact connector is being passed in)
    let credentials = parse_wifi_credentials(qr_payloads).ok_or(
        WifiConnectError::NoCredentialsFound
    )?;
    connector.connect(&credentials)
    // ? means if it is an error, return the error from the curr. function
    // immediately; works for Result or Option; can only use it if this function
    // itself returns a valid Result or Option because there's always a chance
    // it needs to return an error

    // .ok_or converts option to result
}

trait WifiConnector {
    fn connect(&self, credentials: &WifiCredentials) -> Result<(), WifiConnectError>;
}
// trait = promise of behavior; any type that wants to be called WifiConnector
// must implement connect function; also could be like a shared interface of
// sorts where you could have a bunch of different connectors or functions that
// all have connect, so this shared interface makes it easier traits are used
// because want a high-level connector.connect(), not a massive if-else
// statement two types of impl, one is to add methods directly to a type, other
// is a trait impl, teaching a type how to satisfy a trait

struct LinuxNetworkManagerConnector;
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

                LinuxNetworkManagerConnector::check_output_status(add_output)?; // if returns Ok(()) continue, else return immediately the error from the current function

                let up_output = Command::new("nmcli") // connect the profile just created (activating it)
                    .args(["connection", "up", credentials.ssid.as_str()])
                    .output()?;

                LinuxNetworkManagerConnector::check_output_status(up_output)
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

                LinuxNetworkManagerConnector::check_output_status(add_output)?;

                let up_output = Command::new("nmcli")
                    .args(["connection", "up", credentials.ssid.as_str()])
                    .output()?;

                LinuxNetworkManagerConnector::check_output_status(up_output)
            }
        }
    }
}

impl From<std::io::Error> for WifiConnectError {
    // converting from one to another since connect returns WifiConnectError, so
    // .into() is auto. available then (the conversion ? uses)
    fn from(error: std::io::Error) -> Self {
        WifiConnectError::Io(error)
    }
}
