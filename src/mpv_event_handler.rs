use std::{rc::Rc, cell::{RefCell, Cell}, borrow::Borrow};
use crate::{logging::{self, Logger}, info};
use mpv_client::{Handle, Event, Property, Format, mpv_handle, ClientMessage};

pub mod events;
use events::{MpvEvent, Listener, FileInfo, FileMetadata};

const NAME_PAUSE_PROP: &str = "pause";
const REPL_PAUSE_PROP: u64 = 1;

pub struct MpvEventHandler {
    listening: bool,
    mpv: Handle,
    listener: Box<dyn Listener>,
    logger: Rc<Logger>
}

impl MpvEventHandler {
    pub fn new(mpv: Handle, listener: Box<dyn Listener>, logger: Rc<Logger>) -> Result<Self, String>  {
        let mpv = Cell::new(mpv);
        let listener = Cell::new(listener);
        
        let new_self = Self {
            listening: false,
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

    pub fn run(self) {
        logging::info!(self.logger, "Starting mpv-rpc");
        logging::info!(self.logger, "Client name: {}", self.mpv.borrow().as_ptr().cl);

        while self.poll_events() {
            // Wait for shutdown
        }

        if self.listening {
            self.listener.get_mut().close();
        }
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
    
    fn get_event(&self) -> Event {
        self.mpv.wait_event(0.0)
    }

    fn poll_events(&self) -> bool {
        let event = self.mpv.wait_event(0.0);
        let result = match event {
            Event::None => Ok(()),
            Event::Shutdown => return false,
            event => self.handle_event(event)
        };

        if let Err(e) = result {
            logging::error!(self.logger, "Error handling event: {e}");
        }

        true
    }

    fn handle_event(&self, event: Event) -> Result<(), &'static str> {
        logging::info!(self.logger, "Event: {event}");

        if let Event::ClientMessage(message) = event {
            return self.handle_client_message(message);
        }
        
        if self.listening {
            match event {
                Event::PropertyChange(prop_id, prop) => self.send_property_to_listener(prop_id, prop),
                e => self.send_event_to_listener(e)
            }
        } 
        else {
            Ok(())
        }
    }

    fn send_event_to_listener(&self, event: Event) -> Result<(), &'static str>{
        match event {
            Event::FileLoaded => self.send_file_info(),
            Event::PlaybackRestart => self.send_seek(),
            _ => Ok(())
        }
    }

    fn send_property_to_listener(&self, prop_id: u64, prop: Property) -> Result<(), &'static str> {
        logging::info!(self.logger, "Property changed: {prop_id}");
        match prop_id {
            REPL_PAUSE_PROP => {
                match prop.data() {
                    Some(pause) => self.send_pause(pause),
                    None => Err("property pause doesn't exist")
                }
            }
            _ => Ok(())
        }
    }

    fn send_file_info(&self) -> Result<(), &'static str> {
        let filename_res = self.mpv.get_property("filename");
        let artist = self.mpv.get_property("metadata/by-key/artist").ok();
        let album = self.mpv.get_property("metadata/by-key/album").ok();
        let title = self.mpv.get_property("metadata/by-key/title").ok();
        let track = self.mpv.get_property("metadata/by-key/track").ok();

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
        self.listener.get_mut().handle_event(event)
    }


    fn send_seek(&self) -> Result<(), &'static str>{
        let remaining_time = self.mpv.get_property("time-remaining").unwrap_or_else(|_| {
            logging::warning!(self.logger, "Failed retrieving remaing-time.");
            logging::warning!(self.logger, "This usually happens seeking into file end. Possibly mpv bug?");
            logging::warning!(self.logger, "Defaulting to 0.");
            0
        });

        let event = MpvEvent::Seek(remaining_time);
        self.listener.get_mut().handle_event(event)
    }

    fn send_pause(&self, pause: bool) -> Result<(), &'static str> {
        let event = match pause {
            false => MpvEvent::Play,
            true => MpvEvent::Pause
        };
        self.listener.get_mut().handle_event(event)
    }

    fn toggle_listening(&self) -> Result<(), &'static str> {
        let new_state = !self.listening;
        let mut listener_ref = self.listener.get_mut();

        let res = match new_state {
            true => listener_ref.open(),
            false => listener_ref.close()
        };

        match res {
            Ok(()) => self.listening = true,
            Err(e) => return Err(e)
        }
        Ok(())
    }

    fn handle_client_message(&self, message: ClientMessage) -> Result<(), &'static str> {
        let command = message.args().join(" ");
        logging::info!(self.logger, "Client message: {command}");
        
        if command.starts_with("key-binding toggle-rpc d-") {
            self.toggle_listening()
        }
        else {
            Ok(())
        }
    }
}
