use std::rc::Rc;
use std::time::SystemTime;
use std::collections::VecDeque;
use discord_rich_presence::{DiscordIpcClient, DiscordIpc};
use discord_rich_presence::activity::{Activity, Assets, Timestamps};
use crate::utils;
use crate::logging::{self, Logger};
use crate::mpv_event_queue::events::{MpvEventHandler, MpvEvent, FileInfo, MpvRequester, MpvRequest, FileMetadata};

const MAX_STR_LEN: usize = 128;

mod music_brainz;

struct ActivityInfo {
    details: String,
    state: String,
    assets: AssetsInfo,
    timestamps: Timestamps
}

impl AssetsInfo {
    pub fn new(large_image: String, large_text: String) -> Self {
        Self {
            large_image,
            large_text
        }
    }

    pub fn empty() -> Self {
        Self {
            large_image: String::new(),
            large_text: String::new(),
        }
    }

    pub fn get_assets(&self) -> Assets {
        Assets::new()
                    .large_image(&self.large_image)
                    .large_text(&self.large_text)
    }
}

struct AssetsInfo {
    large_image: String,
    large_text: String
}

impl ActivityInfo {
    pub fn new(details: String, state: String, assets: AssetsInfo, timestamps: Timestamps) -> Self {
        Self {
            details,
            state,
            assets,
            timestamps
        }
    }

    pub fn empty() -> Self {
        Self {
            details: String::new(),
            state: String::new(),
            assets: AssetsInfo::empty(),
            timestamps: Timestamps::new()
        }
    }


    pub fn get_activity(&self) -> Activity {
        let assets = self.assets.get_assets();

        Activity::new()
                    .assets(assets)
                    .details(&self.details)
                    .state(&self.state)
                    .timestamps(self.timestamps.clone())
    }
}



pub struct DiscordClient {
    discord: DiscordIpcClient,
    activity_info: ActivityInfo,
    active: bool,
    cover_art: bool,
    mpv_requests: VecDeque<MpvRequest>,
    logger: Rc<Logger>
}

impl DiscordClient {
    pub fn new(client_id: &str, active: bool, cover_art: bool, logger: Rc<Logger>) -> Result<Self, &'static str> {
        let discord = match DiscordIpcClient::new(client_id) {
            Ok(discord) => discord,
            Err(_) => return Err("cannot init discord client")
        };

        let mut new_self = Self {
            discord,
            activity_info: ActivityInfo::empty(),
            active: false,
            cover_art,
            mpv_requests: VecDeque::new(),
            logger
        };

        if active {
            new_self.open()?;
        }
        Ok(new_self)
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
        utils::truncate_string_fmt(&mut state, MAX_STR_LEN);
        state
    }

    fn get_details(file_info: &FileInfo) -> String {
        let metadata = &file_info.metadata;
        let title = match &metadata.title {
            Some(title) => title,
            None => return file_info.filename.clone()
        };

        let mut details = if let Some(track) = &metadata.track {
            format!("{title} [T{track}] ")
        }
        else {
            title.clone()
        };

        utils::truncate_string_fmt(&mut details, MAX_STR_LEN);
        details
    }

    fn get_assets_info(cover_art: bool, metadata: FileMetadata) -> AssetsInfo {
        let (large_image, large_text) = DiscordClient::get_large_info(cover_art, metadata);
        AssetsInfo::new(large_image, large_text)
    }

    fn get_large_info(cover_art: bool, metadata: FileMetadata) -> (String, String) {
        if !cover_art {
            return ("logo".to_string(), "mpv".to_string())
        }

        let cover_art_url = music_brainz::get_cover_art_url(&metadata.title, &metadata.album, &metadata.artist);
        let large_image = match cover_art_url {
            Some(url) => url,
            None => "logo".to_string()
        };
        let large_text = match metadata.title.or(metadata.album) {
            Some(text) => text,
            None => "mpv".to_string()
        };

        (large_image, large_text)
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
            Err(_) => {
                self.active = false;
                Err("cannot set presence")
            }
        }
    }

    fn set_presence(&mut self, file_info: FileInfo) -> Result<(), &'static str> {
        let details = DiscordClient::get_details(&file_info);
        let state = DiscordClient::get_state(&file_info);
        let assets_info = DiscordClient::get_assets_info(self.cover_art, file_info.metadata);

        self.activity_info = ActivityInfo::new(details, state, assets_info, Timestamps::new());
        self.update_presence()
    }

    fn set_timestamps(&mut self, remaining_time: i64) -> Result<(), &'static str> {
        let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
        let current_time = match current_time {
            Ok(time) => time.as_secs() as i64,
            Err(_) => return Err("cannot get current system time")
        };

        let predicted_time = current_time + remaining_time;

        self.activity_info.timestamps = Timestamps::new().end(predicted_time);
        self.update_presence()
    }

    fn clear_timestamps(&mut self) -> Result<(), &'static str> {
        self.activity_info.timestamps = Timestamps::new();
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
                self.request_osd_message("Discord RPC started");
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
                self.request_osd_message("Discord RPC stopped");
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

    fn request_osd_message(&mut self, message: &'static str) {
        self.mpv_requests.push_front(MpvRequest::OSDMessage(message));
    }
}

impl MpvEventHandler for DiscordClient {
    fn handle_event(&mut self, event: MpvEvent) -> Result<(), &'static str> {
        match event {
            MpvEvent::FileLoaded(file_info) => self.set_presence(file_info),
            MpvEvent::Seek(remaining_time) => self.set_timestamps(remaining_time),
            MpvEvent::Play(remaining_time) => self.set_timestamps(remaining_time),
            MpvEvent::Pause(_) => self.clear_timestamps(),
            MpvEvent::Buffering => self.clear_timestamps(),
            MpvEvent::Toggle => self.toggle_activity(),
            MpvEvent::Exit => self.close(),
        }
    }
}

impl MpvRequester for DiscordClient {
    fn next_request<'a>(&mut self) -> Option<MpvRequest> {
        self.mpv_requests.pop_back()
    }
}
