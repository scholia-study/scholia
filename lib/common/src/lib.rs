use epub::doc::EpubDoc;
use std::fs::{File, create_dir_all};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Extracts all HTML resources from an EPUB file and writes them to the output folder.
pub fn extract_html_resources(
    epub_path: &str,
    output_folder: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc = EpubDoc::new(epub_path)?;
    let title = doc.mdata("title").unwrap_or("untitled".to_string());
    println!("Extracting HTML from: {}", title);

    create_dir_all(output_folder)?;

    for resource_name in doc.resources.clone().into_keys() {
        if let Ok(raw_bytes) = doc.get_resource(&resource_name) {
            let html_str = String::from_utf8(raw_bytes)?;
            let output_path = Path::new(output_folder).join(sanitize_filename(&resource_name));
            let mut file = File::create(output_path)?;
            file.write_all(html_str.as_bytes())?;
            println!("Saved: {}", resource_name);
        }
    }

    Ok(())
}

/// Sanitizes filenames by replacing problematic characters (e.g., slashes).
fn sanitize_filename(name: &str) -> String {
    name.replace("/", "_").replace("\\", "_")
}
