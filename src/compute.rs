use candle_core::{DType, Device, IndexOp as _, Tensor};
use candle_transformers::models::encodec;

pub struct Compute {
    model: encodec::Model,
    device: Device,
}

impl Compute {
    pub fn new() -> anyhow::Result<Self> {
        let device = candle_core::Device::Cpu;
        let model_path = hf_hub::api::sync::Api::new()?
            .model("facebook/encodec_24khz".to_string())
            .get("model.safetensors")?;
        let vb = unsafe { candle_nn::VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)? };
        let config = encodec::Config::default();
        let model = encodec::Model::new(&config, vb)?;
        Ok(Self { model, device })
    }

    pub fn decode_codes(&self, code: u32) -> anyhow::Result<Tensor> {
        let code_tensor = Tensor::new(&[[[code]]], &self.device)?;
        // TODO: pad and such
        Ok(self.model.decode(&code_tensor)?.i(0)?.i(0)?)
    }

    // pub fn device(&self) -> &Device {
    //     &self.device
    // }
}
