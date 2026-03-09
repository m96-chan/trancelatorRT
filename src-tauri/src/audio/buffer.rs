pub struct AudioRingBuffer {
    buffer: Vec<i16>,
    capacity: usize,
    write_pos: usize,
    read_pos: usize,
    count: usize,
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![0i16; capacity],
            capacity,
            write_pos: 0,
            read_pos: 0,
            count: 0,
        }
    }

    pub fn write(&mut self, data: &[i16]) -> usize {
        for &sample in data {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.capacity;

            if self.count < self.capacity {
                self.count += 1;
            } else {
                // Overwrite oldest: advance read position
                self.read_pos = (self.read_pos + 1) % self.capacity;
            }
        }
        data.len()
    }

    pub fn read(&mut self, out: &mut [i16]) -> usize {
        let to_read = out.len().min(self.count);
        for i in 0..to_read {
            out[i] = self.buffer[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        self.count -= to_read;
        to_read
    }

    pub fn available(&self) -> usize {
        self.count
    }

    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.read_pos = 0;
        self.count = 0;
    }

    pub fn drain_all(&mut self) -> Vec<i16> {
        let mut out = vec![0i16; self.count];
        self.read(&mut out);
        out
    }
}

pub struct SpeechBuffer {
    samples: Vec<i16>,
    max_samples: usize,
    active: bool,
}

impl SpeechBuffer {
    pub fn new(max_samples: usize) -> Self {
        Self {
            samples: Vec::new(),
            max_samples,
            active: false,
        }
    }

    pub fn on_speech_start(&mut self) {
        self.samples.clear();
        self.active = true;
    }

    pub fn push_frame(&mut self, frame: &[i16]) {
        if self.active && self.samples.len() + frame.len() <= self.max_samples {
            self.samples.extend_from_slice(frame);
        }
    }

    pub fn on_speech_end(&mut self) -> Option<Vec<i16>> {
        self.active = false;
        if self.samples.is_empty() {
            None
        } else {
            Some(std::mem::take(&mut self.samples))
        }
    }

    pub fn current_speech_len(&self) -> usize {
        self.samples.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_buffer_write_and_read() {
        let mut buf = AudioRingBuffer::new(1024);
        let data = vec![1i16, 2, 3, 4, 5];
        assert_eq!(buf.write(&data), 5);
        let mut out = vec![0i16; 5];
        assert_eq!(buf.read(&mut out), 5);
        assert_eq!(out, data);
    }

    #[test]
    fn test_audio_buffer_overflow_drops_oldest() {
        let mut buf = AudioRingBuffer::new(4);
        buf.write(&[1, 2, 3, 4]);
        buf.write(&[5, 6]);
        let mut out = vec![0i16; 4];
        let read = buf.read(&mut out);
        assert_eq!(read, 4);
        assert_eq!(out, vec![3, 4, 5, 6]);
    }

    #[test]
    fn test_audio_buffer_available_samples() {
        let mut buf = AudioRingBuffer::new(1024);
        assert_eq!(buf.available(), 0);
        buf.write(&[1, 2, 3]);
        assert_eq!(buf.available(), 3);
    }

    #[test]
    fn test_audio_buffer_clear() {
        let mut buf = AudioRingBuffer::new(1024);
        buf.write(&[1, 2, 3]);
        buf.clear();
        assert_eq!(buf.available(), 0);
    }

    #[test]
    fn test_audio_buffer_drain_all() {
        let mut buf = AudioRingBuffer::new(1024);
        buf.write(&[10, 20, 30, 40, 50]);
        let drained = buf.drain_all();
        assert_eq!(drained, vec![10, 20, 30, 40, 50]);
        assert_eq!(buf.available(), 0);
    }

    #[test]
    fn test_speech_buffer_accumulates_during_speech() {
        let mut sb = SpeechBuffer::new(16000 * 30);
        sb.on_speech_start();
        sb.push_frame(&[100i16; 480]);
        sb.push_frame(&[200i16; 480]);
        assert_eq!(sb.current_speech_len(), 960);
    }

    #[test]
    fn test_speech_buffer_emits_segment_on_end() {
        let mut sb = SpeechBuffer::new(16000 * 30);
        sb.on_speech_start();
        sb.push_frame(&[100i16; 480]);
        let segment = sb.on_speech_end();
        assert!(segment.is_some());
        assert_eq!(segment.unwrap().len(), 480);
    }

    #[test]
    fn test_speech_buffer_no_segment_if_no_speech() {
        let mut sb = SpeechBuffer::new(16000 * 30);
        let segment = sb.on_speech_end();
        assert!(segment.is_none());
    }
}
