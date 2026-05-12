use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

pub struct EpubReader {
    archive: ZipArchive<File>,
    /// Cache of already-read files
    cache: HashMap<String, String>,
}

impl EpubReader {
    pub fn open(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let file = File::open(path)?;
        let archive = ZipArchive::new(file)?;
        Ok(Self {
            archive,
            cache: HashMap::new(),
        })
    }

    /// Read a file from the EPUB archive as a UTF-8 string.
    pub fn read_file(&mut self, name: &str) -> Result<String, Box<dyn std::error::Error>> {
        if let Some(cached) = self.cache.get(name) {
            return Ok(cached.clone());
        }
        let mut entry = self.archive.by_name(name)?;
        let mut buf = String::new();
        entry.read_to_string(&mut buf)?;
        self.cache.insert(name.to_string(), buf.clone());
        Ok(buf)
    }
}
