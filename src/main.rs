use std::path::{Path, PathBuf};
use std::env;
use leptess::LepTess;

use tempfile::TempDir;
use anyhow::{Context, Result};
use rayon::prelude::*;

// 1. Import the necessary items from pdfium-render
use pdfium_render::prelude::*;

/// Converts PDF to PNG images using the pdfium-render crate.
/// Returns a tuple: (TempDir handle, Vec of image paths)
// This is the new, parallelized version of your function.
fn pdf_to_images(pdf_path: &Path) -> Result<(TempDir, Vec<PathBuf>)> {
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./lib"))
            .or_else(|_| Pdfium::bind_to_system_library())?,
    );
    let document = pdfium.load_pdf_from_file(pdf_path, None)
        .context(format!("Failed to load PDF file: {}", pdf_path.display()))?;

    // NOTE: Hardcoded rendering options from your example.
    let render_config = PdfRenderConfig::new()
        .set_maximum_width(3000); // Using DPI for better quality consistency

    let mut image_paths = Vec::new();
    for (i, page) in document.pages().iter().enumerate() {
        let output_path = temp_dir.path().join(format!("page_{:04}.png", i + 1));
        let image = page.render_with_config(&render_config)?.as_image();
        image.save(&output_path)
            .context(format!("Failed to save image for page {}", i + 1))?;
        image_paths.push(output_path);
    }
    Ok((temp_dir, image_paths))
}


/// Runs LepTess OCR on a single image.
/// This function is thread-safe as it creates a new LepTess instance for each call.
fn ocr_image(image_path: &Path) -> Result<String> {
    // Each thread gets its own LepTess instance.
    let mut lt = LepTess::new(None, "eng").context("Failed to initialize LepTess")?;
    lt.set_image(image_path)
        .context(format!("Failed to set image: {}", image_path.display()))?;
    let text = lt.get_utf8_text().context("Failed to get OCR text")?;
    Ok(text)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        anyhow::bail!("Usage: {} <path_to_pdf>", args[0]);
    }
    let pdf_path = Path::new(&args[1]);
    if !pdf_path.exists() {
        anyhow::bail!("PDF path does not exist: {}", pdf_path.display());
    }

    println!("Processing PDF: {}", pdf_path.display());

    // Step 1: Convert PDF to images and keep TempDir alive
    let (_temp_dir, images) = pdf_to_images(pdf_path).context("Failed to convert PDF to images")?;
    println!("Converted PDF to {} image(s).", images.len());

    // Step 2: OCR each page in parallel using Rayon
    println!("\n--- Starting OCR on {} pages in parallel ---", images.len());

    let page_texts: Vec<Result<String>> = images
        .par_iter()
        .map(|image_path| ocr_image(image_path))
        .collect();

    // Step 3: Print the results sequentially to maintain page order
    for (i, text_result) in page_texts.iter().enumerate() {
        println!("\n--- Page {} ---\n", i + 1);
        match text_result {
            Ok(text) => println!("{}", text),
            Err(e) => eprintln!("Error processing page {}: {}", i + 1, e),
        }
    }
    println!("\n--- End of Document ---");
    // TempDir is dropped here, cleaning up PNGs automatically
    Ok(())
}