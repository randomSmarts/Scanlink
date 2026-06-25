# Rust-QR-Code-Analyzer

## Image Processing

### Workflow:

1. Take a raw frame from camera stream, results in raw camera bytes buffer
2. Decode it into a Luma object, which internally it is a buffer of brightness bytes, plus width/height/type info
3. Convert general image to zedbar::Image, containing data, width, and height
4. Initialize Scanner with preferenced config
5. Scan QR code
6. Return data of first QR code detected

## General QR Detection Algorithm Steps:

1. Camera stream
2. Convert to grayscale image
3. Edge detection
4. Find QR common finder patterns
5. Estimate perspective/QR position (rotation, scale, tilt, etc.)
6. Sample the modules to get binary (on center of every square)
7. Decode QR format (reads QR version, mask pattern, error correction level, etc.)
8. Remove the mask (scanner removes mask pattern according to QR specification)
9. Reed-Solomon error correction (useful if part of QR code is damaged/missing)
10. Return symbols (type, data)

## Different Color Types and Their Uses:

### Main Color Types:

- L8 / Luma8
  - Grayscale
  - 1 byte per pixel
- La8
  - Grayscale + Alpha
  - 2 bytes per pixel
- Rgb8
  - Red/Green/Blue
  - 3 bytes per pixel
- Rgba8
  - Red/Green/Blue/Alpha
  - 4 bytes per pixel
- L16
  - High-precision grayscale
- La16
  - High-precision grayscale + alpha
- Rgb16
  - High-precision RGB
- Rgba16
  - High-precision RGBA
- Rgb32F
  - RGB using 32-bit floats
- Rgba32F
  - RGBA using 32-bit floats

### Needed Translation:

- L = luminance = brightness only = grayscale
- A = alpha = transparency
- 8 = 8 bits per channel = normal images
- 16 = 16 bits per channel = higher precision
- 32F = 32-bit floating-point per channel = advanced/HDR/science stuff

### Use Cases:

- Rgb8
  - Use for normal camera photos
- Rgba8
  - Use if each pixel needs transparency
- L8
  - Use for grayscale images, one channel (one independent piece of info) per pixel
- Rgb16 / Rgba16
  - Use for high-quality editing or scientific/medical images
- Rgb32F / Rgba32F
  - Usually not needed unless doing advanced image processing
