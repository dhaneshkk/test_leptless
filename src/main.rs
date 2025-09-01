use pdfium_render::prelude::*;
use tesseract::Tesseract;
use std::io::Cursor;
use image::DynamicImage;
fn calculate_column_ratio(page: &PdfPage) -> Result<f32, PdfiumError> {
    // 1. Get the total width of the page in PDF "user space" points.
    let page_width = page.width().value;

    // 2. Access the text content of the page.
    let text_page = page.text()?;

    // 3. Initialize min and max horizontal coordinates.
    //    We start with min_x at the largest possible value and max_x at the
    //    smallest so that the first character's bounds will immediately replace them.
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;

    // 4. Iterate over every character on the page to find the text block's extent.
    for character in text_page.chars().iter(){
        // Get the tight bounding box for the character.
        if let Ok(bounds) = character.tight_bounds() {
            // Update the minimum x-coordinate found so far.
            min_x = min_x.min(bounds.left().value);

            // Update the maximum x-coordinate by finding the right edge of the character.
            let right_edge = bounds.left() + bounds.width();
            max_x = max_x.max(right_edge.value);
        }
    }

    // 5. Calculate the column ratio.
    //    If min_x is still f32::MAX, it means no characters were found on the page.
    //    In this case, the text width is zero, so the ratio should also be zero.
    let column_ratio = if min_x <= max_x {
        (max_x - min_x) / page_width
    } else {
        0.0
    };

    Ok(column_ratio)
}
fn main() -> anyhow::Result<()> {
    let bindings = Pdfium::bind_to_system_library()?;
    let pdfium = Pdfium::new(bindings);

    let doc = pdfium.load_pdf_from_file("/home/ubuntu/leptless/2021.eacl-main.75.pdf", None)?;

    for (i, page) in doc.pages().iter().enumerate() {
        // Render page to high-resolution bitmap
        let bitmap = page.render_with_config(
            &PdfRenderConfig::new()
                .set_target_width(2000)
                .set_maximum_height(2000)
                .rotate_if_landscape(PdfPageRenderRotation::None, false)
                .render_form_data(true),
        )?;

        let dyn_img: DynamicImage = bitmap.as_image();

        let mut buf = Vec::new();
        dyn_img.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        // Analyze page layout to decide PSM
        let text_page = page.text()?;
        let column_ratio = calculate_column_ratio(&page)?;
        println!("--- Page {} ---", i + 1);
        println!("Page Width: {:.2}", page.width());
        println!("Column Ratio: {:.2}", column_ratio); // e.g., 0.85 for a full-width page


        // --- CORRECTED SECTION ---
        // Create a new Tesseract instance per page and make it mutable
        let mut tess = Tesseract::new(None, Some("eng"))?;
        println!("--- column_ratio {} ---", column_ratio);
        // Set the PSM in-place
        if column_ratio > 0.8 {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmSingleBlock);
        } else {
            tess.set_page_seg_mode(tesseract::PageSegMode::PsmAutoOsd);
        }

        // Now, set the image and get the text sequentially
        let mut tess = tess.set_image_from_mem(&buf)?;
        let text = tess.get_text()?;
        // --- END CORRECTED SECTION ---

        println!("--- Page {} ---", i + 1);
        println!("{}", text);
    }

    Ok(())
}