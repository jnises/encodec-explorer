fn main() {
    // // TODO: only do this when building for wasm
    // let model_path = hf_hub::api::sync::Api::new()
    //     .unwrap()
    //     .model("facebook/encodec_24khz".to_string())
    //     .get("model.safetensors")
    //     .unwrap();
    // // TODO: check timestamps before copying
    // // TODO: strip out the encoder bits
    // std::fs::copy(&model_path, "model.safetensors").unwrap();
}
