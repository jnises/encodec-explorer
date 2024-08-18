#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! hf-hub = "0.3.2"
//! ```

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
}
