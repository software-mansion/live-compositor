use decklink::{
    get_decklinks, AudioSampleType, DeckLinkError, DisplayModeType, PixelFormat,
    SupportedVideoModeFlags, VideoConnection, VideoInputConversionMode, VideoInputFlags,
};

pub struct ErrorStack<'a>(Option<&'a (dyn std::error::Error + 'static)>);

impl<'a> ErrorStack<'a> {
    pub fn new(value: &'a (dyn std::error::Error + 'static)) -> Self {
        ErrorStack(Some(value))
    }

    pub fn into_string(self) -> String {
        let stack: Vec<String> = self.map(ToString::to_string).collect();
        stack.join("\n")
    }
}

impl<'a> Iterator for ErrorStack<'a> {
    type Item = &'a (dyn std::error::Error + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.map(|err| {
            self.0 = err.source();
            err
        })
    }
}

fn main() {
    if let Err(err) = test() {
        println!("error {}", ErrorStack::new(&err).into_string())
    }
}

pub fn test() -> Result<(), DeckLinkError> {
    let decklinks = get_decklinks()?;
    println!("Detected {} decklinks", decklinks.len());
    for deck in &decklinks {
        println!("{:#?}", deck.info()?);
    }

    let decklink = &decklinks[0];

    let input = decklink.input()?;
    let (_is_supported, mode) = input.supports_video_mode(
        VideoConnection::HDMI,
        DisplayModeType::Mode4K2160p60,
        PixelFormat::Format8BitYUV,
        VideoInputConversionMode::NoVideoInputConversion,
        SupportedVideoModeFlags::default(),
    )?;
    println!("{mode:?}");
    input.enable_video(
        mode,
        PixelFormat::Format8BitYUV,
        VideoInputFlags {
            enable_format_detection: true,
            ..Default::default()
        },
    )?;
    input.enable_audio(48_000, AudioSampleType::Sample32bit, 2)?;
    Ok(())
}
