use compositor_common::scene::Resolution;

use super::Url;

#[derive(Debug)]
pub enum Command<'a> {
    Use(Url<'a>),
    Resolution(Resolution),
    // TODO: Implement rendering onto web canvas
    // Source {
    //     name: &'a str,
    //     buffer: &'a [u8]
    // },
    Render,
}

impl<'a> Command<'a> {
    pub fn get_message(&self) -> Vec<u8> {
        let msg = match self {
            Command::Use(url) => format!("use:{url}"),
            Command::Resolution(Resolution { width, height }) => {
                format!("resolution:{width}x{height}")
            }
            Command::Render => "render".to_owned(),
        };

        msg.into_bytes()
    }
}
