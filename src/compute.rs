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
        let vb = unsafe {
            candle_nn::VarBuilder::from_mmaped_safetensors(&[model_path], DType::F32, &device)?
        };
        let config = encodec::Config::default();
        let model = encodec::Model::new(&config, vb)?;
        Ok(Self { model, device })
    }

    pub fn decode_codes(&self, code: u32) -> anyhow::Result<Vec<f32>> {
        // TODO: pad and such
        let code_tensor = Tensor::new(&[[[code, code, code, code]]], &self.device)?;
        let samples = self.model.decode(&code_tensor)?.i(0)?.i(0)?;
        const FRAGMENT_SIZE: usize = 320;
        Ok((0..FRAGMENT_SIZE)
            .map(|i| {
                let w = i as f32 / (FRAGMENT_SIZE as f32 - 1.0);
                Ok::<_, candle_core::Error>(
                    w * samples.i(FRAGMENT_SIZE + i)?.to_scalar::<f32>()?
                        + (1.0 - w) * samples.i(2 * FRAGMENT_SIZE + i)?.to_scalar::<f32>()?,
                )
            })
            .collect::<Result<Vec<_>, _>>()?)
    }
}
