use std::io;

use nokhwa::{
    pixel_format::LumaFormat,
    utils::{ CameraIndex, RequestedFormat, RequestedFormatType },
};

use scanlink::{
    build_qr_scanner,
    connect_to_wifi_from_qr_payloads,
    save_camera_preview,
    scan_qr_payloads_from_camera,
    setup_camera,
    LinuxNetworkManagerConnector,
};
fn main() {
    let mut user_input = String::new();

    let index = CameraIndex::Index(0);

    let requested = RequestedFormat::new::<LumaFormat>(
        RequestedFormatType::AbsoluteHighestFrameRate
    );

    let mut camera = setup_camera(index, requested);

    let mut scanner = build_qr_scanner();

    loop {
        user_input.clear();

        println!("Type command: p = preview, c = capture, q = quit");

        io::stdin().read_line(&mut user_input).expect("failed to read input");

        let command = user_input.trim().to_lowercase();

        if command == "q" {
            println!("Thank you for using this service. Good bye.");
            break;
        } else if command == "p" {
            save_camera_preview(&mut camera);
        } else if command == "c" {
            let qr_payloads = scan_qr_payloads_from_camera(&mut camera, &mut scanner);

            println!("QR payload detected.");

            let connector = LinuxNetworkManagerConnector;
            let result = connect_to_wifi_from_qr_payloads(&qr_payloads, &connector);

            match result {
                Ok(()) => {
                    println!("Connected to Wi-Fi successfully.");
                }

                Err(error) => {
                    println!("Failed to connect to Wi-Fi: {:?}", error);
                }
            }
        } else {
            println!("Unknown command.");
        }
    }
}
