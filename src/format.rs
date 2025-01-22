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
