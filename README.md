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

## Wi-Fi Connect

### WPA Style:

- `WIFI:T:WPA;S:MyNetworkName;P:MyPassword;;`
- Solution: Can do basic string parsing to read data --> connect to Wi-Fi

### Open Wi-Fi w/Captive Portal

* `WIFI:S:Venue_Free_WiFi;T:nopass;P:;;`
* Connects device to open network, but internet access is blocked until a web/captive portal is completed
* If robot reboots / Wi-Fi lease expires, device will be stuck w/o actual internet
* Solution: Whitelist robot's MAC address beforehand so no captive portals arrive

### Standard URL

* QR code points to standard website
* Read instructions as to how to connect to Wi-Fi
* Main issue here

### WPA-Enterprise / Enrollment URL

* Every single device gets its own unique, temporary credentials/digital certificate
* URL takes the device's browser to a secrue provisioning server
* Server looks at the device and generates a customized network profile (config. file w/security certificates and specific enterprise Wi-Fi settings)
* For unattended devcies can connect beforehand using Mobile Device Management (MDM) software

### Peplink

* Check if router has dual-band, simultaneous Wi-Fi radios
* Local AP connects all devices to router
* Only need to point router to a Wi-Fi WAN
* When connecting to a Wi-Fi with a captive portal, either use the `Captive Portal Sign-On` feature
  * Or connect a device to the Peplink local AP, try to go to the venue's website / any website, venue's system should intercept it and serve the Captive Portal splash page on the device itself, once you are online, the venue's system associates the confirmation with the Peplink’s MAC address and so now every device connected to your Peplink has full internet access
  * For WPA-Enterprise, change the security type, enter the Username and Password, and if need be connect a device to local AP and visit the required website to get the unique 802.1X wireless username and password credentials
