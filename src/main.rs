use pdfium_render::prelude::*;
use tesseract::Tesseract;
use zspell::Dictionary;
 // new spelling library
use image::{DynamicImage, Luma, ImageBuffer};
use std::io::Cursor;
use std::fs;
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

    let column_ratio = if min_x <= max_x {
        (max_x - min_x) / page_width
    } else {
        0.0
    };

    Ok(column_ratio)
}

fn binarize_image(img: &DynamicImage) -> ImageBuffer<Luma<u8>, Vec<u8>> {
    let gray = img.to_luma8();
    let mut binarized = gray.clone();
    let threshold = 128u8;

    for (x, y, pixel) in gray.enumerate_pixels() {
        let Luma([l]) = *pixel;
        let p = if l > threshold { 255 } else { 0 };
        binarized.put_pixel(x, y, Luma([p]));
    }

    binarized
}

fn main() -> anyhow::Result<()> {
    // Initialize Pdfium
    let bindings = Pdfium::bind_to_system_library()?;
    let pdfium = Pdfium::new(bindings);

    // Load PDF
    let doc = pdfium.load_pdf_from_file("/home/ubuntu/leptless/2021.eacl-main.75.pdf", None)?;

    // Initialize zspell dictionary (English)
   // This example just uses some shortened files. Load them to a string
    let aff_content =
        fs::read_to_string("index.aff").expect("failed to load config file");
    let dic_content =
        fs::read_to_string("index.dic").expect("failed to load wordlist file");

    // Use the builder pattern to create our `Dictionary` object
    let dict: Dictionary = zspell::builder()
        .config_str(&aff_content)
        .dict_str(&dic_content)
        .build()
        .expect("failed to build dictionary!");

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

        // Adaptive binarization
        let bin_img = binarize_image(&dyn_img);
        let mut buf = Vec::new();
        DynamicImage::ImageLuma8(bin_img).write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        // Calculate column ratio
        let column_ratio = calculate_column_ratio(&page)?;
        println!("--- Page {} ---", i + 1);
        println!("Column Ratio: {:.2}", column_ratio);

        // Create new Tesseract per page
        let mut tess = Tesseract::new(None, Some("eng"))?;

        // Set PSM based on column layout
        if column_ratio > 0.8 {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmSingleBlock);
        } else {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmAutoOsd);
        }

        // Run OCR
        tess= tess.set_image_from_mem(&buf)?;
        let raw_text = tess.get_text()?;

        // Filter words using zspell
        let cleaned_text: Vec<String> = raw_text
            .split_whitespace()
            .filter(|w| dict.check(w))
            .map(|s| s.to_string())
            .collect();
        let text = cleaned_text.join(" ");

        println!("{}", text);
    }

    Ok(())
}
