use std::{
    collections::VecDeque,
    sync::{Arc, mpsc},
};

use crate::{
    app_config::AppConfigDir,
    parser::{self, parser::PassageLiteral},
    services::audio::{AudioRecorder, AudioSignal},
};

#[derive(Debug)]
pub enum TranscriptParserEvent {
    Error(String),
    End,
    Data(Vec<PassageLiteral>),
}

pub struct FixedQueue<T> {
    inner: VecDeque<T>,
    capacity: usize,
}

impl<T: std::clone::Clone> FixedQueue<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            inner: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    // Pushes an item. If full, drops the oldest item (FIFO)
    pub fn push(&mut self, item: T) {
        if self.inner.len() == self.capacity {
            self.inner.pop_front();
        }
        self.inner.push_back(item);
    }

    // pub fn pop(&mut self) -> Option<T> {
    //     self.inner.pop_front()
    // }
    //
    // pub fn len(&self) -> usize {
    //     self.inner.len()
    // }
    //
    // pub fn capacity(&self) -> usize {
    //     self.capacity
    // }

    pub fn join(&mut self, sep: &str) -> String
    where
        String: std::convert::From<T>,
    {
        let a = self
            .inner
            .iter()
            .map(|v| String::from(v.clone()))
            .collect::<Vec<_>>();
        a.join(sep)
    }
}

fn parse_text(transcript: &str) -> Vec<PassageLiteral> {
    let text = text2num::replace_numbers_in_text(transcript, &text2num::Language::english(), 0.0);
    let text = text.replace("1st", "1");
    let text = text.replace("2nd", "2");
    let text = text.replace("3rd", "3");

    println!(
        "[parsed] {:?}",
        parser::parser::Parser::parse_format(text.clone())
    );

    parser::parser::Parser::parse_passage(text)
}

const TARGET_RATE: u32 = 16_000;
const CHUNK_DURATION: f32 = 5.0;
const BUFFER_CAPACITY: usize = (TARGET_RATE as f32 * CHUNK_DURATION) as usize;

/// Speech to text
pub struct Stt {
    ctx: whisper_rs::WhisperContext,
}

impl Stt {
    pub fn new() -> anyhow::Result<Self> {
        whisper_rs::install_logging_hooks();
        let whisper_model_path = AppConfigDir::dir_path(AppConfigDir::Models).join("whisper.bin");
        let ctx = whisper_rs::WhisperContext::new_with_params(
            whisper_model_path,
            whisper_rs::WhisperContextParameters::default(),
        )
        .inspect_err(|err| println!("[error] {err:?}\nCould not create model context"))?;

        Ok(Self { ctx })
    }

    pub fn start(
        &self,
        // send transcript back
        ttx: async_channel::Sender<TranscriptParserEvent>,

        // handle transcript loop
        tx: mpsc::Sender<AudioSignal>,
        rx: mpsc::Receiver<AudioSignal>,
    ) -> anyhow::Result<mpsc::Sender<AudioSignal>> {
        let tx_clone = tx.clone();
        let recorder = Arc::new(AudioRecorder::new(tx_clone)?);
        recorder.start()?;

        let mut params = whisper_rs::FullParams::new(whisper_rs::SamplingStrategy::BeamSearch {
            beam_size: 5,
            patience: -1.0,
        });
        params.set_language(Some("en"));

        params.set_no_context(true);
        params.set_single_segment(true); // optional: treat each chunk as one segment, simpler output
        params.set_print_progress(false);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);
        // params.set_temperature(0.5);

        let sample_rate = recorder.sample_rate;
        println!("[sample_rate] {sample_rate}");
        println!("[TARGET_RATE] {TARGET_RATE}");
        println!("[CHUNK_SECS] {CHUNK_DURATION}");

        let mut buffer = Vec::with_capacity(BUFFER_CAPACITY);
        let mut state = self
            .ctx
            .create_state()
            .inspect_err(|err| println!("[error] {err:?}\nCould not create state"))?;

        let handle = std::thread::spawn(move || {
            let mut text_buffer = FixedQueue::new(5);
            while let Ok(audio_signal) = rx.recv() {
                let samples = match audio_signal {
                    AudioSignal::Stream(samples) => samples,
                    AudioSignal::Stop => {
                        println!("[stop]");
                        let _ = ttx.send_blocking(TranscriptParserEvent::End);
                        break;
                    }
                };
                let resampled = match samplerate::convert(
                    sample_rate,
                    TARGET_RATE,
                    1,
                    samplerate::ConverterType::SincBestQuality,
                    &samples,
                ) {
                    Ok(sample) => sample,
                    Err(err) => {
                        let err = format!("[error] {err:?}\nCould not resample");
                        println!("{err:?}");
                        let _ = ttx.send_blocking(TranscriptParserEvent::Error(err));
                        continue;
                    }
                };

                buffer.extend(&resampled);

                if buffer.len() < BUFFER_CAPACITY {
                    continue;
                }

                if let Err(err) = state.full(params.clone(), &buffer) {
                    let err = format!("[error] {err:?}\nFailed to run model");
                    println!("{err:?}");
                    let _ = ttx.send_blocking(TranscriptParserEvent::Error(err));
                    continue;
                };
                buffer.clear();
                for segment in state.as_iter() {
                    let transcript = segment.to_string();

                    println!(
                        "[{} - {}]: {}",
                        segment.start_timestamp(),
                        segment.end_timestamp(),
                        transcript
                    );
                    let transcript = transcript.to_string().trim_end().to_string();
                    let transcript = transcript.trim_end_matches(".").to_string();
                    text_buffer.push(transcript);
                    let res = parse_text(&text_buffer.join(" "));
                    if !res.is_empty() {
                        let _ = ttx.send_blocking(TranscriptParserEvent::Data(res));
                    }
                }
            }
        });
        handle.join().unwrap();

        //
        anyhow::Ok(tx)
    }
}
