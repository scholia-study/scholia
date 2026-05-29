fn main() {
    // The OpenAPI doc is produced by the assembled router (the live
    // source of truth), not a hand-maintained list. Split off the doc
    // and discard the axum router — no DB pool is needed.
    let (_router, api) = api::api_router().split_for_parts();
    println!("{}", api.to_pretty_json().unwrap());
}
