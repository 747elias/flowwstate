use fm_core::{Chunk, Guard, Result, error};
use std::path::Path;
pub fn chapter_files(chapter: &str) -> Result<(String, String, String)> {
    let relative = chapter
        .strip_prefix("unterlagen/")
        .ok_or_else(|| error("Bitte ein Kapitel innerhalb von unterlagen wählen"))?;
    if relative.is_empty() {
        return Err(error("Kapitel fehlt"));
    }
    let base = format!("wissen/{relative}");
    Ok((
        format!("{base}.generated.md"),
        format!("{base}.notes.md"),
        format!("{base}.md"),
    ))
}
pub fn validate_citations(answer: &str, sources: &[Chunk]) -> Result<Vec<String>> {
    let mut ids = Vec::new();
    for part in answer.split('[').skip(1) {
        if let Some(id) = part.split(']').next()
            && id.len() == 64
            && id.bytes().all(|b| b.is_ascii_hexdigit())
        {
            if !sources.iter().any(|c| c.id == id) {
                return Err(error(
                    "Antwort enthält eine unbekannte Quelle und wurde verworfen",
                ));
            }
            if !ids.iter().any(|s| s == id) {
                ids.push(id.to_string());
            }
        }
    }
    if ids.is_empty() {
        return Err(error(
            "Antwort enthält keinen Quellenbeleg und wurde verworfen",
        ));
    }
    Ok(ids)
}
pub fn write_draft(
    guard: &Guard,
    chapter: &str,
    body: &str,
    sources: &[Chunk],
    model: &str,
) -> Result<String> {
    let used = validate_citations(body, sources)?;
    let (generated, _, _) = chapter_files(chapter)?;
    if let Some(p) = Path::new(&generated).parent() {
        guard.mkdir(p)?;
    }
    let quote = |s: &str| serde_json::to_string(s).unwrap_or_else(|_| "\"\"".into());
    let mut text = format!(
        "---\nid: {}\ntitel: {}\npfad: {}\nstatus: wartet_auf_freigabe\nquellen:\n",
        uuid::Uuid::new_v4(),
        quote(chapter.rsplit('/').next().unwrap_or(chapter)),
        quote(chapter)
    );
    for c in sources.iter().filter(|c| used.contains(&c.id)) {
        text += &format!(
            "  - datei: {}\n    hash: {}\n    stellen: [{}]\n    abschnitt: {}\n",
            quote(&c.path),
            quote(&format!("sha256:{}", c.hash)),
            quote(&c.location),
            quote(&c.id)
        );
    }
    text += &format!(
        "generator:\n  version: {}\n  modell: {}\n---\n\n{}\n",
        quote(concat!("folder-mind ", env!("CARGO_PKG_VERSION"))),
        quote(model),
        body
    );
    guard.write(&generated, text.as_bytes())?;
    Ok(generated)
}
pub fn approve(guard: &Guard, chapter: &str, sources: &[Chunk]) -> Result<()> {
    let (generated, _, approved) = chapter_files(chapter)?;
    let text = std::fs::read_to_string(guard.resolve(&generated)?)?;
    let body = text
        .splitn(3, "---")
        .nth(2)
        .ok_or_else(|| error("Entwurf enthält keine gültigen Metadaten"))?;
    validate_citations(body, sources)?;
    guard.write(
        approved,
        text.replacen("status: wartet_auf_freigabe", "status: freigegeben", 1)
            .as_bytes(),
    )
}
pub fn notes(guard: &Guard, chapter: &str, text: &str) -> Result<()> {
    let (_, notes, _) = chapter_files(chapter)?;
    if let Some(p) = Path::new(&notes).parent() {
        guard.mkdir(p)?;
    }
    guard.write(notes, text.as_bytes())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn no_source_no_approval_and_notes_survive() {
        let t = tempfile::tempdir().unwrap();
        let g = Guard::new(t.path()).unwrap();
        let c = Chunk {
            id: "a".repeat(64),
            path: "unterlagen/a/x.txt".into(),
            hash: "hash".into(),
            location: "Z. 1".into(),
            text: "TCP".into(),
            extracted_at: 0,
            extractor: "txt".into(),
        };
        assert!(validate_citations("Ohne Beleg", std::slice::from_ref(&c)).is_err());
        assert!(
            validate_citations(&format!("[{}]", "b".repeat(64)), std::slice::from_ref(&c)).is_err()
        );
        notes(&g, "unterlagen/a", "Eigene Gedanken").unwrap();
        let text = format!("# Kapitel\n\n## Zusammenfassung\nTCP [{}]", c.id);
        write_draft(&g, "unterlagen/a", &text, std::slice::from_ref(&c), "mock").unwrap();
        approve(&g, "unterlagen/a", &[c]).unwrap();
        assert_eq!(
            std::fs::read_to_string(g.resolve("wissen/a.notes.md").unwrap()).unwrap(),
            "Eigene Gedanken"
        );
        assert!(
            std::fs::read_to_string(g.resolve("wissen/a.md").unwrap())
                .unwrap()
                .contains("status: freigegeben")
        );
    }
}
