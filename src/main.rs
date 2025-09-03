use pdfium_render::prelude::*;
use tesseract::Tesseract;
use zspell::Dictionary;
use image::{DynamicImage, Luma, ImageBuffer, imageops};
use std::io::Cursor;
use std::fs;

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



// --- COMPLETELY REWRITTEN AND CORRECTED FUNCTION ---
/// Enhance image by increasing contrast and applying a sharpening filter.
/// This function is made type-safe by converting to RGBA8 format first.
fn enhance_image(img: &DynamicImage) -> DynamicImage {
    // Convert to RGBA to work with a consistent format for imageops
    let mut rgba_img = img.to_rgba8();

    // Increase contrast. The imageops::contrast function returns an ImageBuffer.
    rgba_img = imageops::contrast(&rgba_img, 15.0);

    // Sharpening (unsharp mask). This is often more effective than a simple loop.
    // Negative values sharpen, positive values blur.
    let sharpened_buffer = imageops::unsharpen(&rgba_img, 5.0, 10);

    // Convert the final ImageBuffer back to a DynamicImage
    DynamicImage::ImageRgba8(sharpened_buffer)
}



/// Perform OCR with adaptive thresholding and optional image enhancement
fn ocr_with_retry(
    img: &DynamicImage,
    dict: &Dictionary,
    column_ratio: f32,
) -> anyhow::Result<String> {
    let thresholds = [180, 150, 128, 100, 70];
    let min_valid_words = 10; // Set a reasonable minimum number of words
    let perform_ocr_and_filter = |image_buffer: &ImageBuffer<Luma<u8>, Vec<u8>>| -> anyhow::Result<String> {
        let mut buf = Vec::new();
        image_buffer.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        let mut tess = Tesseract::new(None, Some("eng"))?;
        if column_ratio > 0.8 {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmSingleBlock);
        } else {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmAutoOsd);
        }

        let mut tess = tess.set_image_from_mem(&buf)?;
        let raw_text= tess.get_text()?;

    // Step 2: Filter the raw text to keep words AND numbers/dates
    let cleaned_text = raw_text
        .split_whitespace()
        .filter(|word| {
            // First, strip common leading/trailing punctuation for a cleaner check.
            // This handles cases like "(word)" or "number,".
            let cleaned_word = word.trim_matches(|c: char| !c.is_alphanumeric());

            // Ignore empty strings that might result from stripping (e.g., a standalone "--").
            if cleaned_word.is_empty() {
                return false;
            }

            // Condition 1: The word is in the dictionary.
            let is_in_dict = dict.check(cleaned_word);

            // Condition 2: The word is likely a number, date, or code.
            // Heuristic: it contains at least one digit. This is simple and effective.
            let is_numeric_like = cleaned_word.chars().any(|c| c.is_ascii_digit());

            // Keep the original `word` if either condition is true.
            is_in_dict || is_numeric_like
        })
        .collect::<Vec<_>>()
        .join(" ");

    Ok(cleaned_text)
    };

println!("Attempting OCR with multiple thresholds...");


    println!("Thresholding failed. Enhancing image and retrying...");
    let enhanced_img = enhance_image(img);
    for &th in &thresholds {
        let bin_img = binarize_image(&enhanced_img, th);
        let text = perform_ocr_and_filter(&bin_img)?;
        let word_count = text.split_whitespace().count();

        if word_count >= min_valid_words {
            println!("Success with enhanced image and threshold {}.", th);
            return Ok(text);
        }
    }


    Ok(String::from("[OCR failed: insufficient valid words after all attempts]"))
}

fn main() -> anyhow::Result<()> {

    // Initialize Pdfium
    let bindings = Pdfium::bind_to_system_library()?;
    let pdfium = Pdfium::new(bindings);

    // Load PDF
    let doc = pdfium.load_pdf_from_file("/home/ubuntu/leptless/2021.eacl-main.75.pdf", None)?;

    // Load zspell dictionary
    let aff_content = fs::read_to_string("en_US.aff")?;
    let dic_content = fs::read_to_string("en_US.dic")?;
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
