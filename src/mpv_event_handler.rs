use mpv_client::{Handle, Event, Property, Format, mpv_handle};

pub mod events;
use events::{MpvEvent, Listener, FileInfo, FileMetadata};


const NAME_PAUSE_PROP: &str = "pause";
const REPL_PAUSE_PROP: u64 = 1;

pub struct MpvEventHandler {
    mpv: Handle,
    listener: Box<dyn Listener>
}

impl MpvEventHandler {
    pub fn new<'a>(mpv: Handle, listener: Box<dyn Listener>) -> Self {
        Self {
            mpv,
            listener
        }
    }

    pub fn from_ptr<'a>(handle: *mut mpv_handle, listener: Box<dyn Listener>) -> Self {
        MpvEventHandler::new(Handle::from_ptr(handle), listener)
    }

    pub fn initialize(&self) -> Result<(), String> {
       self.observe_property(REPL_PAUSE_PROP, NAME_PAUSE_PROP, bool::MPV_FORMAT)
    }

    fn observe_property(&self, id: u64, name: &str, format: i32) -> Result<(), String>{
        match self.mpv.observe_property(id, name, format) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Couldn't observe property: {name} (id: {id})"))
        }
    }

    fn on_property_change(&self, prop_id: u64, prop: Property) {
        println!("[RPC] Property changed: {prop_id}");
        match prop_id {
            REPL_PAUSE_PROP => self.handle_pause_change(prop.data().unwrap()),
            _ => ()
        }
    }

    fn handle_event(&self, event: Event) {
        println!("[RPC] Event: {event}");
        match event {
            Event::FileLoaded => self.handle_file_loaded(),
            Event::PlaybackRestart => self.handle_playback_restart(),
            _ => ()
        }
    }

    // TODO
    // Replace unwrap with error handling
    fn handle_file_loaded(&self) {
        let filename = self.mpv.get_property("filename").unwrap();
        let artist = self.mpv.get_property("metadata/by-key/artist").ok();
        let album = self.mpv.get_property("metadata/by-key/album").ok();
        let title = self.mpv.get_property("metadata/by-key/title").ok();
        let track = self.mpv.get_property("metadata/by-key/Artist").ok();

        let metadata = FileMetadata {
            artist,
            album,
            title,
            track
        };

        let file_info = FileInfo {
            filename,
            metadata
        };

        let event = MpvEvent::FileLoaded(file_info);
        self.listener.handle_event(event);
    }

    fn handle_playback_restart(&self) {
        let res = self.mpv.get_property("playback-time").ok();
        match res {
            Some(time) => {
                let event = MpvEvent::Seek(time);
                self.listener.handle_event(event);
            }
            None => {
                println!("[RPC] Failed retrieving playback-time.");
                println!("[RPC] This usually happens seeking into file end. Possibly mpv bug?");
            },
        }
    }

    fn handle_pause_change(&self, pause: bool) {
        let event = match pause {
            false => MpvEvent::Play,
            true => MpvEvent::Pause
        };
        self.listener.handle_event(event);
    }

    // TODO! 
    // Add logging and error handling,
    // yes unwrap on property changes and events
    pub fn poll_events(&self) -> bool {
        match self.mpv.wait_event(0.0) {
            Event::None => (),
            Event::Shutdown => return false,
            Event::PropertyChange(prop_id, prop) => self.on_property_change(prop_id, prop),
            event => self.handle_event(event)
        }
        true
    }
}
