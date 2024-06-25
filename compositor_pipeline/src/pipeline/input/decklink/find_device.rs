use decklink::{
    get_decklinks, FlagAttributeId, IntegerAttributeId, StringAttributeId, VideoIOSupport,
};

use super::{DeckLinkError, DeckLinkOptions};

pub(super) fn find_decklink(opts: &DeckLinkOptions) -> Result<decklink::DeckLink, DeckLinkError> {
    let decklinks = get_decklinks()?;

    for mut decklink in decklinks.into_iter() {
        if is_selected_decklink(opts, &mut decklink)? {
            return Ok(decklink);
        }
    }
    Err(DeckLinkError::NoMatchingDeckLink)
}

fn is_selected_decklink(
    opts: &DeckLinkOptions,
    decklink: &mut decklink::DeckLink,
) -> Result<bool, DeckLinkError> {
    let attr = decklink.profile_attributes()?;

    if let Some(subdevice) = opts.subdevice_index {
        if attr.get_integer(IntegerAttributeId::SubDeviceIndex)? != Some(subdevice.into()) {
            return Ok(false);
        }
    }

    if let Some(display_name) = &opts.display_name {
        if attr.get_string(StringAttributeId::DisplayName)?.as_ref() != Some(display_name) {
            return Ok(false);
        }
    }

    let video_io_support = VideoIOSupport::from(
        attr.get_integer(IntegerAttributeId::VideoIOSupport)?
            .ok_or(DeckLinkError::NoCaptureSupport)?,
    );
    if !video_io_support.capture {
        return Err(DeckLinkError::NoCaptureSupport);
    }

    if attr.get_flag(FlagAttributeId::SupportsInputFormatDetection)? != Some(true) {
        return Err(DeckLinkError::NoInputFormatDetection);
    }

    Ok(true)
}
