use std::fs;
use utoipa::OpenApi;
// in ./src/gen_openapi.rs
fn main() {
    let doc = backend::ApiDoc::openapi().to_pretty_json().unwrap();
    let _ = fs::write("./api/openapi.json", doc);
    let doc = backend::ApiDoc::openapi().to_yaml().unwrap();
    let _ = fs::write("./api/openapi.yaml", doc);
}
