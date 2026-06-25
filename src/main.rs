use std::io;

use nokhwa::{ // using a crate, and bringing these names into scope, so I can use Camera instead of nokhwa::Camera
    Camera,
    pixel_format::RgbFormat,
    utils::{ CameraIndex, RequestedFormat, RequestedFormatType }, // importing Certain stuff inside nokhwa::utils
};

use image::{ DynamicImage };

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

    let mut auto_exposure: bool = false;

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
            if !auto_exposure {
                println!("Need to conduct auto-exposure, please wait.");
                for _ in 0..30 {
                    let _ = camera.frame();
                }
                auto_exposure = true;
            }

            let frame = camera.frame().expect("Failed to capture frame."); // gives the next image frame
            println!("Captured image, len is {}.", frame.buffer().len()); // size of camera's original data

            let decoded_frame = frame
                .decode_image::<RgbFormat>() // converts frame into RGB pixel data, specifying the desired output
                .expect("Failed to decide frame.");

            println!("Decoded RGB image size in pixels: {}", decoded_frame.len() / 3); // decoded_frame.len() is bytes; dividing by 3 gives pixels because RGB has 3 bytes per pixel

            let width = frame.resolution().width();
            let height = frame.resolution().height();

            println!("Frame height: {}", height);
            println!("Frame width: {}", width);

            image
                ::save_buffer_with_format(
                    "test.png",
                    &decoded_frame,
                    width,
                    height,
                    image::ColorType::Rgb8,
                    image::ImageFormat::Png
                )
                .expect("Failed to save png"); // color type is 3 bytes per pixel, encode as PNG

            println!("Saved image properly, resolution: {}", frame.resolution());

            let gray_img = DynamicImage::ImageRgb8(decoded_frame).to_luma8(); // take the RGB image, wrap it as a DynamicImage, convert it to 8-bit grayscale, and store it in gray_img (converting to brightness pixels)
            // DynamicImage::ImageRgb8 means wrap the RGB image inside a general image enum, and then convert to brightness

            gray_img.save("gray.png").expect("Failed to save grayscale image.");
            

            let mut prepared = rqrr::PreparedImage::prepare(gray_img); // Give rqrr the grayscale image and it'll finish preparing it for QR searching

            let grids = prepared.detect_grids(); // Search the image for QR-code-shaped square patterns

            println!("Scanning grids.");
            println!("Number of QR grids found: {}", grids.len());

            for grid in grids {
                // For each QR-looking thing in the image, there could be many QR codes
                let (_meta, content) = grid.decode().expect("Failed to decode QR."); // Try to decode the QR grid into actual data, returning a result; (_meta, content) means to take the returned pair, put the first item in _meta (means that we know it exists but probably won't use it, don't warn me) and second item in content (decoded QR text)

                println!("QR Content: {}", content);
            }
        }
    }
}
