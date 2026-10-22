use pulldown_cmark::{CowStr, Event, Options, Parser, Tag, TagEnd, html};

pub fn render(text: &str) -> String {
    let options =
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let events = Parser::new_ext(text, options).filter_map(|event| match event {
        Event::Html(raw) | Event::InlineHtml(raw) => Some(Event::Text(raw)),
        Event::Start(Tag::Image { .. }) | Event::End(TagEnd::Image) => None,
        Event::Start(Tag::Link {
            link_type,
            dest_url,
            title,
            id,
        }) => {
            let safe = dest_url.starts_with('#')
                || (dest_url.starts_with('/')
                    && !dest_url.starts_with("//")
                    && !dest_url.contains('\\'))
                || url::Url::parse(&dest_url)
                    .is_ok_and(|u| matches!(u.scheme(), "http" | "https" | "mailto"));
            Some(Event::Start(Tag::Link {
                link_type,
                dest_url: if safe { dest_url } else { CowStr::from("#") },
                title,
                id,
            }))
        }
        event => Some(event),
    });
    let mut output = String::new();
    html::push_html(&mut output, events);
    output
}

pub fn cited(text: &str, sources: &[fm_core::Chunk]) -> String {
    let mut text = text.to_string();
    for (i, source) in sources.iter().enumerate() {
        text = text.replace(
            &format!("[{}]", source.id),
            &format!("[{}](/api/source?id={})", i + 1, source.id),
        );
    }
    render(&text)
}

#[cfg(test)]
mod tests {
    #[test]
    fn formats_markdown_without_executable_html() {
        let html = super::render(
            "# Titel\n\n**Fett** und `Code`\n\n- Eintrag\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n<script>alert(1)</script>\n\n[Link](javascript:alert%281%29)\n\n![Bild](https://example.org/tracker)",
        );
        assert!(html.contains("<h1>Titel</h1>"));
        assert!(html.contains("<strong>Fett</strong>"));
        assert!(html.contains("<table>"));
        assert!(!html.contains("<script>"));
        assert!(!html.contains("href=\"javascript:"));
        assert!(!html.contains("<img"));
    }
}
