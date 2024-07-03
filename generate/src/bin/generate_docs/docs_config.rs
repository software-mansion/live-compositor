use std::collections::HashMap;

use crate::{
    definition::TypeDefinition,
    markdown::{overrides, MarkdownGenerator},
};

type OverrideFn = fn(&mut MarkdownGenerator<'_>, TypeDefinition);

#[derive(Clone)]
pub struct DocsConfig {
    pub ignored_definitions: Vec<&'static str>,
    pub always_inlined_definitions: Vec<&'static str>,
    pub never_inlined_definitions: Vec<&'static str>,
    pub variant_discriminators: HashMap<&'static str, &'static str>,
    pub overrides: HashMap<&'static str, OverrideFn>,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            ignored_definitions: vec!["Component"],
            always_inlined_definitions: vec!["PortOrPortRange", "Resolution", "Audio", "Video"],
            never_inlined_definitions: vec![],
            variant_discriminators: [
                ("AudioEncoderOptions", "type"),
                ("VideoEncoderOptions", "type"),
                ("InputRtpVideoOptions", "decoder"),
                ("InputRtpAudioOptions", "decoder"),
            ]
            .into(),
            overrides: [
                ("InputStream", overrides::force_multiline as OverrideFn),
                ("Resolution", overrides::force_multiline),
                ("InputAudio", overrides::force_multiline),
            ]
            .into(),
        }
    }
}
