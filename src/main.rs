
use std::path::{Path,PathBuf};
use std::env;
use leptess::LepTess;
use std::process::Command;
use std::fs;
use tempfile::TempDir;
use anyhow::{Context,Result};

/// Converts PDF to PNG images using pdftoppm.
/// Returns a tuple: (TempDir handle, Vec of image paths)
fn pdf_to_images(pdf_path: &Path) -> Result<(TempDir, Vec<PathBuf>)> {
    let temp_dir = TempDir::new().context("Failed to create temporary directory")?;
    let output_prefix = temp_dir.path().join("page");

    // Convert PDF pages to PNG at 300 DPI
    let status = Command::new("pdftoppm")
        .arg("-png")
        .arg("-r")
        .arg("300")
        .arg(pdf_path)
        .arg(&output_prefix)
        .status()
        .context("Failed to execute pdftoppm")?;

    if !status.success() {
        anyhow::bail!("pdftoppm failed with status: {}", status);
    }

    // Collect generated PNGs
    let mut images = Vec::new();
    for entry in fs::read_dir(temp_dir.path()).context("Failed to read temp dir")? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|s| s == "png").unwrap_or(false) {
            images.push(path);
        }
    }

    images.sort(); // Ensure pages are in order
    Ok((temp_dir, images))
}

/// Runs LepTess OCR on a single image
fn ocr_image(image_path: &Path) -> Result<String> {
    let mut lt = LepTess::new(None, "eng").context("Failed to initialize LepTess")?;
    let _=lt.set_image(image_path);
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
    let (_temp_dir, images) = pdf_to_images(pdf_path).context("Failed to convert PDF to
images")?;
    println!("Converted PDF to {} image(s).", images.len());

    // Step 2: OCR each page
    for (i, image_path) in images.iter().enumerate() {
        println!("\n--- Page {} ---\n", i + 1);
        let text = ocr_image(image_path).context("Failed to OCR page")?;
        println!("{}", text);
    }
    println!("\n--- End of Document ---");
    // TempDir is dropped here, cleaning up PNGs automatically
    Ok(())
}