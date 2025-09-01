use pdfium_render::prelude::*;
use tesseract::Tesseract;
use image;
use std::io::Cursor;

fn main() -> anyhow::Result<()> {
    // Initialize Pdfium (bind to system libpdfium)
    let bindings = Pdfium::bind_to_system_library()?;
    let pdfium = Pdfium::new(bindings);

    // Load your PDF
    let doc = pdfium.load_pdf_from_file("/home/ubuntu/leptless/2021.eacl-main.75.pdf", None)?;

    // Initialize Tesseract (English, assumes tessdata installed)
    let mut tess = Tesseract::new(None, Some("eng"))?;
    // Set PSM mode for multi-column handling
    // Options: 1 (OSD+auto), 3 (auto), 4 (single col), 6 (single block), 11 (sparse), 12 (sparse+OSD)
    tess = tess.set_variable("tessedit_pageseg_mode", "1")?;
    // Process each page
    for (i, page) in doc.pages().iter().enumerate() {
        let render = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(2000) // high DPI for OCR quality
                .set_maximum_height(2000)
                .rotate_if_landscape(PdfPageRenderRotation::None,false)
                .render_form_data(true),
        );

        // render_with_config returns Result<PdfBitmap, PdfiumError>
        let bitmap = render?;

        // Convert Pdfium bitmap → DynamicImage
        let dyn_img = bitmap.as_image();

        // Encode image as PNG in memory
        let mut buf = Vec::new();
        dyn_img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        // OCR the image
        tess = tess.set_image_from_mem(&buf)?;
        let text = tess.get_text()?;

        println!("--- Page {} ---", i + 1);
        println!("{}", text);
    }

    Ok(())
}
