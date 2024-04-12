use std::collections::HashMap;

pub struct DocsConfig {
    pub ignored_definitions: Vec<&'static str>,
    pub always_inlined_definitions: Vec<&'static str>,
    pub never_inlined_definitions: Vec<&'static str>,
    pub variant_discriminators: HashMap<&'static str, &'static str>,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            ignored_definitions: vec!["Component"],
            always_inlined_definitions: vec!["PortOrPortRange", "Resolution", "Audio", "Video"],
            never_inlined_definitions: vec![],
            variant_discriminators: [("AudioEncoderOptions", "type")].into(),
        }
    }
}
