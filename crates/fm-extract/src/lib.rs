mod ocr;
mod office;
use fm_core::{Chunk, Result, error};
use std::path::Path;
pub fn supported(path: &str) -> bool {
    Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| {
            matches!(
                s.to_lowercase().as_str(),
                "pdf"
                    | "png"
                    | "jpg"
                    | "jpeg"
                    | "bmp"
                    | "tif"
                    | "tiff"
                    | "docx"
                    | "pptx"
                    | "xlsx"
                    | "csv"
                    | "html"
                    | "htm"
                    | "txt"
                    | "md"
            )
        })
}
pub fn extract(path: &str, bytes: &[u8]) -> Result<Vec<Chunk>> {
    let ext = Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let pages: Vec<(String, String)> = match ext.as_str() {
        "pdf" => {
            let texts = pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(|_| {
                error("PDF kann nicht gelesen werden. Datei beschädigt oder verschlüsselt.")
            })?;
            if texts.iter().all(|t| t.trim().is_empty()) {
                ocr::extract(bytes, true)?
            } else {
                #[cfg(windows)]
                let recognized = if texts.iter().any(|t| t.trim().is_empty()) {
                    ocr::extract(bytes, true)?
                } else {
                    Vec::new()
                };
                texts
                    .into_iter()
                    .enumerate()
                    .map(|(i, t)| {
                        #[cfg(windows)]
                        if t.trim().is_empty()
                            && let Some(page) = recognized.get(i)
                        {
                            return page.clone();
                        }
                        (format!("S. {}", i + 1), t)
                    })
                    .collect()
            }
        }
        "png" | "jpg" | "jpeg" | "bmp" | "tif" | "tiff" => ocr::extract(bytes, false)?,
        "docx" | "pptx" | "xlsx" => office::extract(&ext, bytes)?,
        "csv" => vec![(
            "Zeilen".into(),
            String::from_utf8(bytes.to_vec()).map_err(error)?,
        )],
        "html" | "htm" => {
            let text = String::from_utf8(bytes.to_vec()).map_err(error)?;
            let doc = scraper::Html::parse_document(&text);
            let selector = scraper::Selector::parse("body").map_err(error)?;
            let body = doc
                .select(&selector)
                .next()
                .map(|n| n.text().collect::<Vec<_>>().join(" "))
                .unwrap_or_default();
            vec![("Abschnitt 1".into(), body)]
        }
        "txt" | "md" => vec![(
            "Zeilen".into(),
            String::from_utf8(bytes.to_vec()).map_err(error)?,
        )],
        _ => return Err(error("Dieses Dateiformat wird noch nicht unterstützt")),
    };
    let hash = fm_core::hash(bytes);
    let mut chunks = Vec::new();
    for (location, text) in pages {
        let lines: Vec<_> = text.lines().collect();
        for (i, part) in lines.chunks(24).enumerate() {
            let text = part.join("\n");
            if text.trim().is_empty() {
                continue;
            }
            let location = if location == "Zeilen" {
                format!("Z. {}–{}", i * 24 + 1, i * 24 + part.len())
            } else {
                location.clone()
            };
            chunks.push(Chunk {
                id: fm_core::hash(format!("{path}:{hash}:{location}:{i}").as_bytes()),
                path: path.into(),
                hash: hash.clone(),
                location,
                text,
                extracted_at: fm_core::now(),
                extractor: format!("folder-mind/{ext}"),
            });
        }
    }
    if chunks.is_empty() {
        return Err(error(
            "Kein Text gefunden. Das Dokument benötigt eventuell OCR.",
        ));
    }
    Ok(chunks)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formats_and_locations() {
        for (p, b) in [
            ("a.txt", b"Netzwerk\nTCP".as_slice()),
            ("a.md", b"# Netzwerk".as_slice()),
            ("a.html", b"<html><body>Netzwerk</body></html>".as_slice()),
        ] {
            let c = extract(p, b).unwrap();
            assert!(c[0].text.contains("Netzwerk"));
            assert!(!c[0].location.is_empty());
        }
    }
}

#[cfg(test)]
mod pdf_test {
    #[test]
    fn pdf_page() {
        let c = super::extract(
            "netzwerk.pdf",
            include_bytes!("../../../tests/fixtures/netzwerk.pdf"),
        )
        .unwrap();
        assert!(c[0].text.contains("Ethernet"));
        assert_eq!(c[0].location, "S. 1");
    }
}
