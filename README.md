# photo_processor

## Photo Alignment, Crop & Compress Tool
A high-performance, client-side image alignment, cropping, and compression engine built with Rust and WebAssembly (wasm-bindgen). This tool provides near-instantaneous canvas manipulations for high-stakes identification document photo compliance.

---

## Key Technical Highlights

* WASM Acceleration: Image transformations, resampling, and pixel matrices are computed in compiled Rust, offloading intensive graphics operations from the single-threaded JavaScript main loop.
* Context-Aware Panning: Movement vector translations map dynamically to arbitrary canvas orientations. Toggling between 0°, 90°, 180°, and 270° orientations preserves absolute canvas-relative tracking during drag operations.
* Pixel-Perfect Adjustments: Built for micro-step keyboard alignment down to individual physical pixels.
* Advanced Color Processing: Automatic hardware alpha-channel blending to white backgrounds for strict legal JPEG specs.
* Lanczos3 Filtering: Uses premium Lanczos3 resampling to retain sharp high-fidelity facial biometrics during resolution downscaling.

------------------------------
## Structural Overview
The core image processing layer lives inside src/lib.rs and exposes the ImageProcessor interface to JavaScript applications:
```rust
pub struct ImageProcessor {
    original_image: DynamicImage,
    offset_x: i32,
    offset_y: i32,
    zoom_factor: f32,
    rotation_angle: i32,
}
```
## Core API Methods

* ```ImageProcessor::new(raw_bytes: &[u8]) -> Result<ImageProcessor, JsValue>```: Decodes raw binary file buffers directly into a stateful Rust instance.
* ```pan_image(dx: i32, dy: i32)```: Applies directional transforms modified by current rotation angles.
* ```zoom_image(factor_delta: f32)```: Alters scale factor using absolute relative zoom parameters.
* ```rotate_clockwise()```: Shifts structural output target frames incrementally by 90°.
* ```export_at_quality(format_str: &str, target_w: u32, target_h: u32, quality: u8) -> Result<Vec<u8>, JsValue>```: Slices matching coordinate bounds and generates compressed binary assets.

------------------------------
## Building and Compilation## Prerequisites
Ensure you have the Rust toolchain and wasm-pack installed:

```console
curl --proto '=https' --tlsv1.2 -sSf https://rustup.rs | sh
cargo install wasm-pack
```

## Compilation
Build the WebAssembly package for modern browser bundlers (Webpack, Vite, Rollup):

```console
wasm-pack build --target bundler
```

For direct usage as a classic ES module without a build step:

```console
wasm-pack build --target web
```

------------------------------
## 📦 JavaScript Integration Example

```js
import init, { ImageProcessor } from './pkg/passport_photo_processor.js';
async function run() {
    await init();
    
    const response = await fetch('user_upload.jpg');
    const arrayBuffer = await response.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);
    
    // Initialize the engine
    const processor = new ImageProcessor(bytes);
    
    // Perform spatial transformations based on UI interactions
    processor.zoom_image(0.25);
    processor.pan_image(15, -30);
    processor.rotate_clockwise();
    
    // Export strict US Visa compliant square JPEG bytes
    const outputBytes = processor.export_at_quality("jpeg", 600, 600, 85);
    
    // Bind output bytes straight into a downloadable UI Blob
    const blob = new Blob([outputBytes.buffer], { type: "image/jpeg" });
    const url = URL.createObjectURL(blob);
    document.getElementById("preview").src = url;
}

run();
```

------------------------------
## License
