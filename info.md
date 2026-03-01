im using utoipa - Auto-generated OpenAPI documentation for my rust axum server. is there a way to generate the open api spec at compile time or must i run the server and visit the api to get the spec?

Gemini said
You're in luck. While utoipa generates the internal structure at compile time using macros, the actual JSON/YAML file isn't automatically written to your disk just by building. However, you definitely don't need to run the full server and "scrape" the endpoint to get it.

Here are the two best ways to handle this:

1. The "Helper Binary" approach (Recommended)
   This is the most common pattern in the Rust ecosystem. You create a tiny secondary entry point in your project that simply prints the spec to the console or a file.

```rs
use utoipa::OpenApi;
use your_crate::ApiDoc; // Import your actual ApiDoc struct

fn main() {
    let doc = ApiDoc::openapi();
    println!("{}", doc.to_pretty_json().unwrap());
}
```

```sh
cargo run --bin gen_openapi > openapi.json
```
