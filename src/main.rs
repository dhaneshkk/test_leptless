use pdfium_render::prelude::*;
use leptess::LepTess;

use std::fs::File;
use std::io::{Cursor, Write};

use image::{ColorType, ExtendedColorType, ImageEncoder};
use image::codecs::png::PngEncoder;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ---- Init PDFium ----
    let bindings = Pdfium::bind_to_library(
        Pdfium::pdfium_platform_library_name_at_path("./lib"),
    ).or_else(|_| Pdfium::bind_to_system_library())?;
    let pdfium = Pdfium::new(bindings);

    // ---- Load PDF ----
    let doc = pdfium.load_pdf_from_file(
        "/home/ubuntu/leptless/2021.eacl-main.75.pdf",
        None,
    )?;
    let page_count = doc.pages().len();
    println!("Loaded PDF with {} pages", page_count);

    // ---- Open output file once ----
    let mut file = File::create("ocr_output.txt")?;

    // ---- Init Tesseract once ----
    let mut lt = LepTess::new(None, "eng")?;

    // ---- Process each page sequentially ----
    for (index, page) in doc.pages().iter().enumerate() {
        println!("---- Processing Page {} ----", index + 1);

        // Try direct text extraction first
        let extracted_text = page.text().unwrap().all();
        if !extracted_text.trim().is_empty() {
            writeln!(
                file,
                "\n\n===== Page {} (Extracted) =====\n\n{}",
                index + 1,
                extracted_text
            )?;
            println!("---- Text extracted from Page {} ----", index + 1);
            continue;
        }

        // Otherwise, render page (at ~300dpi)
        let rendered = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(2480)
                .set_target_height(3508),
        )?;
        let gray_image = rendered.as_image().to_luma8();

        // Encode grayscale as PNG in memory for OCR
        let mut png_data: Vec<u8> = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_data);
            let encoder = PngEncoder::new(&mut cursor);
            encoder.write_image(
                &gray_image,
                gray_image.width(),
                gray_image.height(),
                ExtendedColorType::from(ColorType::L8),
            )?;
        }

        // OCR
        lt.set_image_from_mem(&png_data)?;
        let ocr_text = lt.get_utf8_text()?;

        // Write OCR result
        writeln!(
            file,
            "\n\n===== Page {} (OCR) =====\n\n{}",
            index + 1,
            ocr_text
        )?;
        println!("---- OCR complete for Page {} ----", index + 1);
    }

    println!("\n✅ OCR complete. Results saved to ocr_output.txt");
    Ok(())
}
