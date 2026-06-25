use std::{ io };

use nokhwa::{ // using a crate, and bringing these names into scope, so I can use Camera instead of nokhwa::Camera
    Camera,
    pixel_format::RgbFormat,
    utils::{ CameraIndex, RequestedFormat, RequestedFormatType }, // importing Certain stuff inside nokhwa::utils
};

use image::{ DynamicImage };

use zedbar::config::*; // Bring everything public from zedbar::config into scope
use zedbar::{ DecoderConfig, Scanner };

fn main() {
    let mut user_input: String = String::new();

    let index = CameraIndex::Index(0); // CameraIndex is a type, and so using CameraIndex::Index(0) means ot use camera number 0

    let requested = RequestedFormat::new::<RgbFormat>(
        // creates a camera format request, uses turbofish syntax telling rust call new and use RgbFormat as the format type with highest frame rate
        RequestedFormatType::AbsoluteHighestFrameRate
    );

    let mut camera = Camera::new(index, requested) // mut because opening/using the camera changes its internal state
        .expect("Failed to create camera."); // returns Result, if Camera::new worked give me Camera else give me error message

    camera.open_stream().expect("Failed to open camera stream."); // starts the camera, changes internal state; use expect because it returns a Result

    loop {
        // loop can return a value to a var; runs ≥ 1
        user_input.clear();

        println!("Type command: c = capture, q = quit");

        io::stdin()
            .read_line(&mut user_input) // mutable reference to fill in user_input since it's a String buffer
            .expect("failed to read input"); // expect says it should succeed, if it doesn't crash program and print this message; unwraps the Result and if Ok(...) then returns inside value of how many bytes it was read (which we don't care about); if Ok(...) fills in String buffer

        let command = user_input.trim().to_lowercase(); // when I press enter in terminal, it appens \n so need to get rid of it, and converting to lowercase

        if command == "q" {
            println!("Thank you for using this service. Good bye.");
            break;
        } else if command == "c" {
            println!("The data in the QR code is: {}", detect_qr_code_loop(&mut camera));
        }
    }
}

fn detect_qr_code_loop(camera: &mut Camera) -> String {
    loop {
        for _ in 0..30 {
            let _ = camera.frame();
        }

        let frame = camera.frame().expect("Failed to capture frame.");
        let decoded_frame = frame.decode_image::<RgbFormat>().expect("Failed to decode frame.");

        let width = frame.resolution().width();
        let height = frame.resolution().height();

        let gray_img = DynamicImage::ImageRgb8(decoded_frame).to_luma8();

        let mut img = zedbar::Image
            ::from_gray(&gray_img, width, height)
            .expect("Failed to create zedbar image.");

        let config = DecoderConfig::new() // settings object
            .enable(QrCode) // tell scanner to look for QR codes
            .enable(Ean13) // Also look for EAN-13 barcodes (common on retail items)
            .test_inverted(true) // if normal scan finds nothing, try scanning an inverted image (white squares on black background)
            .retry_undecoded_regions(true) // If zedbar sees QR finder patterns but can't decode the QR, auto crop/upscale the sus regions and try again
            .scan_density(2, 2); // density control, about scanning every x rows and every y columns

        let mut scanner = Scanner::with_config(config);

        let symbols = scanner.scan(&mut img);

        for symbol in symbols {
            let data = symbol.data_string().unwrap_or("");
            if !data.is_empty() {
                return data.to_string();
            }
        }
    }
}
