# PDF-OCR-Rust

This project is a command-line utility written in Rust that performs Optical Character Recognition (OCR) on a given PDF file. It extracts the text from each page and saves the combined text into a single output file.

The program is designed to be memory-efficient, processing one page at a time. This approach allows it to handle large PDF files without consuming excessive amounts of RAM.

## About The Project

This utility leverages the power of the Tesseract OCR engine via Rust bindings. The core functionality involves:

1.  **PDF Page Rendering**: Each page of the input PDF is rendered into a high-resolution in-memory image (as a bitmap).
2.  **Image-to-Text Conversion**: The generated image is then processed by the Tesseract OCR engine to extract any text.
3.  **Sequential Processing**: To keep memory usage low, the program processes each page sequentially—rendering, performing OCR, and writing the result to a file before moving to the next page.
4.  **Consolidated Output**: The extracted text from all pages is appended to a single output file, with clear separators for each page.

### Built With

*   [Rust](https://www.rust-lang.org/)
*   [pdfium-render](https://crates.io/crates/pdfium-render) - For rendering PDF pages.
*   [leptess](https://crates.io/crates/leptess) - A high-level wrapper for the Tesseract OCR engine.
*   [image](https://crates.io/crates/image) - For handling image data in memory.

## Getting Started

To get a local copy up and running, follow these steps.

### Prerequisites

This project has external dependencies that need to be installed on your system before you can build and run the program.

*   **Rust Toolchain**: If you don't have Rust installed, you can get it from [rustup.rs](https://rustup.rs/).

*   **Tesseract OCR Engine & Development Libraries**
    *   **On Debian/Ubuntu-based systems**:
        ```sh
        sudo apt-get update
        sudo apt-get install tesseract-ocr libtesseract-dev
        ```
    *   **On macOS** (using Homebrew):
        ```sh
        brew install tesseract
        ```

*   **PDFium Library**
    *   The `pdfium-render` crate needs access to the PDFium library. The easiest way to provide this is to download a pre-compiled binary and place it in a location where the program can find it.
    *   **Download**: You can find pre-compiled binaries for various platforms on the [pdfium-binaries GitHub repository](https://github.com/bblanchon/pdfium-binaries/releases).
    *   **Setup**:
        1.  Download the appropriate binary for your system (e.g., `pdfium-linux-x64.tgz` for 64-bit Linux).
        2.  Extract the archive.
        3.  Create a `lib` directory in the root of this project.
        4.  Copy the PDFium library file (e.g., `libpdfium.so` on Linux, `pdfium.dll` on Windows, or `libpdfium.dylib` on macOS) into the newly created `lib` directory.

### Installation

1.  **Clone the repo**:
    ```sh
    git clone https://github.com/your_username/pdf-ocr-rust.git
    cd pdf-ocr-rust
    ```

2.  **Build the project**:
    ```sh
    cargo build --release
    ```

## Usage

To run the OCR process on a PDF file, you first need to modify the `main.rs` file to point to your desired input PDF.

1.  **Open `src/main.rs`** in a text editor.

2.  **Change the PDF file path**:
    *   Locate the following line:
        ```rust
        let doc = pdfium.load_pdf_from_file(
            "/home/ubuntu/leptless/2021.eacl-main.75.pdf",
            None,
        )?;
        ```    *   Replace `"/home/ubuntu/leptless/2021.eacl-main.75.pdf"` with the absolute or relative path to your PDF file.

3.  **Run the program**:
    ```sh
    cargo run --release
    ```

The program will print progress updates to the console for each page it processes.

```
Loaded PDF with X pages
---- Rendering Page 1 ----
---- OCR Page 1 complete and written ----
---- Rendering Page 2 ----
---- OCR Page 2 complete and written ----
...
✅ OCR complete. Results saved to ocr_output.txt
```

Upon completion, you will find a file named `ocr_output.txt` in the project's root directory containing the extracted text from the PDF.