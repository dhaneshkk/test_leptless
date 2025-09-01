
# Multi-column  PDF OCR in Rust

`leptless` is a Rust tool that extracts text from PDF files using **Pdfium** for rendering pages and **Tesseract** for OCR. It analyzes the text layout to choose the best Tesseract Page Segmentation Mode (PSM) for improved OCR accuracy.

## Features

* Render PDFs to high-resolution images using Pdfium.
* Automatically analyze text layout to determine single vs multi-column pages.
* Use Tesseract for OCR with automatic PSM selection.
* Supports multi-page PDF files.
* Memory-efficient by processing page-by-page.

## Requirements

* Rust 1.70+ (or latest stable)
* [Pdfium library](https://pdfium.googlesource.com/pdfium/) installed on your system
* Tesseract OCR installed with English language data (`eng`)
* `pkg-config` (for linking Pdfium)

On Ubuntu:

```bash
sudo apt update
sudo apt install tesseract-ocr libtesseract-dev pkg-config
```

Pdfium can be installed via your package manager or built from source.

## Installation

1. Clone the repository:

```bash
git clone https://github.com/yourusername/leptless.git
cd leptless
```

2. Build the project:

```bash
cargo build --release
```

## Usage

Place your PDF in the project directory (e.g., `input.pdf`), then run:

```bash
cargo run
```

The program will process each page, analyze its layout, and print the OCR text:

```
--- Page 1 ---
Text extracted from page 1...

--- Page 2 ---
Text extracted from page 2...
```

## How it Works

1. **Pdfium** renders each PDF page to a high-resolution bitmap.
2. The program analyzes character positions (`tight_bounds`) to estimate column layout.
3. Based on the horizontal spread of text, the program selects a suitable **Tesseract PSM**:

    * Wide single-block pages → `PsmSingleBlock`
    * Multi-column pages → `PsmAutoOsd`
4. **Tesseract** OCR runs on the rendered image, extracting text.

## Determine multi-column

```rust
let mut tess = Tesseract::new(None, Some("eng"))?;
if column_ratio > 0.8 {
    tess.set_page_seg_mode(tesseract::PageSegMode::PsmSingleBlock);
} else {
    tess.set_page_seg_mode(tesseract::PageSegMode::PsmAutoOsd);
}
tess.set_image_from_mem(&buf)?;
let text = tess.get_text()?;
```
### column-ratio calculation

1.  **`page.width()`**: This method is called on the `PdfPage` object to get its width as a `PdfPoints` struct. We use `.value` to get the floating-point number representing the width.
2.  **`page.text()?`**: This extracts a `PdfText` object, which provides access to the text content and character details.
3.  **Initialization**: `min_x` is initialized to `f32::MAX` and `max_x` to `f32::MIN`. This ensures that the boundaries of the very first character processed will become the initial values for `min_x` and `max_x`.
4.  **Character Iteration**: The code loops through every character on the page. For each one, `tight_bounds()` provides its precise location and size.
5.  **Boundary Tracking**:
    *   `min_x.min(bounds.left())` updates `min_x` if the current character is further to the left than any character seen before.
    *   `max_x.max(bounds.left() + bounds.width())` calculates the right edge of the current character and updates `max_x` if this character extends further to the right.
6.  **Ratio Calculation**: Finally, the width of the text block (`max_x - min_x`) is divided by the `page_width`. A check is included to handle pages with no text, preventing a division-by-zero or nonsensical result and correctly returning a ratio of `0.0`.
## License

MIT License. See `LICENSE` for details.

---

I can also create a **visual diagram for the workflow** for this README, which makes it clearer for new users.

Do you want me to add that diagram?
