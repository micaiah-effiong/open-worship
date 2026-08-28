use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use anyhow::{Result, anyhow};
use cpal::{
    SampleFormat,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};
pub mod transcibe;

pub enum AudioSignal {
    Stream(Vec<f32>),
    Stop,
}

pub struct AudioRecorder {
    input_stream: cpal::Stream,
    output_stream: cpal::Stream,
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new(tx: std::sync::mpsc::Sender<AudioSignal>) -> Result<Self> {
        let host = cpal::default_host();

        let input_device = host
            .default_input_device()
            .ok_or_else(|| anyhow!("No microphone found"))?;
        let output_device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("No output found"))?;

        let input_config = Self::f32_stream_config(&input_device)?;
        let output_config = Self::f32_stream_config(&output_device)?;

        let sample_rate = input_config.sample_rate();
        let output_channels = output_config.channels();

        let sample_queue = Arc::new(Mutex::new(VecDeque::<f32>::new()));

        let input_stream = {
            let queue = sample_queue.clone();
            let tx_clone = tx.clone();
            input_device.build_input_stream(
                input_config.into(),
                move |data: &[f32], _| {
                    let _ = tx.send(AudioSignal::Stream(data.to_vec()));
                    queue.lock().unwrap().extend(data.iter());
                },
                move |err| {
                    eprintln!("Audio error: {err}");
                    let _ = tx_clone.send(AudioSignal::Stop);
                },
                None,
            )?
        };

        let output_stream = {
            let queue = sample_queue.clone();
            output_device.build_output_stream(
                output_config.into(),
                move |output: &mut [f32], _| {
                    let mut queue = queue.lock().unwrap();

                    for frames in output.chunks_mut(output_channels.into()) {
                        let sample = queue.pop_front().unwrap_or_default();
                        frames.fill(sample);
                    }
                },
                |err| eprintln!("Output error: {err}"),
                None,
            )?
        };

        Ok(Self {
            input_stream,
            output_stream,
            sample_queue,
            sample_rate,
        })
    }

    pub fn start(&self) -> Result<()> {
        self.input_stream.play()?;
        // self.output_stream.play()?;
        Ok(())
    }

    pub fn stop(self) {}

    fn f32_stream_config(device: &cpal::Device) -> Result<cpal::SupportedStreamConfig> {
        let config = device
            .default_input_config()
            .or_else(|_| device.default_output_config())?;

        let label = match device.supports_input() {
            true => "Input",
            false => "Output",
        };

        if config.sample_format() != SampleFormat::F32 {
            return Err(anyhow!(
                "{label} device \"{:?}\" does not support F32",
                device.id()
            ));
        }

        println!("{label} device: {:?}", device.id());
        println!("  sample rate: {}", config.sample_rate());
        println!("  channels: {}", config.channels());

        Ok(config)
    }
}

impl AudioRecorder {}
