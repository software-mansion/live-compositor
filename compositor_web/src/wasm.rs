use std::sync::{Arc, Mutex};

use bytes::Bytes;
use compositor_api::types as api;
use compositor_render::{
    image::{ImageSource, ImageType},
    InputId, OutputFrameFormat, OutputId, RegistryType, Renderer, RendererId, RendererSpec,
};
use glyphon::fontdb::Source;
use input_uploader::InputUploader;
use output_downloader::OutputDownloader;
use tracing::error;
use types::to_js_error;
use wasm_bindgen::prelude::*;
use wgpu::create_wgpu_context;

mod input_uploader;
mod output_downloader;
mod types;
mod wgpu;

// Executed during WASM module init
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();

    Ok(())
}

#[wasm_bindgen]
pub async fn create_renderer(options: JsValue) -> Result<LiveCompositorRenderer, JsValue> {
    let options = types::from_js_value::<types::RendererOptions>(options)?;
    // This option will only be respected for the first renderer
    let _ = wasm_log::try_init(wasm_log::Config::new(options.logger_level.into()));

    let (device, queue) = create_wgpu_context().await?;

    let (renderer, _) = Renderer::new(compositor_render::RendererOptions {
        wgpu_ctx: Some((device, queue)),
        ..options.into()
    })
    .map_err(to_js_error)?;
    let input_uploader = InputUploader::default();
    let output_downloader = OutputDownloader::default();

    let inner = InnerLiveCompositorRenderer {
        renderer,
        input_uploader,
        output_downloader,
    };

    // return Ok(inner);
    Ok(LiveCompositorRenderer(Mutex::new(inner)))
}

#[wasm_bindgen]
pub struct LiveCompositorRenderer(Mutex<InnerLiveCompositorRenderer>);

#[wasm_bindgen]
pub struct InnerLiveCompositorRenderer {
    renderer: Renderer,
    input_uploader: InputUploader,
    output_downloader: OutputDownloader,
}

#[wasm_bindgen]
impl LiveCompositorRenderer {
    pub fn render(&self, input: types::FrameSet) -> Result<types::FrameSet, JsValue> {
        let mut inner = self.0.lock().unwrap();
        inner.render(input)
    }

    pub fn update_scene(
        &self,
        output_id: String,
        resolution: JsValue,
        scene: JsValue,
    ) -> Result<(), JsValue> {
        let mut inner = self.0.lock().unwrap();
        inner.update_scene(output_id, resolution, scene)
    }

    pub fn register_input(&self, input_id: String) {
        let mut inner = self.0.lock().unwrap();
        inner.register_input(input_id)
    }

    pub async fn register_image(
        &self,
        renderer_id: String,
        image_spec: JsValue,
    ) -> Result<(), JsValue> {
        let mut inner = self.0.lock().unwrap();
        inner.register_image(renderer_id, image_spec).await
    }

    pub async fn register_font(&self, font_url: String) -> Result<(), JsValue> {
        let mut inner = self.0.lock().unwrap();
        inner.register_font(font_url).await
    }

    pub fn unregister_input(&self, input_id: String) {
        let mut inner = self.0.lock().unwrap();
        inner.unregister_input(input_id)
    }

    pub fn unregister_output(&self, output_id: String) {
        let mut inner = self.0.lock().unwrap();
        inner.unregister_output(output_id)
    }

    pub fn unregister_image(&self, renderer_id: String) -> Result<(), JsValue> {
        let mut inner = self.0.lock().unwrap();
        inner.unregister_image(renderer_id)
    }
}

impl Drop for InnerLiveCompositorRenderer {
    fn drop(&mut self) {
        error!("DROP");
    }
}

#[wasm_bindgen]
impl InnerLiveCompositorRenderer {
    pub fn render(&mut self, input: types::FrameSet) -> Result<types::FrameSet, JsValue> {
        let (device, queue) = self.renderer.wgpu_ctx();
        let frame_set = self.input_uploader.upload(&device, &queue, input)?;

        let outputs = self.renderer.render(frame_set).map_err(to_js_error)?;
        self.output_downloader
            .download_outputs(&device, &queue, outputs)
    }

    pub fn update_scene(
        &mut self,
        output_id: String,
        resolution: JsValue,
        scene: JsValue,
    ) -> Result<(), JsValue> {
        let resolution = types::from_js_value::<api::Resolution>(resolution)?;
        let scene = types::from_js_value::<api::Component>(scene)?;

        self.renderer
            .update_scene(
                OutputId(output_id.into()),
                resolution.into(),
                OutputFrameFormat::RgbaWgpuTexture,
                scene.try_into().map_err(to_js_error)?,
            )
            .map_err(to_js_error)
    }

    pub fn register_input(&mut self, input_id: String) {
        self.renderer.register_input(InputId(input_id.into()));
    }

    pub async fn register_image(
        &mut self,
        renderer_id: String,
        image_spec: JsValue,
    ) -> Result<(), JsValue> {
        let image_spec = types::from_js_value::<api::ImageSpec>(image_spec)?;
        let (url, image_type) = match image_spec {
            api::ImageSpec::Png { url, .. } => (url, ImageType::Png),
            api::ImageSpec::Jpeg { url, .. } => (url, ImageType::Jpeg),
            api::ImageSpec::Svg {
                url, resolution, ..
            } => (
                url,
                ImageType::Svg {
                    resolution: resolution.map(Into::into),
                },
            ),
            api::ImageSpec::Gif { url, .. } => (url, ImageType::Gif),
        };

        let Some(url) = url else {
            return Err(JsValue::from_str("Expected `url` field in image spec"));
        };

        let bytes = download(&url).await?;
        let renderer_spec = RendererSpec::Image(compositor_render::image::ImageSpec {
            src: ImageSource::Bytes { bytes },
            image_type,
        });
        self.renderer
            .register_renderer(RendererId(renderer_id.into()), renderer_spec)
            .map_err(to_js_error)
    }

    pub async fn register_font(&mut self, font_url: String) -> Result<(), JsValue> {
        let bytes = download(&font_url).await?;
        self.renderer
            .register_font(Source::Binary(Arc::new(bytes.to_vec())));
        Ok(())
    }

    pub fn unregister_input(&mut self, input_id: String) {
        let input_id = InputId(input_id.into());
        self.renderer.unregister_input(&input_id);
        self.input_uploader.remove_input(&input_id);
    }

    pub fn unregister_output(&mut self, output_id: String) {
        let output_id = OutputId(output_id.into());
        self.renderer.unregister_output(&output_id);
        self.output_downloader.remove_output(&output_id);
    }

    pub fn unregister_image(&mut self, renderer_id: String) -> Result<(), JsValue> {
        self.renderer
            .unregister_renderer(&RendererId(renderer_id.into()), RegistryType::Image)
            .map_err(to_js_error)
    }
}

async fn download(url: &str) -> Result<Bytes, JsValue> {
    let resp = reqwest::get(url).await.map_err(to_js_error)?;
    let resp = resp.error_for_status().map_err(to_js_error)?;
    resp.bytes().await.map_err(to_js_error)
}
