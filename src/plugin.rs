use std::rc::Rc;
use mpv_client::mpv_handle;
use crate::logging::{self, Logger};
use crate::config::Config;
use crate::discord_client::DiscordClient;
use crate::mpv_event_queue::MpvEventQueue;
use crate::mpv_event_queue::events::{MpvEventHandler, MpvRequester, MpvEvent, MpvRequest};

pub struct RPCPlugin {
    logger: Rc<Logger>,
    mpv: MpvEventQueue,
    discord: DiscordClient,
}

impl RPCPlugin {
    pub fn new(handle: *mut mpv_handle, client_id: &str) -> Result<Self, &'static str> {
        let logger = Rc::new(Logger::from_env());
        let config = Config::from_config_file(&logger);
        let mpv = MpvEventQueue::from_ptr(handle, Rc::clone(&logger))?;
        let discord = DiscordClient::new(client_id, config.active, config.cover_art, Rc::clone(&logger))?;

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
                None => (),
                Some(event) => {
                    if self.handle_event(event)
                    {
                        break;
                    }
                }
            }

            let request = self.discord.next_request();
            match request {
                None => (),
                Some(request) => self.handle_request(request)
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

    fn handle_request(&self, request: MpvRequest) {
        if let Err(e) = self.mpv.handle_request(request) {
            logging::error!(self.logger, "Failed to handle mpv request: {e}");
        }
    }
}