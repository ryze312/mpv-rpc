use std::fmt::Display;

pub struct FileInfo {
    pub filename: String,
    pub metadata: FileMetadata
}

pub struct FileMetadata {
    pub artist: Option<String>,
    pub album: Option<String>,
    pub title: Option<String>,
    pub track: Option<String>
}

pub enum MpvEvent {
    Toggle,
    Play,
    Pause,
    Exit,
    FileLoaded(FileInfo),
    Seek(i64)
}

impl Display for MpvEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_name = match self {
            MpvEvent::Toggle => "Toggle",
            MpvEvent::Play => "Play",
            MpvEvent::Pause => "Pause",
            MpvEvent::Exit => "Exit",
            MpvEvent::FileLoaded(_) => "FileLoaded",
            MpvEvent::Seek(_) => "Seek"
        };
        write!(f, "{}", event_name)
    }
}


pub trait MpvEventHandler {
    fn handle_event(&mut self, event: MpvEvent) -> Result<(), &'static str>;
}