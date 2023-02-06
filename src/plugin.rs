use std::rc::Rc;
use mpv_client::mpv_handle;
use crate::logging::{self, Logger};
use crate::discord_client::DiscordClient;
use crate::mpv_event_queue::MpvEventQueue;
use crate::mpv_event_queue::events::{MpvEventHandler, MpvEvent};

pub struct RPCPlugin {
    logger: Rc<Logger>,
    mpv: MpvEventQueue,
    discord: DiscordClient,
}

impl RPCPlugin {
    pub fn new(handle: *mut mpv_handle, client_id: &str) -> Result<Self, &'static str> {
        let logger = Rc::new(Logger::from_env());
        let mpv = MpvEventQueue::from_ptr(handle, Rc::clone(&logger))?;
        let discord = DiscordClient::new(client_id, Rc::clone(&logger))?;

        Ok(Self {
            logger,
            mpv,
            discord
        })
    }

    pub fn run(mut self) {
        loop {
            let event = self.mpv.next_event();
            match event {
                None => continue,
                Some(event) => {
                    if self.handle_event(event)
                    {
                        break;
                    }
                }
            }
        }
    }

    fn handle_event(&mut self, event: MpvEvent) -> bool {
        let exit = match event {
            MpvEvent::Exit => true,
            _ => false
        };

        if let Err(e) = self.discord.handle_event(event) {
            logging::error!(self.logger, "Failed to handle event: {e}");
        }

        exit
    }
}