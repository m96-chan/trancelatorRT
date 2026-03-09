use super::state::PipelineState;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AudioError {
    #[error("No input device available")]
    NoInputDevice,
    #[error("No output device available")]
    NoOutputDevice,
    #[error("Stream error: {0}")]
    StreamError(String),
    #[error("Invalid state transition: {from:?} -> {to:?}")]
    InvalidStateTransition {
        from: PipelineState,
        to: PipelineState,
    },
    #[error("VAD error: {0}")]
    VadError(String),
    #[error("Buffer overflow")]
    BufferOverflow,
    #[error("Permission denied: RECORD_AUDIO")]
    PermissionDenied,
}

pub type AudioResult<T> = Result<T, AudioError>;
