use std::sync::Arc;

use anyhow::{anyhow, Result};
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    BufferSize, Device, OutputCallbackInfo, SampleFormat, Stream, SupportedBufferSize,
    SupportedStreamConfigRange,
};
use crossbeam::atomic::AtomicCell;
use log::warn;

const NUM_CHANNELS: usize = 2;

pub trait Synth {
    fn play(&self, sample_rate: u32, channels: usize, out_samples: &mut [f32]);
}

pub struct AudioManager {
    device: Option<Device>,
    config_range: Option<SupportedStreamConfigRange>,
    buffer_size: Arc<AtomicCell<u32>>,
    forced_buffer_size: Option<u32>,
    stream: Option<Stream>,
    error_callback: Arc<Box<dyn Fn(String) + Send + Sync>>,
    synth: Arc<dyn Synth + Send + Sync>,
}

impl AudioManager
{
    pub fn new<U>(synth: Arc<dyn Synth + Send + Sync>, error_callback: U) -> Self
    where
        U: Fn(String) + Send + Sync + 'static,
    {
        let mut s = Self {
            device: None,
            config_range: None,
            buffer_size: Arc::new(AtomicCell::new(0)),
            forced_buffer_size: None,
            stream: None,
            error_callback: Arc::new(Box::new(error_callback)),
            synth,
        };
        s.setup();
        s
    }

    pub fn get_devices(&self) -> Vec<Device> {
        let host = cpal::default_host();
        match host.output_devices() {
            Ok(devices) => devices.collect(),
            Err(_) => vec![],
        }
    }

    pub fn set_device(&mut self, device: Device) {
        if self.device.as_ref().and_then(|d| d.name().ok()) != device.name().ok() {
            self.stream = None;
            self.config_range = None;
            self.device = Some(device);
            self.setup();
        }
    }

    fn setup(&mut self) {
        self.stream = None;
        let r = (|| -> Result<_> {
            if self.device.is_none() {
                let host = cpal::default_host();
                self.device = host.default_output_device();
                self.config_range = None;
            }
            if let Some(ref device) = self.device {
                if self.config_range.is_none() {
                    self.config_range = Some(
                        device
                            .supported_output_configs()?
                            // just pick the first valid config
                            .find(|config| {
                                // only stereo configs
                                config.sample_format() == SampleFormat::F32
                                    && config.channels() == 2
                            })
                            .ok_or_else(|| anyhow!("no valid output audio config found"))?,
                    );
                }
                if let Some(ref supported_config) = self.config_range {
                    let sample_rate = device.default_output_config()?.sample_rate().clamp(
                        supported_config.min_sample_rate(),
                        supported_config.max_sample_rate(),
                    );
                    let mut config = supported_config
                        .clone()
                        .with_sample_rate(sample_rate)
                        .config();
                    if let SupportedBufferSize::Range { min, max } = supported_config.buffer_size()
                    {
                        match self.forced_buffer_size {
                            Some(size) => {
                                config.buffer_size = BufferSize::Fixed(size.clamp(*min, *max));
                            }
                            None => {
                                config.buffer_size = BufferSize::Default;
                            }
                        }
                    }
                    let sample_rate = sample_rate.0;
                    let channels = config.channels.into();
                    let synth = self.synth.clone();
                    let error_callback = self.error_callback.clone();
                    let buffer_size = self.buffer_size.clone();
                    let stream = device.build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &OutputCallbackInfo| {
                            buffer_size.store((data.len() / channels) as u32);
                            synth.play(sample_rate, channels, data);
                        },
                        move |error| {
                            error_callback(format!("error: {:?}", error));
                        },
                        // no timeout
                        None,
                    )?;
                    stream.play()?;
                    self.stream = Some(stream);
                }
            } else {
                warn!("no output device found");
            }
            Ok(())
        })();
        if let Err(e) = r {
            (self.error_callback)(format!("error: {:?}", e));
        }
    }

    pub fn get_name(&self) -> Option<String> {
        self.device.as_ref()?.name().ok()
    }

    pub fn get_buffer_size(&self) -> Option<u32> {
        match self.buffer_size.load() {
            0 => None,
            n => Some(n),
        }
    }

    pub fn get_buffer_size_range(&self) -> Option<(u32, u32)> {
        match self.config_range.as_ref()?.buffer_size() {
            SupportedBufferSize::Range { min, max } => Some((*min, *max)),
            SupportedBufferSize::Unknown => None,
        }
    }

    pub fn set_forced_buffer_size(&mut self, buffer_size: Option<u32>) {
        if self.forced_buffer_size != buffer_size {
            self.forced_buffer_size = buffer_size;
            self.setup();
        }
    }
}
