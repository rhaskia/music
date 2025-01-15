use log::info;
use rodio::{Decoder, OutputStream, Sink, OutputStreamHandle, Source};
use std::fs::File;
use std::io::BufReader;
use std::time::Duration;

pub struct AudioPlayer {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
    current_song_len: f64,
}

impl PartialEq for AudioPlayer {
    fn eq(&self, other: &Self) -> bool {
        self.current_song_len == other.current_song_len
    }
}

impl AudioPlayer {
    pub fn new() -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let sink = Sink::try_new(&stream_handle).unwrap();

        AudioPlayer { stream, stream_handle, sink, current_song_len: 1.0 }
    }

    pub fn play_track(&mut self, file_path: &str) {
        info!("Playing track: {file_path:?}");
        let file = BufReader::new(File::open(file_path).unwrap());
        let source = Decoder::new(file).unwrap();
        let was_paused = self.sink.is_paused();
        self.current_song_len = source.total_duration().unwrap().as_secs_f64();
        self.sink.clear();
        self.sink.append(source);
        if !was_paused {
            self.sink.play();
        }
        info!("Track successfully played");
    }

    pub fn skip(&mut self) {
        self.sink.skip_one();
    }

    pub fn toggle_playing(&mut self) {
        if self.sink.is_paused() {
            self.sink.play();
        } else {
            self.sink.pause();
        }
    }

    pub fn playing(&self) -> bool {
        !self.sink.is_paused()
    }

    pub fn progress_percent(&self) -> f64 {
        self.sink.get_pos().as_secs_f64() / self.current_song_len
    }

    pub fn progress_secs(&self) -> f64 {
        self.sink.get_pos().as_secs_f64()
    }

    pub fn track_ended(&self) -> bool {
        self.sink.empty()
    }

    pub fn song_length(&self) -> f64 {
        self.current_song_len
    }

    pub fn set_pos(&mut self, pos: f64) {
        let _ = self.sink.try_seek(Duration::from_secs_f64(pos));
    }
}