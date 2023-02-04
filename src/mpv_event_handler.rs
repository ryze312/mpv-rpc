use mpv_client::{Handle, Event, Property, mpv_handle};

pub struct MpvHandler {
    mpv: Handle
}

impl MpvHandler {
    pub fn new(mpv: Handle) -> Self {
        Self {
            mpv
        }
    }

    pub fn from_ptr(handle: *mut mpv_handle) -> Self {
        MpvHandler::new(Handle::from_ptr(handle))
    }

    #[allow(dead_code)]
    fn observe_property(&self, id: u64, name: &str, format: i32) -> Result<(), String>{
        match self.mpv.observe_property(id, name, format) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("Couldn't observe property: {name} (id: {id})"))
        }
    }
    
    #[allow(dead_code)]
    fn on_property_change(&self, prop_id: u64, _prop: Property) {
        println!("[RPC] Property changed: {prop_id}");
        match prop_id {
            _ => ()
        }
    }

    fn handle_event(&self, event: Event) {
        println!("[RPC] Event: {event}");
        match event {
            _ => ()
        }
    }

    // TODO! 
    // Add logging
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