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
    FileLoaded(FileInfo),
    Seek(i64),
    Pause,
    Play
}

pub trait Listener {
    fn handle_event(&self, event: MpvEvent);
}