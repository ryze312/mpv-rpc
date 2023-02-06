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
    Buffering,
    Exit,
    FileLoaded(FileInfo),
    Play(i64),
    Pause(i64),
    Seek(i64)
}

pub enum MpvRequest {
    OSDMessage(&'static str)
}

pub trait MpvEventHandler {
    fn handle_event(&mut self, event: MpvEvent) -> Result<(), &'static str>;
}

pub trait MpvRequester {
    fn next_request<'a>(&mut self) -> Option<MpvRequest>;
}
