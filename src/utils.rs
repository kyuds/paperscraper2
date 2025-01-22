use std::{
    fs::File,
    io::{self, Write}
};

use crate::parser::ArxivResult;

// Formatter for various Arxiv data results.
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

    // pub fn to_bedrock_input
}

// Utils to store readme files on local device.
pub struct LocalSaver;

impl LocalSaver {
    pub fn save_raw_arxiv_results_as_readme(fname: &str, data: &Vec<ArxivResult>) -> io::Result<()> {
        let mut file = File::create(fname)?;
        data.iter().try_for_each(|result| -> io::Result<()> {
            file.write_all(Formatter::to_readme(result).as_bytes())?;
            Ok(())
        })?;
        file.flush()?;
        Ok(())
    }
}

// Utils to store data to AWS S3.
pub struct S3Saver;

impl S3Saver {

}
