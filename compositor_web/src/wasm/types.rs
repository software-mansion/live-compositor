use std::time::Duration;

use compositor_render::{web_renderer::WebRendererInitOptions, InputId, Resolution};
use serde::{de::DeserializeOwned, Deserialize};
use wasm_bindgen::prelude::*;

#[derive(Debug, Deserialize)]
pub struct RendererOptions {
    framerate: Framerate,
    stream_fallback_timeout_ms: u64,
}

#[derive(Debug, Deserialize)]
pub struct Framerate {
    num: u32,
    den: u32,
}

#[wasm_bindgen]
pub struct InputFrameSet {
    pub pts_ms: f64,

    #[wasm_bindgen(skip)]
    pub frames: js_sys::Map,
}

#[wasm_bindgen]
impl InputFrameSet {
    #[wasm_bindgen(constructor)]
    pub fn new(pts_ms: f64, frames: js_sys::Map) -> Self {
        Self { pts_ms, frames }
    }

    #[wasm_bindgen(getter)]
    pub fn frames(&self) -> js_sys::Map {
        self.frames.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_frames(&mut self, frames: js_sys::Map) {
        self.frames = frames;
    }
}

pub struct InputFrame {
    pub id: InputId,
    pub resolution: Resolution,
    pub format: FrameFormat,
    pub data: Vec<u8>,
}

#[wasm_bindgen]
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FrameFormat {
    RgbaBytes,
}

impl From<FrameFormat> for compositor_render::OutputFrameFormat {
    fn from(value: FrameFormat) -> Self {
        match value {
            FrameFormat::RgbaBytes => compositor_render::OutputFrameFormat::RgbaWgpuTexture,
        }
    }
}

impl From<RendererOptions> for compositor_render::RendererOptions {
    fn from(value: RendererOptions) -> Self {
        Self {
            web_renderer: WebRendererInitOptions {
                enable: false,
                enable_gpu: false,
            },
            framerate: value.framerate.into(),
            stream_fallback_timeout: Duration::from_millis(value.stream_fallback_timeout_ms),
            force_gpu: false,
            wgpu_features: wgpu::Features::empty(),
            wgpu_ctx: None,
        }
    }
}

impl From<Framerate> for compositor_render::Framerate {
    fn from(framerate: Framerate) -> Self {
        Self {
            num: framerate.num,
            den: framerate.den,
        }
    }
}

impl TryFrom<JsValue> for InputFrame {
    type Error = JsValue;

    fn try_from(entry: JsValue) -> Result<Self, Self::Error> {
        // 0 - map key
        let id = js_sys::Reflect::get_u32(&entry, 0)?
            .as_string()
            .ok_or(JsValue::from_str("Expected string used as a key"))?;
        let id = InputId(id.into());

        // 1 - map value
        let value = js_sys::Reflect::get_u32(&entry, 1)?;
        let resolution =
            from_js_value::<compositor_api::types::Resolution>(value.get("resolution")?)?.into();
        let format: FrameFormat = from_js_value(value.get("format")?)?;
        let data: js_sys::Uint8ClampedArray = value.get("data")?.into();

        Ok(Self {
            id,
            resolution,
            format,
            data: data.to_vec(),
        })
    }
}

pub fn from_js_value<T: DeserializeOwned>(value: JsValue) -> Result<T, JsValue> {
    serde_wasm_bindgen::from_value(value).map_err(to_js_error)
}

pub fn to_js_error(error: impl std::error::Error) -> JsValue {
    JsValue::from_str(&error.to_string())
}

trait JsValueExt {
    fn get(&self, key: &str) -> Result<JsValue, JsValue>;
}

impl JsValueExt for JsValue {
    fn get(&self, key: &str) -> Result<JsValue, JsValue> {
        js_sys::Reflect::get(self, &JsValue::from_str(key))
    }
}
