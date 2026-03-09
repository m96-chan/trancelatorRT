use super::error::{AudioError, AudioResult};
use webrtc_vad::{SampleRate, Vad, VadMode as WebrtcVadMode};

#[derive(Debug, Clone, Copy)]
pub enum VadMode {
    Quality,
    LowBitrate,
    Aggressive,
    VeryAggressive,
}

pub struct VoiceActivityDetector {
    vad: Vad,
}

// Safety: Vad contains a raw pointer to C libfvad, but it is only accessed
// through &mut self methods, so single-threaded access via Mutex is safe.
unsafe impl Send for VoiceActivityDetector {}

impl VoiceActivityDetector {
    pub fn new(mode: VadMode, sample_rate_hz: u32) -> AudioResult<Self> {
        let sr = match sample_rate_hz {
            8000 => SampleRate::Rate8kHz,
            16000 => SampleRate::Rate16kHz,
            32000 => SampleRate::Rate32kHz,
            48000 => SampleRate::Rate48kHz,
            _ => {
                return Err(AudioError::VadError(format!(
                    "Unsupported sample rate: {}",
                    sample_rate_hz
                )))
            }
        };
        let webrtc_mode = match mode {
            VadMode::Quality => WebrtcVadMode::Quality,
            VadMode::LowBitrate => WebrtcVadMode::LowBitrate,
            VadMode::Aggressive => WebrtcVadMode::Aggressive,
            VadMode::VeryAggressive => WebrtcVadMode::VeryAggressive,
        };
        let vad = Vad::new_with_rate_and_mode(sr, webrtc_mode);
        Ok(Self { vad })
    }

    pub fn is_speech(&mut self, samples: &[i16]) -> AudioResult<bool> {
        self.vad
            .is_voice_segment(samples)
            .map_err(|_| AudioError::VadError("Invalid frame length".into()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeechEvent {
    None,
    SpeechStart,
    SpeechEnd,
}

pub struct SpeechSegmentTracker {
    speech_threshold: u32,
    silence_threshold: u32,
    consecutive_speech: u32,
    consecutive_silence: u32,
    in_speech: bool,
}

impl SpeechSegmentTracker {
    pub fn new(speech_threshold: u32, silence_threshold: u32) -> Self {
        Self {
            speech_threshold,
            silence_threshold,
            consecutive_speech: 0,
            consecutive_silence: 0,
            in_speech: false,
        }
    }

    pub fn update(&mut self, is_speech: bool) -> SpeechEvent {
        if is_speech {
            self.consecutive_speech += 1;
            self.consecutive_silence = 0;

            if !self.in_speech && self.consecutive_speech >= self.speech_threshold {
                self.in_speech = true;
                return SpeechEvent::SpeechStart;
            }
        } else {
            self.consecutive_silence += 1;
            self.consecutive_speech = 0;

            if self.in_speech && self.consecutive_silence >= self.silence_threshold {
                self.in_speech = false;
                return SpeechEvent::SpeechEnd;
            }
        }

        SpeechEvent::None
    }

    pub fn in_speech(&self) -> bool {
        self.in_speech
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_detects_silence() {
        let mut vad = VoiceActivityDetector::new(VadMode::VeryAggressive, 16000).unwrap();
        let silence = vec![0i16; 480];
        assert!(!vad.is_speech(&silence).unwrap());
    }

    #[test]
    fn test_vad_detects_speech_like_signal() {
        let mut vad = VoiceActivityDetector::new(VadMode::Quality, 16000).unwrap();
        let samples: Vec<i16> = (0..480)
            .map(|i| {
                let t = i as f32 / 16000.0;
                (f32::sin(2.0 * std::f32::consts::PI * 300.0 * t) * 30000.0) as i16
            })
            .collect();
        assert!(vad.is_speech(&samples).unwrap());
    }

    #[test]
    fn test_vad_invalid_frame_size() {
        let mut vad = VoiceActivityDetector::new(VadMode::Quality, 16000).unwrap();
        let bad_frame = vec![0i16; 100];
        assert!(vad.is_speech(&bad_frame).is_err());
    }

    #[test]
    fn test_vad_all_modes_construct() {
        for mode in [
            VadMode::Quality,
            VadMode::LowBitrate,
            VadMode::Aggressive,
            VadMode::VeryAggressive,
        ] {
            assert!(VoiceActivityDetector::new(mode, 16000).is_ok());
        }
    }

    #[test]
    fn test_speech_segment_tracker_basic() {
        let mut tracker = SpeechSegmentTracker::new(3, 5);

        assert_eq!(tracker.update(true), SpeechEvent::None);
        assert_eq!(tracker.update(true), SpeechEvent::None);
        assert_eq!(tracker.update(true), SpeechEvent::SpeechStart);

        assert_eq!(tracker.update(false), SpeechEvent::None);
        assert_eq!(tracker.update(false), SpeechEvent::None);
        assert_eq!(tracker.update(false), SpeechEvent::None);
        assert_eq!(tracker.update(false), SpeechEvent::None);
        assert_eq!(tracker.update(false), SpeechEvent::SpeechEnd);
    }

    #[test]
    fn test_speech_segment_tracker_intermittent_noise() {
        let mut tracker = SpeechSegmentTracker::new(3, 5);
        tracker.update(true);
        tracker.update(false);
        tracker.update(true);
        tracker.update(false);
        assert!(!tracker.in_speech());
    }
}
