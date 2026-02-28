use quick_xml::de::from_str;
use serde::Deserialize;

#[derive(Debug)]
pub struct Metadata {
    pub title: String,
    pub author: String,
    pub language: String,
    pub publisher: String,
    pub date: String,
}

#[derive(Debug, Deserialize)]
struct Package {
    metadata: OpfMetadata,
}

#[derive(Debug, Deserialize)]
struct OpfMetadata {
    #[serde(rename = "title")]
    title: Option<String>,
    #[serde(rename = "creator", default)]
    creators: Vec<Creator>,
    #[serde(rename = "language")]
    language: Option<String>,
    #[serde(rename = "publisher")]
    publisher: Option<String>,
    #[serde(rename = "date")]
    date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Creator {
    #[serde(rename = "@role")]
    role: Option<String>,
    #[serde(rename = "$text")]
    text: Option<String>,
}

pub fn parse_opf(xml: &str) -> Result<Metadata, Box<dyn std::error::Error>> {
    let package: Package = from_str(xml)?;
    let meta = package.metadata;

    let author = meta
        .creators
        .iter()
        .find(|c| c.role.as_deref() == Some("aut"))
        .or_else(|| meta.creators.first())
        .and_then(|c| c.text.clone())
        .unwrap_or_default();

    Ok(Metadata {
        title: meta.title.unwrap_or_default(),
        author,
        language: meta.language.unwrap_or_default(),
        publisher: meta.publisher.unwrap_or_default(),
        date: meta.date.unwrap_or_default(),
    })
}
