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

    pub fn decode_codes(&self, codes: &[u32]) -> anyhow::Result<Vec<f32>> {
        const SEQUENCE_LEN: usize = 4;
        let mut v = Vec::with_capacity(codes.len() * SEQUENCE_LEN);
        for b in 0..codes.len() {
            for _ in 0..SEQUENCE_LEN {
                v.push(codes[b]);
            }
        }
        let code_tensor = Tensor::from_vec(v, (1, codes.len(), SEQUENCE_LEN), &self.device)?;
        let all_samples = self.model.decode(&code_tensor)?.i(0)?.i(0)?;
        const FRAGMENT_SIZE: usize = 320;
        let weights = Tensor::from_vec(
            (0..FRAGMENT_SIZE)
                .map(|i| i as f32 / (FRAGMENT_SIZE as f32 - 1.0))
                .collect(),
            (FRAGMENT_SIZE,),
            &self.device,
        )?;
        let samples = ((all_samples.i(FRAGMENT_SIZE..(2 * FRAGMENT_SIZE))? * &weights)?
            + (all_samples.i((2 * FRAGMENT_SIZE)..(3 * FRAGMENT_SIZE))? * (1.0 - weights))?)?;
        let mean = samples.mean_all()?;
        //mean.to_scalar::<f32>().unwrap();
        let dc0 = (samples.broadcast_sub(&mean))?;
        Ok(dc0.to_vec1()?)
        // let samples = (0..FRAGMENT_SIZE)
        //     .map(|i| {
        //         let w = i as f32 / (FRAGMENT_SIZE as f32 - 1.0);
        //         Ok::<_, candle_core::Error>(
        //             w * all_samples.i(FRAGMENT_SIZE + i)?.to_scalar::<f32>()?
        //                 + (1.0 - w) * all_samples.i(2 * FRAGMENT_SIZE + i)?.to_scalar::<f32>()?,
        //         )
        //     })
        //     .collect::<Result<Vec<_>, _>>()?;
    }
}
