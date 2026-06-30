use nokhwa::{
    Camera,
    pixel_format::{ LumaFormat },
    utils::{ CameraIndex, RequestedFormat },
};

fn warm_up_camera(camera: &mut Camera) {
    for _ in 0..5 {
        let _ = camera.frame();
    }
}

pub fn setup_camera(index: CameraIndex, requested: RequestedFormat) -> Camera {
    let mut camera = Camera::new(index, requested).expect("Failed to create camera"); // mut because opening/using the camera changes its internal state
    camera.open_stream().expect("Failed to open camera stream."); // starts the camera, changes internal state; use expect because it returns a Result

    warm_up_camera(&mut camera); // removing unnecessary flame flushing from hot path because camera acquisition latency dominates QR decode latency
    camera
}

pub fn save_camera_preview(camera: &mut Camera) {
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
