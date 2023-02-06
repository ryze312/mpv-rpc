use std::rc::Rc;
use std::time::SystemTime;
use crate::logging::{self, Logger};
use crate::mpv_event_queue::events::{MpvEventHandler, MpvEvent, FileInfo};
use discord_rich_presence::{DiscordIpcClient, DiscordIpc};
use discord_rich_presence::activity::{Activity, Assets, Timestamps};

struct ActivityInfo {
    details: String,
    state: String,
    timestamps: Timestamps
}

impl ActivityInfo {
    pub fn new(details: String, state: String, timestamps: Timestamps) -> Self {
        Self {
            details,
            state,
            timestamps
        }
    }

    pub fn empty() -> Self {
        Self {
            details: String::new(),
            state: String::new(),
            timestamps: Timestamps::new()
        }
    }


    pub fn get_activity(&self) -> Activity {
        let assets = Assets::new()
                    .large_image("logo")
                    .large_text("mpv");

        Activity::new()
                    .assets(assets)
                    .details(&self.details)
                    .state(&self.state)
                    .timestamps(self.timestamps.clone())
    }
}


pub struct DiscordClient {
    logger: Rc<Logger>,
    discord: DiscordIpcClient,
    activity_info: ActivityInfo,
    active: bool
}

impl DiscordClient {
    pub fn new(client_id: &str, logger: Rc<Logger>) -> Result<Self, &'static str> {
        let discord = match DiscordIpcClient::new(client_id) {
            Ok(discord) => discord,
            Err(_) => return Err("cannot init discord client")
        };

        Ok(Self {
            logger,
            discord,
            activity_info: ActivityInfo::empty(),
            active: false
        })
    }

    fn get_state(file_info: &FileInfo) -> String {
        let metadata = &file_info.metadata;
        let mut state = String::new();

        if let Some(artist) = &metadata.artist {
            state += &format!("by {artist}");
        }

        if let Some(album) = &metadata.album {
            state += &format!(" on {album}");
        }
        state
    }

    fn get_details(file_info: &FileInfo) -> String {
        let metadata = &file_info.metadata;
        let title = match &metadata.title {
            Some(title) => title,
            None => return file_info.filename.clone()
        };

        if let Some(track) = &metadata.track {
            format!("{title} [T{track}] ")
        }
        else {
            title.clone()
        }
    }

    fn update_presence(&mut self) -> Result<(), &'static str> {
        if !self.active {
            return Ok(())
        }

        logging::info!(self.logger, "Updating rich presence");

        match self.discord.set_activity(self.activity_info.get_activity()) {
            Ok(()) => {
                Ok(())
            }
            Err(_) => Err("cannot set presence")
        }
    }

    fn set_presence(&mut self, file_info: FileInfo) -> Result<(), &'static str> {
        let details = DiscordClient::get_details(&file_info);
        let state = DiscordClient::get_state(&file_info);
        self.activity_info = ActivityInfo::new(details, state, Timestamps::new());

        self.update_presence()
    }

    fn update_timestamps(&mut self, remaining_time: i64) -> Result<(), &'static str> {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
        let current_time = match current_time {
            Ok(time) => time.as_secs() as i64,
            Err(_) => return Err("cannot get current system time")
        };

        let predicted_time = current_time + remaining_time;

        self.activity_info.timestamps = Timestamps::new().end(predicted_time);
        self.update_presence()
    }

    fn open(&mut self) -> Result<(), &'static str> {
        if self.active {
            return Ok(());
        }

        logging::info!(self.logger, "Opening discord client");
        match self.discord.connect() {
            Ok(()) => {
                self.active = true;
                self.update_presence()
            }
            Err(_) => Err("cannot connect to Discord")
        }
    }

    fn close(&mut self) -> Result<(), &'static str> {
        if !self.active {
            return Ok(());
        }

        logging::info!(self.logger, "Closing discord client");
        match self.discord.close() {
            Ok(()) => {
                self.active = false;
                Ok(())
            }
            Err(_) => Err("cannot disconnect from Discord")
        }
    }

    fn toggle_activity(&mut self) -> Result<(), &'static str> {
        match self.active {
            false => self.open(),
            true => self.close()
        }
    }
}

impl MpvEventHandler for DiscordClient {
    fn handle_event(&mut self, event: MpvEvent) -> Result<(), &'static str> {
        match event {
            MpvEvent::FileLoaded(file_info) => self.set_presence(file_info),
            MpvEvent::Seek(remaining_time) => self.update_timestamps(remaining_time),
            MpvEvent::Toggle => self.toggle_activity(),
            MpvEvent::Exit => self.close(),
            _ => Ok(())
        }
    }
}