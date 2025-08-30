This Rust program performs Optical Character Recognition (OCR) on a PDF file by converting each page into an image and then extracting the text from that image. The extracted text from all pages is then saved into a single text file.

Here's a breakdown of the code's functionality:

### Initialization and PDF Loading
*   **PDFium Initialization**: The code begins by initializing the `pdfium-render` library, which is a Rust wrapper for the PDFium library used by Google Chrome to render PDF files. It first attempts to load the PDFium library from a local `./lib` directory and, if that fails, falls back to a system-wide installation.
*   **Loading the PDF**: A specific PDF file, `/home/ubuntu/leptless/2021.eacl-main.75.pdf`, is loaded into memory. The program then prints the total number of pages in the document.

### Page Processing Loop
The program iterates through each page of the loaded PDF to perform the following actions sequentially, which helps in keeping memory usage low.

*   **Rendering**: Each page is rendered into an image with a resolution of 2480x3508 pixels (approximately 300 DPI for a standard A4 page), which is a good quality for OCR. The `pdfium-render` crate handles this conversion from a PDF page to a bitmap image.
*   **Image Encoding**: The rendered bitmap is then encoded into the PNG format and stored in memory. This step is necessary because the OCR library, `leptess`, will work with this PNG data. The PNG image itself is not saved to a file.
*   **Optical Character Recognition (OCR)**:
    *   An instance of `LepTess` is created. `LepTess` is a high-level wrapper from the `leptess` crate, which provides bindings for the Tesseract OCR engine. The OCR engine is configured to use the English language model ("eng").
    *   The in-memory PNG image data is passed to the `LepTess` instance.
    *   The `get_utf8_text()` function is called to perform OCR and retrieve the recognized text.
*   **Writing to File**: The extracted text from the page is written to a file named `ocr_output.txt`. Each page's text is clearly demarcated with a "===== Page X =====" header. This direct writing to disk after processing each page also contributes to efficient memory management.

### Completion
Once all the pages have been processed, the program prints a completion message to the console, indicating that the OCR results have been saved to `ocr_output.txt`.