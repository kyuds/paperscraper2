use crate::parser::ArxivResult;

pub struct Formatter;

impl Formatter {
    pub fn to_readme(data: &ArxivResult) -> String {
        format!("### {}\n_{}_<br/>\n{}<br/>\n_Published: {}_, [{}]({})\n\n",
            data.title,
            data.authors.join(", "),
            data.summary,
            data.published.format("%Y.%m.%d"),
            data.link, data.link
        )
    }

    pub fn to_jsonl_with_id(id: usize, data: &ArxivResult) -> String {
        format!(
            concat!("{{\"id\": {}, \"title\": \"{}\", \"authors\": [{}], ",
                    "\"summary\": \"{}\", \"pub\": \"{}\", \"link\": \"{}\"}}\n"),
            id,
            data.title,
            data.authors.iter().map(|a| { format!("\"{}\"", a)}).collect::<Vec<_>>().join(", "),
            data.summary,
            data.published.format("%Y.%m.%d"),
            data.link
        )
    }

    // pub fn to_bedrock_input
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};

    fn get_sample_arxiv() -> ArxivResult {
        ArxivResult {
            title: "title".to_string(),
            summary: "summary".to_string(),
            authors: vec!["john doe".to_string()],
            published: Utc.timestamp_opt(0, 0).unwrap(),
            link: "www.example.com".to_string()
        }
    }

    const BASE_README: &str = concat!(
        "### title\n_john doe_<br/>\nsummary<br/>\n_Published: 1970.01.01_, ",
        "[www.example.com](www.example.com)\n\n"
    );

    const BASE_JSONL: &str = concat!(
        "{\"id\": 0, \"title\": \"title\", \"authors\": [\"john doe\"], \"summary\": \"summary\", ",
        "\"pub\": \"1970.01.01\", \"link\": \"www.example.com\"}\n"
    );

    #[test]
    fn test_readme() {
        let base = String::from(BASE_README);
        let readme = Formatter::to_readme(&get_sample_arxiv());
        assert_eq!(base, readme);
    }

    #[test]
    fn test_jsonl() {
        let base = String::from(BASE_JSONL);
        let jsonl = Formatter::to_jsonl_with_id(0, &get_sample_arxiv());
        assert_eq!(base, jsonl);
    }
}
