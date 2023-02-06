use musicbrainz_rs::entity::release_group::{ReleaseGroup, ReleaseGroupSearchQuery};
use musicbrainz_rs::entity::CoverartResponse;
use musicbrainz_rs::{Search, FetchCoverart};

pub fn get_cover_art_url(title: &Option<String>, album: &Option<String>, artist: &Option<String>) -> Option<String>{
    let mut builder = ReleaseGroupSearchQuery::query_builder();
    if let Some(ref title) = title {
        builder.release_group(title);
    }

    if let Some(ref album) = album {
        builder.or().release_group(album);
    }

    if let Some(ref artist) = artist {
        // Some artist fields might contain + characters
        // Pointing at multiple artists
        for part in artist.split("+") {
            builder.and().artist(part);
        }
    }

    let query = builder.build();
    
    let result = match ReleaseGroup::search(query).execute() {
        Ok(res) => res,
        Err(_) => return None
    };

    let release = match result.entities.get(0) {
        Some(group) => group,
        None => return None
    };

    let cover_art = match release.get_coverart().front().execute() {
        Ok(art) => art,
        Err(_) => return None
    };
    match cover_art {
        CoverartResponse::Url(url) => Some(url),
        _ => None
    }
}
