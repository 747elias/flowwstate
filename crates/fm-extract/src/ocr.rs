use fm_core::{Result, error};
#[cfg(not(windows))]
pub fn extract(_bytes: &[u8], _pdf: bool) -> Result<Vec<(String, String)>> {
    Err(error(
        "OCR ist im Windows-Build über die integrierte Windows-Texterkennung verfügbar.",
    ))
}
#[cfg(windows)]
pub fn extract(bytes: &[u8], pdf: bool) -> Result<Vec<(String, String)>> {
    let bytes = bytes.to_vec();
    std::thread::spawn(move || run(&bytes, pdf))
        .join()
        .map_err(|_| error("Windows-Texterkennung wurde unerwartet beendet"))?
}
#[cfg(windows)]
fn run(bytes: &[u8], pdf: bool) -> Result<Vec<(String, String)>> {
    use windows::{
        Data::Pdf::{PdfDocument, PdfPageRenderOptions},
        Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat},
        Media::Ocr::OcrEngine,
        Storage::Streams::{DataWriter, InMemoryRandomAccessStream},
        Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
    };
    // Ein eigener MTA-Thread hält WinRT vom Tray-Nachrichtenloop getrennt.
    unsafe {
        RoInitialize(RO_INIT_MULTITHREADED)
            .map_err(|_| error("Windows-Texterkennung konnte nicht initialisiert werden"))?;
    }
    struct Runtime;
    impl Drop for Runtime {
        fn drop(&mut self) {
            unsafe {
                RoUninitialize();
            }
        }
    }
    let _runtime = Runtime;
    let result = (|| -> windows::core::Result<Vec<(String, String)>> {
        let stream = InMemoryRandomAccessStream::new()?;
        let output = stream.GetOutputStreamAt(0)?;
        let writer = DataWriter::CreateDataWriter(&output)?;
        writer.WriteBytes(bytes)?;
        writer.StoreAsync()?.get()?;
        writer.FlushAsync()?.get()?;
        writer.DetachStream()?;
        stream.Seek(0)?;
        let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
        let recognize = |image: &InMemoryRandomAccessStream| -> windows::core::Result<String> {
            image.Seek(0)?;
            let decoder = BitmapDecoder::CreateAsync(image)?.get()?;
            let bitmap = decoder
                .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Bgra8, BitmapAlphaMode::Ignore)?
                .get()?;
            Ok(engine.RecognizeAsync(&bitmap)?.get()?.Text()?.to_string())
        };
        if pdf {
            let document = PdfDocument::LoadFromStreamAsync(&stream)?.get()?;
            let mut pages = Vec::new();
            for i in 0..document.PageCount()? {
                let page = document.GetPage(i)?;
                let image = InMemoryRandomAccessStream::new()?;
                let options = PdfPageRenderOptions::new()?;
                options.SetDestinationWidth(1600)?;
                page.RenderWithOptionsToStreamAsync(&image, &options)?
                    .get()?;
                pages.push((format!("S. {} (OCR)", i + 1), recognize(&image)?));
                page.Close()?;
            }
            Ok(pages)
        } else {
            Ok(vec![("Bild (OCR)".into(), recognize(&stream)?)])
        }
    })();
    result.map_err(|_|error("Windows-Texterkennung nicht verfügbar oder Bild nicht lesbar. Prüfe die in Windows vorhandenen OCR-Sprachen; alternativ eine PDF mit eingebettetem Text verwenden."))
}
