fn main() {
    let target = "assets/wdl.epub";
    let out = "assets/raw/wdl/html";

    if let Err(e) = common::extract_html_resources(target, out) {
        eprintln!("Error extracting HTML: {}", e);
    }
}
