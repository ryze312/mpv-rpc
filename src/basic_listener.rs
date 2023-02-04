use std::rc::Rc;
use crate::mpv_event_handler::events::{MpvEvent, Listener, FileInfo};
use crate::logging::{self, Logger};

pub struct MpvListener {
    logger: Rc<Logger>
}

impl MpvListener {
    pub fn new(logger: Rc<Logger>) -> Self {
        Self {
            logger
        }
    }


    fn print_file_info(&self, file_info: FileInfo) -> Result<(), &'static str> {
        let FileInfo {filename, metadata} = file_info;

        logging::info!(self.logger, "FILENAME {}", filename);
        
        if let Some(artist) = metadata.artist {
            logging::info!(self.logger, "ARTIST: {artist}");
        }

        if let Some(album) = metadata.album {
            logging::info!(self.logger, "ALBUM: {album}");
        }

        if let Some(title) = metadata.title {
            logging::info!(self.logger, "TITLE: {title}");
        }

        if let Some(track) = metadata.track {
            logging::info!(self.logger, "TRACK: {track}");
        }
        Ok(())
    }

    fn print_seek_time(&self, time: i64) -> Result<(),  &'static str>{
        logging::info!(self.logger, "SEEKING: {time}");
        Ok(())
    }

    fn print_play(&self) -> Result<(),  &'static str> {
        logging::info!(self.logger, "PLAY");
        Ok(())
    }

    fn print_pause(&self) -> Result<(),  &'static str> {
        logging::info!(self.logger, "PAUSE");
        Ok(())
    }
}


impl Listener for MpvListener {
    fn handle_event(&self, event: MpvEvent) -> Result<(), &'static str>{
        match event {
            MpvEvent::FileLoaded(file_info) => self.print_file_info(file_info),
            MpvEvent::Seek(time) => self.print_seek_time(time),
            MpvEvent::Pause => self.print_pause(),
            MpvEvent::Play => self.print_play(),
        }
    }
}