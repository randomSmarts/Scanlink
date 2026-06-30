use nokhwa::{ Camera, pixel_format::LumaFormat };

use zedbar::config::*; // Bring everything public from zedbar::config into scope
use zedbar::{ DecoderConfig, Scanner };

pub fn build_qr_scanner() -> Scanner {
    let config = DecoderConfig::new()
        .enable(QrCode)
        .test_inverted(true)
        .retry_undecoded_regions(true)
        .scan_density(2, 2);

    Scanner::with_config(config)
}

pub fn fallback_detect_qr_code_loop(camera: &mut Camera, scanner: &mut Scanner) -> Vec<String> {
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
