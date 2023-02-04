use std::rc::Rc;

use mpv_client::{Handle, Event, Property, Format, mpv_handle};
use crate::logging::{self, Logger};

pub mod events;
use events::{MpvEvent, Listener, FileInfo, FileMetadata};

const NAME_PAUSE_PROP: &str = "pause";
const REPL_PAUSE_PROP: u64 = 1;

pub struct MpvEventHandler {
    mpv: Handle,
    listener: Box<dyn Listener>,
    logger: Rc<Logger>
}

impl MpvEventHandler {
    pub fn new(mpv: Handle, listener: Box<dyn Listener>, logger: Rc<Logger>) -> Result<Self, String>  {
        let new_self = Self {
            mpv,
            listener,
            logger
        };
        
        new_self.initialize()?;        
        Ok(new_self)
    }

    pub fn from_ptr<'a>(handle: *mut mpv_handle, listener: Box<dyn Listener>, logger: Rc<Logger>) -> Result<Self, String> {
        MpvEventHandler::new(Handle::from_ptr(handle), listener, logger)
    }

    fn initialize(&self) -> Result<(), String> {
       self.observe_property(REPL_PAUSE_PROP, NAME_PAUSE_PROP, bool::MPV_FORMAT)
    }

    fn observe_property(&self, id: u64, name: &str, format: i32) -> Result<(), String>{
        match self.mpv.observe_property(id, name, format) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Couldn't observe property: {name} (id: {id})"))
        }
    }

    fn handle_event(&self, event: Event) -> Result<(), &'static str>{
        logging::info!(self.logger, "Event: {event}");
        match event {
            Event::FileLoaded => self.handle_file_loaded(),
            Event::PlaybackRestart => self.handle_playback_restart(),
            _ => Ok(())
        }
    }

    fn on_property_change(&self, prop_id: u64, prop: Property) -> Result<(), &'static str> {
        logging::info!(self.logger, "Property changed: {prop_id}");
        match prop_id {
            REPL_PAUSE_PROP => {
                match prop.data() {
                    Some(pause) => self.handle_pause_change(pause),
                    None => Err("property pause doesn't exist")
                }
            }
            _ => Ok(())
        }
    }

    fn handle_file_loaded(&self) -> Result<(), &'static str> {
        let filename_res = self.mpv.get_property("filename");
        let artist = self.mpv.get_property("metadata/by-key/artist").ok();
        let album = self.mpv.get_property("metadata/by-key/album").ok();
        let title = self.mpv.get_property("metadata/by-key/title").ok();
        let track = self.mpv.get_property("metadata/by-key/Artist").ok();

        let filename = match filename_res {
            Ok(name) => name,
            Err(_) => return Err("filename property doesn't exist")
        };

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
        self.listener.handle_event(event)
    }

    fn handle_playback_restart(&self) -> Result<(), &'static str>{
        let res = self.mpv.get_property("playback-time").ok();
        match res {
            Some(time) => {
                let event = MpvEvent::Seek(time);
                self.listener.handle_event(event)
            }
            None => {
                logging::warning!(self.logger, "Failed retrieving playback-time.");
                logging::warning!(self.logger, "This usually happens seeking into file end. Possibly mpv bug?");
                Ok(())
            },
        }
    }

    fn handle_pause_change(&self, pause: bool) -> Result<(), &'static str> {
        let event = match pause {
            false => MpvEvent::Play,
            true => MpvEvent::Pause
        };
        self.listener.handle_event(event)
    }


    pub fn run(&self) {
        while self.poll_events() {
            // Wait for shutdown
        }
    }

    // Refactor to accept warnings as well
    pub fn poll_events(&self) -> bool {
        let event = self.mpv.wait_event(0.0);
        let event_name = event.to_string();
        let result = match event {
            Event::None => Ok(()),
            Event::Shutdown => return false,
            Event::PropertyChange(prop_id, prop) => self.on_property_change(prop_id, prop),
            e => self.handle_event(e)
        };

        if let Err(e) = result {
            logging::error!(self.logger, "Error handling event {event_name}: {e}");
        }
        true
    }
}
