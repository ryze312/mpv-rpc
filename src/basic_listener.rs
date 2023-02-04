use crate::mpv_event_handler::events::{MpvEvent, Listener, FileInfo};

pub struct MpvListener;

impl MpvListener {
    fn print_file_info(&self, file_info: FileInfo) {
        let FileInfo {filename, metadata} = file_info;

        println!("[RPC] FILENAME: {filename}");
        
        if let Some(artist) = metadata.artist {
            println!("[RPC] ARTIST: {artist}");
        }

        if let Some(album) = metadata.album {
            println!("[RPC] ALBUM: {album}");
        }

        if let Some(title) = metadata.title {
            println!("[RPC] TITLE: {title}");
        }

        if let Some(track) = metadata.track {
            println!("[RPC] TRACK: {track}");
        }
    }

    fn print_seek_time(&self, time: i64) {
        println!("[RPC] SEEKING: {time}");
    }

    fn print_play(&self) {
        println!("[RPC] PLAY");
    }

    fn print_pause(&self) {
        println!("[RPC] PAUSE");
    }
}


impl Listener for MpvListener {
    fn handle_event(&self, event: MpvEvent) {
        match event {
            MpvEvent::FileLoaded(file_info) => self.print_file_info(file_info),
            MpvEvent::Seek(time) => self.print_seek_time(time),
            MpvEvent::Pause => self.print_pause(),
            MpvEvent::Play => self.print_play(),
        }
    }
}