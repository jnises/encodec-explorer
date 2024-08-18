#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! hf-hub = "0.3.2"
//! #safetensors = "0.4"
//! #memmap2 = "0.9"
//! ```

// use memmap2::MmapOptions;
// use safetensors::SafeTensors;
// use std::fs::File;
// use std::path::Path;

fn main() {
    let model_path = hf_hub::api::sync::Api::new()
        .unwrap()
        .model("facebook/encodec_24khz".to_string())
        .get("model.safetensors")
        .unwrap();
    // TODO: strip out the encoder bits
    std::fs::copy(
        &model_path,
        std::path::Path::new(&std::env::var("TRUNK_STAGING_DIR").unwrap()).join("model.safetensors"),
    )
    .unwrap();
    // let file = File::open(&model_path).unwrap();
    // let buffer = unsafe { MmapOptions::new().map(&file).unwrap() };
    // let tensors = SafeTensors::deserialize(&buffer).unwrap();
    // let filter_fn = |n: &str| !n.starts_with("encoder");
    // let filtered: Vec<_> = tensors
    //     .names()
    //     .into_iter()
    //     .filter(|n| filter_fn(n.as_str()))
    //     .collect();
    // let mut meta = SafeTensors::read_metadata(&buffer)
    //     .unwrap()
    //     .1
    //     .metadata()
    //     .clone()
    //     .unwrap();
    // meta.retain(|k, _| filter_fn(k.as_str()));
    // safetensors::serialize_to_file(
    //     filtered.iter().map(|n| (n, tensors.tensor(n).unwrap())),
    //     &Some(meta),
    //     &Path::new("decoder_only.safetensors"),
    // )
    // .unwrap();
}
