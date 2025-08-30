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
    let page_count = doc.pages().len() as usize;
    println!("Loaded PDF with {} pages", page_count);

    // ---- Open output file once ----
    let mut file = File::create("ocr_output.txt")?;

    // ---- Process each page sequentially (low RAM) ----
    for (index, page) in doc.pages().iter().enumerate() {
        println!("---- Rendering Page {} ----", index + 1);

        // Render at ~300dpi
        let rendered = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(2480)
                .set_target_height(3508),
        )?;
        let rgb_image = rendered.as_image().to_rgb8();

        // Encode to PNG in memory (only for OCR, not saved)
        let mut png_data: Vec<u8> = Vec::new();
        {
            let mut cursor = Cursor::new(&mut png_data);
            let encoder = PngEncoder::new(&mut cursor);
            encoder.write_image(
                &rgb_image,
                rgb_image.width(),
                rgb_image.height(),
                ExtendedColorType::from(ColorType::Rgb8),
            )?;
        }

        // OCR
        let mut lt = LepTess::new(None, "eng")?;
        lt.set_image_from_mem(&png_data)?;
        let text = lt.get_utf8_text()?;

        // Write directly to disk (free RAM immediately after)
        writeln!(file, "\n\n===== Page {} =====\n\n{}", index + 1, text)?;
        println!("---- OCR Page {} complete and written ----", index + 1);
    }

    println!("\n✅ OCR complete. Results saved to ocr_output.txt");
    Ok(())
}
