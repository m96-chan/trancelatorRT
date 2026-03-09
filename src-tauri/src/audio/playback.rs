use super::error::{AudioError, AudioResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

pub struct AudioPlayback {
    stream: Option<cpal::Stream>,
    active: bool,
}

// Safety: cpal::Stream is not Send due to platform-specific internals,
// but we only access it through &mut self methods behind a Mutex.
unsafe impl Send for AudioPlayback {}

impl AudioPlayback {
    pub fn new() -> Self {
        Self {
            stream: None,
            active: false,
        }
    }

    pub fn play(&mut self, samples: Vec<i16>, sample_rate: u32) -> AudioResult<()> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or(AudioError::NoOutputDevice)?;

        let stream_config = cpal::StreamConfig {
            channels: 1,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        let samples = Arc::new(Mutex::new((samples, 0usize)));
        let samples_clone = Arc::clone(&samples);

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let mut guard = samples_clone.lock().unwrap();
                    let (ref buf, ref mut pos) = *guard;
                    for sample in data.iter_mut() {
                        if *pos < buf.len() {
                            *sample = buf[*pos];
                            *pos += 1;
                        } else {
                            *sample = 0;
                        }
                    }
                },
                |err| eprintln!("Playback error: {}", err),
                None,
            )
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        stream
            .play()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        self.stream = Some(stream);
        self.active = true;
        Ok(())
    }

    pub fn stop(&mut self) -> AudioResult<()> {
        self.stream = None;
        self.active = false;
        Ok(())
    }

    pub fn is_playing(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playback_initial_state() {
        let playback = AudioPlayback::new();
        assert!(!playback.is_playing());
    }
}
