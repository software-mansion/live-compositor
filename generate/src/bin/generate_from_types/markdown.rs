use crate::{definition::TypeDefinition, docs_config::DocsConfig};

pub mod overrides;
mod properties;
mod type_definition;

pub struct MarkdownGenerator<'a> {
    pub(super) indent: usize,
    document: String,
    was_newline: bool,
    config: &'a DocsConfig,
}

impl MarkdownGenerator<'_> {
    const INDENT_SIZE: usize = 2;

    pub fn generate(definition: TypeDefinition, config: &DocsConfig) -> String {
        let mut generator = MarkdownGenerator {
            indent: 0,
            document: String::new(),
            was_newline: true,
            config,
        };

        let name = definition.name.clone().unwrap();
        let description = definition.description.clone();

        generator.add_header(2, &name);
        generator.with_codeblock(|code_block| {
            code_block.add_text(format!("type {name} = "));
            code_block.write_type_definition(definition.clone());
        });
        generator.add_text(description);
        generator.write_properties(&definition);

        generator.document
    }

    fn add_header<S: Into<String>>(&mut self, level: usize, text: S) {
        if !self.was_newline {
            self.add_text("\n");
        }
        self.add_text(format!("{} {}\n", "#".repeat(level), text.into()));
    }

    pub fn with_codeblock<F: FnOnce(&mut Self)>(&mut self, f: F) {
        self.add_text("```typescript\n");
        f(self);
        self.add_text("\n```\n");
    }

    pub fn add_text<S: Into<String>>(&mut self, text: S) {
        const INDENT: &str = " ";

        let text = text.into();
        for c in text.chars() {
            if self.was_newline {
                self.document.push_str(&INDENT.repeat(self.indent));
                self.was_newline = false;
            }

            self.document.push(c);
            if c == '\n' {
                self.was_newline = true;
            }
        }
    }

    pub fn inc_indent(&mut self) {
        self.indent += Self::INDENT_SIZE;
    }

    pub fn dec_indent(&mut self) {
        self.indent -= Self::INDENT_SIZE;
    }

    pub fn calculate_generation_length<F: Fn(&mut MarkdownGenerator)>(&self, f: F) -> usize {
        let mut generator = MarkdownGenerator {
            indent: self.indent,
            document: String::new(),
            was_newline: false,
            config: self.config,
        };

        f(&mut generator);
        generator.document.len()
    }
}
