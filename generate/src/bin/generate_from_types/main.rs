mod definition;
mod docs_config;
mod document;
mod generate_docs;
mod generate_json_schema;
mod markdown;
mod schema_parser;

fn main() {
    generate_docs::generate_docs();
    generate_json_schema::generate_json_schema();
}
