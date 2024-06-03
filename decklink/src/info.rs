use crate::{
    api::{device::DeckLinkConfiguration, profile::ProfileAttributes},
    enums::{
        self,
        ffi::{FlagAttributeId, IntegerAttributeId, IntegerConfigurationId, StringAttributeId},
    },
    DeckLink, DeckLinkError, VideoIOSupport, VideoInputConversionMode,
};

#[derive(Debug)]
pub struct DeckLinkInfo {
    pub current_profile: ProfileAttributesInfo,
    pub profiles: Vec<ProfileInfo>,
    pub configuration: ConfigurationInfo,
}

impl DeckLink {
    pub fn info(&self) -> Result<DeckLinkInfo, DeckLinkError> {
        let profiles = match self.profile_manager()? {
            Some(manager) => manager
                .profiles()?
                .into_iter()
                .map(|profile| -> Result<ProfileInfo, DeckLinkError> {
                    Ok(ProfileInfo {
                        is_active: profile.is_active()?,
                        attributes: profile.attributes()?.info()?,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
            None => vec![],
        };
        let current_profile = self.profile_attributes()?.info()?;
        let configuration = self.configuration()?.info()?;
        Ok(DeckLinkInfo {
            current_profile,
            profiles,
            configuration,
        })
    }
}

#[derive(Debug)]
pub struct ProfileInfo {
    pub is_active: bool,
    pub attributes: ProfileAttributesInfo,
}

#[derive(Debug)]
pub struct ProfileAttributesInfo {
    pub video_io_support: Option<VideoIOSupport>,
    pub model_name: Option<String>,
    pub vendor_name: Option<String>,
    pub display_name: Option<String>,
    pub device_handle: Option<String>,
    pub ethernet_mac_address: Option<String>,
    // throws 0x80000003 E_INVALIDARG
    // pub serial_port_device_name: Option<String>,
    pub profile_id: Option<i64>,
    pub max_audio_channels: Option<i64>,

    // throws 0x80000003 E_INVALIDARG
    // pub max_hdmi_audio_channels: Option<i64>,
    pub number_of_subdevices: Option<i64>,

    pub subdevice_index: Option<i64>,
    pub persistent_id: Option<i64>,
    pub device_group_id: Option<i64>,
    pub topological_id: Option<i64>,

    pub supports_input_format_detection: Option<bool>,
    pub has_serial_port: Option<bool>,
}

impl ProfileAttributes {
    pub fn info(&self) -> Result<ProfileAttributesInfo, DeckLinkError> {
        Ok(ProfileAttributesInfo {
            video_io_support: self
                .get_integer(IntegerAttributeId::VideoIOSupport)?
                .map(From::from),
            model_name: self.get_string(StringAttributeId::ModelName)?,
            vendor_name: self.get_string(StringAttributeId::VendorName)?,
            display_name: self.get_string(StringAttributeId::DisplayName)?,
            device_handle: self.get_string(StringAttributeId::DeviceHandle)?,
            ethernet_mac_address: self.get_string(StringAttributeId::EthernetMACAddress)?,

            profile_id: self.get_integer(IntegerAttributeId::ProfileID)?,
            max_audio_channels: self.get_integer(IntegerAttributeId::MaximumAudioChannels)?,
            number_of_subdevices: self.get_integer(IntegerAttributeId::NumberOfSubDevices)?,
            subdevice_index: self.get_integer(IntegerAttributeId::SubDeviceIndex)?,
            persistent_id: self.get_integer(IntegerAttributeId::PersistentID)?,
            device_group_id: self.get_integer(IntegerAttributeId::DeviceGroupID)?,
            topological_id: self.get_integer(IntegerAttributeId::TopologicalID)?,

            supports_input_format_detection: self
                .get_flag(FlagAttributeId::SupportsInputFormatDetection)?,
            has_serial_port: self.get_flag(FlagAttributeId::HasSerialPort)?,
        })
    }
}

#[derive(Debug)]
pub struct ConfigurationInfo {
    pub video_input_conversion_mode: Option<VideoInputConversionMode>,
}

impl DeckLinkConfiguration {
    pub fn info(&self) -> Result<ConfigurationInfo, DeckLinkError> {
        Ok(ConfigurationInfo {
            video_input_conversion_mode: self
                .get_integer(IntegerConfigurationId::ConfigVideoInputConversionMode)?
                .map(|value| enums::ffi::into_video_input_conversion_mode(value as u32)),
        })
    }
}
