use pdfium_render::prelude::*;
use tesseract::Tesseract;
use zspell::Dictionary;
use image::{DynamicImage, Luma, ImageBuffer, imageops};
use std::io::Cursor;
use std::fs;
use image::GenericImageView;
use  image::GenericImage;
use image::Pixel;
/// Calculate horizontal text spread on the page
fn calculate_column_ratio(page: &PdfPage) -> Result<f32, PdfiumError> {
    let page_width = page.width().value;
    let text_page = page.text()?;

    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;

    for ch in text_page.chars().iter() {
        if let Ok(bounds) = ch.tight_bounds() {
            let left = bounds.left().value;
            let right = left + bounds.width().value;
            min_x = min_x.min(left);
            max_x = max_x.max(right);
        }
    }

    if min_x <= max_x {
        Ok((max_x - min_x) / page_width)
    } else {
        Ok(0.0)
    }
}

/// Binarize image with a given threshold
fn binarize_image(img: &DynamicImage, threshold: u8) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let gray = img.to_luma8();
    let mut binarized = gray.clone();

    for (x, y, pixel) in gray.enumerate_pixels() {
        let Luma([l]) = *pixel;
        let p = if l > threshold { 255 } else { 0 };
        binarized.put_pixel(x, y, Luma([p]));
    }

    binarized
}

/// Enhance image by contrast adjustment and slight sharpening



fn enhance_image(img: &DynamicImage) -> DynamicImage {
    let mut enhanced = img.clone();

    // Increase contrast
    enhanced = image::DynamicImage::ImageRgba8(imageops::contrast(&enhanced, 20.0));

    // Slight sharpening via unsharp mask
    let blurred = imageops::blur(&enhanced, 1.0);
    let (width, height) = enhanced.dimensions();

    let mut sharpened = enhanced.clone();
    for y in 0..height {
        for x in 0..width {
            let orig_pixel = enhanced.get_pixel(x, y);
            let blur_pixel = blurred.get_pixel(x, y);

            let new_pixel = orig_pixel.map2(&blur_pixel, |o, b| {
                let val = o.saturating_add(o.saturating_sub(b)); // simple sharpening
                val
            });

            sharpened.put_pixel(x, y, new_pixel);
        }
    }

    sharpened
}


/// Perform OCR with adaptive thresholding and optional image enhancement
fn ocr_with_retry(
    img: &DynamicImage,
    dict: &Dictionary,
    column_ratio: f32,
) -> anyhow::Result<String> {
    // Try thresholds first
    let thresholds = [180, 150, 128, 100, 70];

    for &th in &thresholds {
        let bin_img = binarize_image(img, th);
        let mut buf = Vec::new();
        DynamicImage::ImageLuma8(bin_img).write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        let mut tess = Tesseract::new(None, Some("eng"))?;
        if column_ratio > 0.8 {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmSingleBlock);
        } else {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmAutoOsd);
        }

        tess = tess.set_image_from_mem(&buf)?;
        let raw_text = tess.get_text()?;

        let valid_words: Vec<String> = raw_text
            .split_whitespace()
            .filter(|w| dict.check(w))
            .map(|s| s.to_string())
            .collect();

        if valid_words.len() >= 5 {
            return Ok(valid_words.join(" "));
        }
    }

    // Retry with enhanced image if thresholds failed
    let enhanced_img = enhance_image(img);
    for &th in &thresholds {
        let bin_img = binarize_image(&enhanced_img, th);
        let mut buf = Vec::new();
        DynamicImage::ImageLuma8(bin_img).write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        let mut tess = Tesseract::new(None, Some("eng"))?;
        if column_ratio > 0.8 {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmSingleBlock);
        } else {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmAutoOsd);
        }

        tess = tess.set_image_from_mem(&buf)?;
        let raw_text = tess.get_text()?;

        let valid_words: Vec<String> = raw_text
            .split_whitespace()
            .filter(|w| dict.check(w))
            .map(|s| s.to_string())
            .collect();

        if valid_words.len() >= 5 {
            return Ok(valid_words.join(" "));
        }
    }

    Ok(String::from("[OCR failed: insufficient valid words]"))
}

fn main() -> anyhow::Result<()> {
    // Initialize Pdfium
    let bindings = Pdfium::bind_to_system_library()?;
    let pdfium = Pdfium::new(bindings);

    // Load PDF
    let doc = pdfium.load_pdf_from_file("/home/ubuntu/leptless/2021.eacl-main.75.pdf", None)?;

    // Load zspell dictionary
    let aff_content = fs::read_to_string("index.aff")?;
    let dic_content = fs::read_to_string("index.dic")?;
    let dict: Dictionary = zspell::builder()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()?;

    for (i, page) in doc.pages().iter().enumerate() {
        // Render high-res bitmap
        let bitmap = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(2000)
                .set_maximum_height(2000)
                .rotate_if_landscape(PdfPageRenderRotation::None, false)
                .render_form_data(true),
        )?;
        let dyn_img: DynamicImage = bitmap.as_image();

        // Calculate column ratio
        let column_ratio = calculate_column_ratio(&page)?;
        println!("--- Page {} ---", i + 1);
        println!("Column Ratio: {:.2}", column_ratio);

        // OCR with retries and enhancement
        let text = ocr_with_retry(&dyn_img, &dict, column_ratio)?;
        println!("{}", text);
    }

    Ok(())
}
