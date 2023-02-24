use std::{rc::Rc, time::Duration};
use mpv_client::{Handle, Event, Property, Format, mpv_handle, ClientMessage};
use crate::logging::{self, Logger};

pub mod events;
use events::{MpvEvent, MpvRequest, FileInfo, FileMetadata};


const NAME_PAUSE_PROP: &str = "pause";
const REPL_PAUSE_PROP: u64 = 1;

pub struct MpvEventQueue {
    mpv: Handle,
    logger: Rc<Logger>
}

impl MpvEventQueue {
    pub fn new(mpv: Handle,logger: Rc<Logger>) -> Result<Self, &'static str>  {        
        let new_self = Self {
            mpv,
            logger,
        };
        
        new_self.initialize()?;        
        Ok(new_self)
    }

    pub fn from_ptr<'a>(handle: *mut mpv_handle, logger: Rc<Logger>) -> Result<Self, &'static str> {
        MpvEventQueue::new(Handle::from_ptr(handle), logger)
    }

    fn initialize(&self) -> Result<(), &'static str> {
        self.observe_property(REPL_PAUSE_PROP, NAME_PAUSE_PROP, bool::MPV_FORMAT)
    }

    fn observe_property(&self, id: u64, name: &str, format: i32) -> Result<(), &'static str> {
        match self.mpv.observe_property(id, name, format) {
            Ok(_) => Ok(()),
            Err(_) => Err("cannot observe property")
        }
    }

    pub fn next_event(&mut self) -> Option<MpvEvent> {
        let event = self.mpv.wait_event(-1.0);
        let mpv_event = self.convert_event(event);
        mpv_event
    }

    pub fn handle_request(&self, request: MpvRequest) -> Result<(), &'static str> {
        match request {
            MpvRequest::OSDMessage(message) => self.display_osd_message(message)
        }
    }
    
    pub fn display_osd_message(&self, message: &str) -> Result<(), &'static str> {
        match self.mpv.osd_message(message, Duration::from_secs(1)) {
            Ok(()) => Ok(()),
            Err(_) => Err("cannot print OSD message")
        }
    }

    fn convert_event(&self, event: Event) -> Option<MpvEvent> {
        match event {
            Event::None => (),
            ref event => logging::info!(self.logger, "Event: {event}")
        }

        match event {
            Event::FileLoaded => self.get_file_info_event(),
            Event::PlaybackRestart => self.get_seek_event(),
            Event::ClientMessage(message) => self.get_toggle_event(message),
            Event::PropertyChange(prop_id, prop) => self.get_property_event(prop_id, prop),
            Event::Shutdown => Some(MpvEvent::Exit),
            _ => None
        }
    }

    fn get_file_info_event(&self) -> Option<MpvEvent> {
        let filename = self.mpv.get_property("filename").unwrap();
        let artist = self.mpv.get_property("metadata/by-key/artist").ok();
        let album_artist = self.mpv.get_property("metadata/by-key/album_artist").ok();
        let album = self.mpv.get_property("metadata/by-key/album").ok();
        let title = self.mpv.get_property("metadata/by-key/title").ok();
        let track = self.mpv.get_property("metadata/by-key/track").ok();

        let metadata = FileMetadata {
            artist,
            album_artist,
            album,
            title,
            track
        };

        let file_info = FileInfo {
            filename,
            metadata
        };

        Some(MpvEvent::FileLoaded(file_info))
    }

    fn get_property_event(&self, prop_id: u64, prop: Property) -> Option<MpvEvent> {
        logging::info!(self.logger, "Property changed: {prop_id}");
        match prop_id {
            1 => self.convert_pause_prop(prop.data().unwrap()),
            2 => self.convert_buffering_prop(prop.data().unwrap()),
            _ => None
        }
    }

    fn convert_pause_prop(&self, pause: bool) -> Option<MpvEvent> {
        let time = self.get_remaining_time();
        match pause {
            false => Some(MpvEvent::Play(time)),
            true => Some(MpvEvent::Pause(time))
        }
    }

    pub fn convert_buffering_prop(&self, buffering: bool) -> Option<MpvEvent> {
        match buffering {
            true => Some(MpvEvent::Buffering),
            false => None
        }
    }

    fn get_seek_event(&self) -> Option<MpvEvent> {
        Some(MpvEvent::Seek(self.get_remaining_time()))
    }

    fn get_remaining_time(&self) -> i64 {
        self.mpv.get_property("time-remaining").unwrap_or_default()
    }

    fn get_toggle_event(&self, message: ClientMessage) -> Option<MpvEvent> {
        let command = message.args().join(" ");
        logging::info!(self.logger, "Client message: {command}");
        
        if command.starts_with("key-binding toggle-rpc d-") {
            Some(MpvEvent::Toggle)
        }
        else {
            None
        }
    }

}
