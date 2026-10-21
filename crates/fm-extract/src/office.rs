use fm_core::{Result, error};
use std::io::{Cursor, Read};
fn xml_text(bytes: &[u8]) -> Result<String> {
    use quick_xml::events::Event;
    let mut reader = quick_xml::Reader::from_reader(bytes);
    let mut out = String::new();
    loop {
        match reader.read_event() {
            Ok(Event::Text(t)) => {
                let decoded = t.decode().map_err(error)?;
                out.push_str(&quick_xml::escape::unescape(&decoded).map_err(error)?);
            }
            Ok(Event::End(e)) => {
                if matches!(e.local_name().as_ref(), b"p" | b"tr" | b"br") {
                    out.push('\n');
                } else if e.local_name().as_ref() == b"tc" {
                    out.push('\t');
                }
            }
            Ok(Event::Eof) => break,
            Ok(Event::DocType(_)) => {
                return Err(error(
                    "DTD-Deklarationen sind in Office-Dateien nicht erlaubt",
                ));
            }
            Err(_) => return Err(error("Ungültiges XML in Office-Datei")),
            _ => {}
        }
    }
    Ok(out)
}
pub fn extract(ext: &str, bytes: &[u8]) -> Result<Vec<(String, String)>> {
    if ext == "xlsx" {
        use calamine::{Reader, Xlsx};
        let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(bytes)).map_err(error)?;
        let mut result = Vec::new();
        for name in workbook.sheet_names().to_vec() {
            let range = workbook.worksheet_range(&name).map_err(error)?;
            for (i, row) in range.rows().enumerate() {
                result.push((
                    format!("Blatt {name}, Zeile {}", i + 1),
                    row.iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join(" | "),
                ));
            }
        }
        return Ok(result);
    }
    let mut zip = zip::ZipArchive::new(Cursor::new(bytes)).map_err(error)?;
    if zip.len() > 10000 {
        return Err(error("Office-Datei enthält zu viele Einträge"));
    }
    let mut result = Vec::new();
    let mut total = 0u64;
    for i in 0..zip.len() {
        let mut f = zip.by_index(i).map_err(error)?;
        let name = f.name().to_string();
        let selected = if ext == "docx" {
            name == "word/document.xml"
        } else {
            name.starts_with("ppt/slides/slide")
                && name.ends_with(".xml")
                && !name.contains("_rels")
        };
        if !selected {
            continue;
        }
        total += f.size();
        if total > 100 * 1024 * 1024 {
            return Err(error("Entpackte Office-Texte sind zu gross"));
        }
        let mut bytes = Vec::new();
        f.by_ref()
            .take(100 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)
            .map_err(error)?;
        let text = xml_text(&bytes)?;
        let location = if ext == "docx" {
            "Abschnitt 1".to_string()
        } else {
            format!(
                "Folie {}",
                name.trim_start_matches("ppt/slides/slide")
                    .trim_end_matches(".xml")
            )
        };
        result.push((location, text));
    }
    result.sort_by_key(|(p, _)| {
        p.strip_prefix("Folie ")
            .and_then(|n| n.parse::<u32>().ok())
            .unwrap_or(0)
    });
    Ok(result)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn office_formats() {
        for (name, bytes) in [
            (
                "docx",
                include_bytes!("../../../tests/fixtures/unterricht.docx").as_slice(),
            ),
            (
                "pptx",
                include_bytes!("../../../tests/fixtures/unterricht.pptx").as_slice(),
            ),
            (
                "xlsx",
                include_bytes!("../../../tests/fixtures/unterricht.xlsx").as_slice(),
            ),
        ] {
            let text = extract(name, bytes).unwrap();
            assert!(text.iter().any(|(_, t)| t.contains("Netzwerk")), "{name}");
        }
    }
}
