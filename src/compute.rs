use candle_core::{DType, Device, IndexOp as _, Tensor};
use candle_transformers::models::encodec;
use eframe::wasm_bindgen::JsCast;
use gloo_utils::errors::JsError;
use js_sys::wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Response};

#[cfg(target_arch = "wasm32")]
fn into_jserr(v: JsValue) -> JsError {
    JsError::try_from(v).unwrap()
}

#[cfg(target_arch = "wasm32")]
async fn fetch(url: &str) -> anyhow::Result<Vec<u8>> {
    let response = JsFuture::from(web_sys::window().unwrap().fetch_with_str(url))
        .await
        .map_err(into_jserr)?
        .dyn_into::<Response>()
        .map_err(into_jserr)?;
    let blob = JsFuture::from(response.blob().map_err(into_jserr)?)
        .await
        .map_err(into_jserr)?
        .dyn_into::<Blob>()
        .map_err(into_jserr)?;
    let buffer = JsFuture::from(blob.array_buffer())
        .await
        .map_err(into_jserr)?
        .dyn_into::<js_sys::ArrayBuffer>()
        .map_err(into_jserr)?;
    Ok(js_sys::Uint8Array::new(&buffer).to_vec())
}

pub struct Compute {
    model: encodec::Model,
    device: Device,
}

impl Compute {
    pub async fn new() -> anyhow::Result<Self> {
        let device = candle_core::Device::Cpu;
        let model_path = "model.safetensors";
        #[cfg(target_arch = "wasm32")]
        let vb = candle_nn::VarBuilder::from_buffered_safetensors(
            fetch(model_path).await?,
            DType::F32,
            &device,
        )?;
        #[cfg(not(target_arch = "wasm32"))]
        let vb = unsafe {
            let model_path = hf_hub::api::sync::Api::new()?
                .model("facebook/encodec_24khz".to_string())
                .get("model.safetensors")?;
            candle_nn::VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)?
        };
        let config = encodec::Config::default();
        let model = encodec::Model::new(&config, vb)?;
        Ok(Self { model, device })
    }

    pub fn decode_codes(&self, codes: &Tensor) -> anyhow::Result<Vec<f32>> {
        assert!(codes.dtype() == DType::U32);
        assert!(codes.shape().dims2().is_ok());
        // TODO: perhaps we don't need to concat all of the fragments? just the edges?
        let code_tensor = Tensor::cat(&[codes, codes, codes, codes], 1)?.unsqueeze(0)?;
        let all_samples = self.model.decode(&code_tensor)?.i(0)?.i(0)?;
        const FRAGMENT_SIZE: usize = 320;
        let buffer_size = FRAGMENT_SIZE * codes.shape().dims2()?.1;
        let weights = Tensor::from_vec(
            (0..buffer_size)
                .map(|i| i as f32 / (buffer_size as f32 - 1.0))
                .collect(),
            (buffer_size,),
            &self.device,
        )?;
        let samples = ((all_samples.i(buffer_size..(2 * buffer_size))? * &weights)?
            + (all_samples.i((2 * buffer_size)..(3 * buffer_size))? * (1.0 - weights))?)?;
        let mean = samples.mean_all()?;
        let dc0 = (samples.broadcast_sub(&mean))?;
        Ok(dc0.to_vec1()?)
    }

    pub fn device(&self) -> &Device {
        &self.device
    }
}
