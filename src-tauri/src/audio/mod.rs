pub mod buffer;
pub mod capture;
pub mod error;
pub mod playback;
pub mod state;
pub mod vad;

use error::AudioResult;
use state::{PipelineState, PipelineStateMachine};
use vad::{SpeechEvent, SpeechSegmentTracker, VadMode, VoiceActivityDetector};
use buffer::SpeechBuffer;
use capture::{AudioCapture, CaptureConfig};
use playback::AudioPlayback;
use parking_lot::Mutex;
use std::sync::{mpsc, Arc};

pub struct AudioPipeline {
    state: Arc<Mutex<PipelineStateMachine>>,
    capture: Mutex<AudioCapture>,
    playback: Mutex<AudioPlayback>,
    vad: Arc<Mutex<VoiceActivityDetector>>,
    segment_tracker: Arc<Mutex<SpeechSegmentTracker>>,
    speech_buffer: Arc<Mutex<SpeechBuffer>>,
    segment_sender: mpsc::Sender<Vec<i16>>,
}

impl AudioPipeline {
    pub fn new(
        config: CaptureConfig,
        segment_sender: mpsc::Sender<Vec<i16>>,
    ) -> AudioResult<Self> {
        let sample_rate = config.sample_rate;
        let vad = VoiceActivityDetector::new(VadMode::Aggressive, sample_rate)?;

        Ok(Self {
            state: Arc::new(Mutex::new(PipelineStateMachine::new())),
            capture: Mutex::new(AudioCapture::new(config)),
            playback: Mutex::new(AudioPlayback::new()),
            vad: Arc::new(Mutex::new(vad)),
            segment_tracker: Arc::new(Mutex::new(SpeechSegmentTracker::new(3, 15))),
            speech_buffer: Arc::new(Mutex::new(SpeechBuffer::new(16000 * 30))),
            segment_sender,
        })
    }

    pub fn start_recording(&self) -> AudioResult<()> {
        self.state.lock().transition(PipelineState::Recording)?;

        let vad = Arc::clone(&self.vad);
        let tracker = Arc::clone(&self.segment_tracker);
        let speech_buffer = Arc::clone(&self.speech_buffer);
        let sender = self.segment_sender.clone();

        self.capture.lock().start(move |data: &[i16]| {
            // Process in 480-sample frames (30ms at 16kHz)
            for chunk in data.chunks(480) {
                if chunk.len() < 480 {
                    continue;
                }

                let is_speech = vad.lock().is_speech(chunk).unwrap_or(false);
                let event = tracker.lock().update(is_speech);

                let mut sb = speech_buffer.lock();
                match event {
                    SpeechEvent::SpeechStart => {
                        sb.on_speech_start();
                        sb.push_frame(chunk);
                    }
                    SpeechEvent::SpeechEnd => {
                        sb.push_frame(chunk);
                        if let Some(segment) = sb.on_speech_end() {
                            let _ = sender.send(segment);
                        }
                    }
                    SpeechEvent::None => {
                        if tracker.lock().in_speech() {
                            sb.push_frame(chunk);
                        }
                    }
                }
            }
        })?;

        Ok(())
    }

    pub fn stop_recording(&self) -> AudioResult<()> {
        self.state.lock().transition(PipelineState::Idle)?;
        self.capture.lock().stop()?;
        Ok(())
    }

    pub fn pause(&self) -> AudioResult<()> {
        self.state.lock().transition(PipelineState::Paused)?;
        self.capture.lock().stop()?;
        Ok(())
    }

    pub fn resume(&self) -> AudioResult<()> {
        self.state.lock().transition(PipelineState::Recording)?;
        // Re-start capture would need callback re-setup
        // For now, transition state only
        Ok(())
    }

    pub fn play_audio(&self, samples: Vec<i16>, sample_rate: u32) -> AudioResult<()> {
        self.playback.lock().play(samples, sample_rate)
    }

    pub fn state(&self) -> PipelineState {
        self.state.lock().state()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_pipeline() -> (AudioPipeline, mpsc::Receiver<Vec<i16>>) {
        let (tx, rx) = mpsc::channel();
        let pipeline = AudioPipeline::new(CaptureConfig::default(), tx).unwrap();
        (pipeline, rx)
    }

    #[test]
    fn test_pipeline_initial_state() {
        let (pipeline, _rx) = create_test_pipeline();
        assert_eq!(pipeline.state(), PipelineState::Idle);
    }

    #[test]
    fn test_pipeline_invalid_stop_from_idle() {
        let (pipeline, _rx) = create_test_pipeline();
        assert!(pipeline.stop_recording().is_err());
    }

    #[test]
    fn test_pipeline_invalid_pause_from_idle() {
        let (pipeline, _rx) = create_test_pipeline();
        assert!(pipeline.pause().is_err());
    }
}
