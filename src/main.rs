use anyhow::Result;
use image::{ImageBuffer, Luma};
use leptess::LepTess;
use pdfium_render::prelude::*; // Pdfium rendering
use std::path::Path;

fn main() -> Result<()> {
    let bindings = Pdfium::bind_to_system_library()?;
    let pdfium = Pdfium::new(bindings);
    // Bind to system-installed PDFium

    let doc = pdfium.load_pdf_from_file("/home/ubuntu/leptless/2021.eacl-main.75.pdf", None)?;

    // Initialize Tesseract
    let mut lt = LepTess::new(None, "eng")?;

    // Loop through all pages
    for (i, page) in doc.pages().iter().enumerate() {
        // Render with auto-rotation
        let rendered = page
            .render_with_config(
                &PdfRenderConfig::new()
                    .set_target_width(2480)   // A4 ~300dpi
                    .set_maximum_height(3508) // A4 ~300dpi
                    .rotate_if_landscape(PdfPageRenderRotation::None, true),
            )?
            .as_image();

        // Convert to grayscale
        let gray_image: ImageBuffer<Luma<u8>, Vec<u8>> = rendered.to_luma8();

        // Save for debugging
        let filename = format!("page_{}.png", i + 1);
        gray_image.save(&filename)?;
        println!("Saved rendered page {}", filename);

        // OCR
        lt.set_image(&Path::new(&filename))?;
        let text = lt.get_utf8_text()?;
        println!("===== OCR Page {} =====", i + 1);
        println!("{}", text);
    }

    Ok(())
}
