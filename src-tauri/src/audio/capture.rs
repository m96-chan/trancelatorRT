use super::error::{AudioError, AudioResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

pub struct CaptureConfig {
    pub sample_rate: u32,
    pub channels: u16,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            sample_rate: 16000,
            channels: 1,
        }
    }
}

pub struct AudioCapture {
    stream: Option<cpal::Stream>,
    config: CaptureConfig,
    active: bool,
}

// Safety: cpal::Stream is not Send due to platform-specific internals,
// but we only access it through &mut self methods behind a Mutex.
unsafe impl Send for AudioCapture {}

impl AudioCapture {
    pub fn new(config: CaptureConfig) -> Self {
        Self {
            stream: None,
            config,
            active: false,
        }
    }

    pub fn start<F>(&mut self, mut callback: F) -> AudioResult<()>
    where
        F: FnMut(&[i16]) + Send + 'static,
    {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AudioError::NoInputDevice)?;

        // Try requested config first, fall back to device default
        let stream_config = {
            let requested = cpal::StreamConfig {
                channels: self.config.channels,
                sample_rate: cpal::SampleRate(self.config.sample_rate),
                buffer_size: cpal::BufferSize::Default,
            };
            // Check if the device supports our requested config
            match device.supported_input_configs() {
                Ok(configs) => {
                    let supported = configs.into_iter().any(|c| {
                        c.channels() >= self.config.channels
                            && c.min_sample_rate().0 <= self.config.sample_rate
                            && c.max_sample_rate().0 >= self.config.sample_rate
                    });
                    if supported {
                        requested
                    } else {
                        // Use device default config
                        device
                            .default_input_config()
                            .map(|c| c.into())
                            .unwrap_or(requested)
                    }
                }
                Err(_) => requested,
            }
        };

        let err_callback = |err: cpal::StreamError| {
            eprintln!("Audio stream error: {}", err);
        };

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    callback(data);
                },
                err_callback,
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

    pub fn sample_rate(&self) -> u32 {
        self.config.sample_rate
    }

    pub fn is_active(&self) -> bool {
        self.active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CaptureConfig::default();
        assert_eq!(config.sample_rate, 16000);
        assert_eq!(config.channels, 1);
    }

    #[test]
    fn test_capture_initial_state() {
        let capture = AudioCapture::new(CaptureConfig::default());
        assert!(!capture.is_active());
        assert_eq!(capture.sample_rate(), 16000);
    }
}
