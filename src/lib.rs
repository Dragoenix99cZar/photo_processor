use image::{DynamicImage, GenericImageView, ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ImageProcessor {
    original_image: DynamicImage,
    offset_x: i32,
    offset_y: i32,
    zoom_factor: f32,
    rotation_angle: i32, // Track rotation states: 0, 90, 180, 270
}

#[wasm_bindgen]
impl ImageProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new(raw_bytes: &[u8]) -> Result<ImageProcessor, JsValue> {
        console_error_panic_hook::set_once();

        let img = image::load_from_memory(raw_bytes)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse image bytes: {}", e)))?;

        Ok(ImageProcessor {
            original_image: img,
            offset_x: 0,
            offset_y: 0,
            zoom_factor: 1.0,
            rotation_angle: 0,
        })
    }

    pub fn pan_image(&mut self, dx: i32, dy: i32) {
        // Adapt movement vectors contextually matching current view rotations
        match self.rotation_angle {
            90 => {
                self.offset_x += dy;
                self.offset_y -= dx;
            }
            180 => {
                self.offset_x -= dx;
                self.offset_y -= dy;
            }
            270 => {
                self.offset_x -= dy;
                self.offset_y += dx;
            }
            _ => {
                self.offset_x += dx;
                self.offset_y += dy;
            }
        }
    }

    pub fn zoom_image(&mut self, factor_delta: f32) {
        self.zoom_factor += factor_delta;
        if self.zoom_factor < 0.05 {
            self.zoom_factor = 0.05;
        }
    }

    // Call this incrementally from JS to spin the framework layout clockwise
    pub fn rotate_clockwise(&mut self) {
        self.rotation_angle = (self.rotation_angle + 90) % 360;
    }

    pub fn get_position(&self) -> Vec<i32> {
        vec![self.offset_x, self.offset_y]
    }

    pub fn get_zoom(&self) -> f32 {
        self.zoom_factor
    }

    pub fn get_rotation(&self) -> i32 {
        self.rotation_angle
    }

    pub fn export_at_quality(
        &self,
        format_str: &str,
        target_w: u32,
        target_h: u32,
        quality: u8,
    ) -> Result<Vec<u8>, JsValue> {
        // Pre-rotate the core reference canvas layout before cutting clipping matrix windows out
        let rotated_base = match self.rotation_angle {
            90 => self.original_image.rotate90(),
            180 => self.original_image.rotate180(),
            270 => self.original_image.rotate270(),
            _ => self.original_image.clone(),
        };

        let (orig_w, orig_h) = rotated_base.dimensions();
        let view_size = 600.0;
        let scale = self.zoom_factor;

        let crop_w = (view_size / scale) as i32;
        let crop_h = crop_w;

        let src_x =
            ((orig_w as f32 / 2.0) - (self.offset_x as f32 / scale) - (crop_w as f32 / 2.0)) as i32;
        let src_y =
            ((orig_h as f32 / 2.0) - (self.offset_y as f32 / scale) - (crop_h as f32 / 2.0)) as i32;

        let x_start = src_x.max(0) as u32;
        let y_start = src_y.max(0) as u32;
        let x_end = (src_x + crop_w).min(orig_w as i32) as u32;
        let y_end = (src_y + crop_h).min(orig_h as i32) as u32;

        if x_end <= x_start || y_end <= y_start {
            return Err(JsValue::from_str(
                "Transform crop target area lies completely outside source boundaries.",
            ));
        }

        let intersect_w = x_end - x_start;
        let intersect_h = y_end - y_start;

        let cropped_slice = rotated_base.crop_imm(x_start, y_start, intersect_w, intersect_h);

        let res_ratio = target_w as f32 / view_size;
        let final_slice_w = ((intersect_w as f32 * scale) * res_ratio).round() as u32;
        let final_slice_h = ((intersect_h as f32 * scale) * res_ratio).round() as u32;

        let resized_slice = cropped_slice.resize_exact(
            final_slice_w.max(1),
            final_slice_h.max(1),
            image::imageops::FilterType::Lanczos3,
        );

        let is_jpeg = format_str == "jpeg";
        let bg_color = if is_jpeg {
            Rgba([255, 255, 255, 255])
        } else {
            Rgba([0, 0, 0, 0])
        };
        let mut final_canvas = ImageBuffer::from_pixel(target_w, target_h, bg_color);

        let mut dest_x = 0;
        let mut dest_y = 0;

        if src_x < 0 {
            dest_x = ((-src_x as f32 * scale) * res_ratio).round() as u32;
        }
        if src_y < 0 {
            dest_y = ((-src_y as f32 * scale) * res_ratio).round() as u32;
        }

        let final_w = final_slice_w.min(target_w - dest_x.min(target_w - 1));
        let final_h = final_slice_h.min(target_h - dest_y.min(target_h - 1));

        for y in 0..final_h {
            for x in 0..final_w {
                if x < resized_slice.width() && y < resized_slice.height() {
                    let pixel = resized_slice.get_pixel(x, y);
                    if is_jpeg && pixel[3] < 255 {
                        let alpha = pixel[3] as f32 / 255.0;
                        let r = ((pixel[0] as f32 * alpha) + (255.0 * (1.0 - alpha))) as u8;
                        let g = ((pixel[1] as f32 * alpha) + (255.0 * (1.0 - alpha))) as u8;
                        let b = ((pixel[2] as f32 * alpha) + (255.0 * (1.0 - alpha))) as u8;
                        final_canvas.put_pixel(dest_x + x, dest_y + y, Rgba([r, g, b, 255]));
                    } else {
                        final_canvas.put_pixel(dest_x + x, dest_y + y, pixel);
                    }
                }
            }
        }

        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);
        let dynamic_canvas = DynamicImage::ImageRgba8(final_canvas);

        let format = match format_str {
            "png" | "webp" => ImageFormat::Png,
            _ => ImageFormat::Jpeg,
        };

        if format == ImageFormat::Jpeg {
            let mut encoder =
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
            encoder
                .encode_image(&dynamic_canvas)
                .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        } else {
            dynamic_canvas
                .write_to(&mut cursor, format)
                .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        }

        Ok(buffer)
    }
}
