use super::error::{AudioError, AudioResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum PipelineState {
    Idle,
    Recording,
    Paused,
    Processing,
}

pub struct PipelineStateMachine {
    state: PipelineState,
}

impl PipelineStateMachine {
    pub fn new() -> Self {
        Self {
            state: PipelineState::Idle,
        }
    }

    pub fn state(&self) -> PipelineState {
        self.state
    }

    pub fn transition(&mut self, to: PipelineState) -> AudioResult<()> {
        let valid = matches!(
            (self.state, to),
            (PipelineState::Idle, PipelineState::Recording)
                | (PipelineState::Recording, PipelineState::Processing)
                | (PipelineState::Recording, PipelineState::Paused)
                | (PipelineState::Recording, PipelineState::Idle)
                | (PipelineState::Paused, PipelineState::Recording)
                | (PipelineState::Paused, PipelineState::Idle)
                | (PipelineState::Processing, PipelineState::Idle)
        );

        if valid {
            self.state = to;
            Ok(())
        } else {
            Err(AudioError::InvalidStateTransition {
                from: self.state,
                to,
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initial_state_is_idle() {
        let sm = PipelineStateMachine::new();
        assert_eq!(sm.state(), PipelineState::Idle);
    }

    #[test]
    fn test_idle_to_recording() {
        let mut sm = PipelineStateMachine::new();
        assert!(sm.transition(PipelineState::Recording).is_ok());
        assert_eq!(sm.state(), PipelineState::Recording);
    }

    #[test]
    fn test_recording_to_processing() {
        let mut sm = PipelineStateMachine::new();
        sm.transition(PipelineState::Recording).unwrap();
        assert!(sm.transition(PipelineState::Processing).is_ok());
    }

    #[test]
    fn test_recording_to_paused() {
        let mut sm = PipelineStateMachine::new();
        sm.transition(PipelineState::Recording).unwrap();
        assert!(sm.transition(PipelineState::Paused).is_ok());
    }

    #[test]
    fn test_invalid_transition_idle_to_processing() {
        let mut sm = PipelineStateMachine::new();
        assert!(sm.transition(PipelineState::Processing).is_err());
    }

    #[test]
    fn test_invalid_transition_idle_to_paused() {
        let mut sm = PipelineStateMachine::new();
        assert!(sm.transition(PipelineState::Paused).is_err());
    }

    #[test]
    fn test_paused_to_recording() {
        let mut sm = PipelineStateMachine::new();
        sm.transition(PipelineState::Recording).unwrap();
        sm.transition(PipelineState::Paused).unwrap();
        assert!(sm.transition(PipelineState::Recording).is_ok());
    }

    #[test]
    fn test_paused_to_idle() {
        let mut sm = PipelineStateMachine::new();
        sm.transition(PipelineState::Recording).unwrap();
        sm.transition(PipelineState::Paused).unwrap();
        assert!(sm.transition(PipelineState::Idle).is_ok());
    }

    #[test]
    fn test_processing_to_idle() {
        let mut sm = PipelineStateMachine::new();
        sm.transition(PipelineState::Recording).unwrap();
        sm.transition(PipelineState::Processing).unwrap();
        assert!(sm.transition(PipelineState::Idle).is_ok());
    }
}
