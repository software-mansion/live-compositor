mod definition;
mod docs_config;
mod document;
mod generate_docs;
mod generate_json_schema;
mod markdown;
mod schema_parser;

fn main() {
    tracing_subscriber::fmt().init();
    let check_flag = std::env::args().any(|arg| &arg == "--check");
    generate_docs::generate_docs();
    generate_json_schema::generate_json_schema(check_flag);
}
