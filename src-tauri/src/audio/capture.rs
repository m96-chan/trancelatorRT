use super::error::{AudioError, AudioResult};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SampleFormat;

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

/// Downsample from `src_rate` to `dst_rate` using linear interpolation.
fn resample(samples: &[i16], src_rate: u32, dst_rate: u32) -> Vec<i16> {
    if src_rate == dst_rate {
        return samples.to_vec();
    }
    let ratio = src_rate as f64 / dst_rate as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_pos = i as f64 * ratio;
        let idx = src_pos as usize;
        let frac = src_pos - idx as f64;
        let s0 = samples[idx] as f64;
        let s1 = if idx + 1 < samples.len() {
            samples[idx + 1] as f64
        } else {
            s0
        };
        output.push((s0 + frac * (s1 - s0)) as i16);
    }
    output
}

/// Convert multi-channel audio to mono by averaging channels.
fn to_mono(samples: &[i16], channels: u16) -> Vec<i16> {
    if channels <= 1 {
        return samples.to_vec();
    }
    let ch = channels as usize;
    samples
        .chunks(ch)
        .map(|frame| {
            let sum: i32 = frame.iter().map(|&s| s as i32).sum();
            (sum / ch as i32) as i16
        })
        .collect()
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

        let default_config = device
            .default_input_config()
            .map_err(|e| AudioError::StreamError(e.to_string()))?;

        let sample_format = default_config.sample_format();
        let device_rate = default_config.sample_rate().0;
        let device_channels = default_config.channels();
        let target_rate = self.config.sample_rate;
        let stream_config: cpal::StreamConfig = default_config.into();

        eprintln!(
            "[capture] Device: {}Hz {}ch {:?} → target {}Hz mono",
            device_rate, device_channels, sample_format, target_rate
        );

        let err_callback = |err: cpal::StreamError| {
            eprintln!("Audio stream error: {}", err);
        };

        // Closure to convert device audio to 16kHz mono i16
        let process_audio = move |mono_i16: Vec<i16>| {
            let resampled = resample(&mono_i16, device_rate, target_rate);
            callback(&resampled);
        };

        // Build stream matching the device's native sample format
        let stream = match sample_format {
            SampleFormat::I16 => {
                let mut process = process_audio;
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let mono = to_mono(data, device_channels);
                            process(mono);
                        },
                        err_callback,
                        None,
                    )
                    .map_err(|e| AudioError::StreamError(e.to_string()))?
            }
            SampleFormat::F32 => {
                let mut process = process_audio;
                device
                    .build_input_stream(
                        &stream_config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let i16_data: Vec<i16> = data
                                .iter()
                                .map(|&s| (s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                                .collect();
                            let mono = to_mono(&i16_data, device_channels);
                            process(mono);
                        },
                        err_callback,
                        None,
                    )
                    .map_err(|e| AudioError::StreamError(e.to_string()))?
            }
            _ => {
                return Err(AudioError::StreamError(format!(
                    "Unsupported sample format: {:?}",
                    sample_format
                )));
            }
        };

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
