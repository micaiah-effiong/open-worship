use gtk::glib;

use crate::dto::{SongData, SongVerse};
use openlyrics;

pub fn openlyrics_to_song_data(content: &str) -> Option<SongData> {
    let lyrics_res = quick_xml::de::from_str::<openlyrics::types::Song>(&content);

    let song_lyrics = match lyrics_res {
        Ok(res) => res,
        Err(e) => {
            glib::g_warning!("Openlyrics", "Error parsing openlyrics: {:?}", e);
            return None;
        }
    };

    let title = &song_lyrics.properties.titles.titles[0].title;
    let _authors = &song_lyrics.properties.authors.authors[0].name;
    let lyrics = &song_lyrics
        .lyrics
        .lyrics
        .iter()
        .filter_map(|v| match v {
            openlyrics::types::LyricEntry::Verse {
                name,
                lang: _,
                translit: _,
                lines,
            } => {
                let line = lines
                    .iter()
                    .map(|v| openlyrics::simplify_contents(&v.contents).join("\n"))
                    .collect::<Vec<_>>()
                    .join("\n");
                return Some((name.clone(), line));
            }
            _ => return None,
        })
        .collect::<Vec<_>>();

    let song_verses = lyrics
        .iter()
        .map(|(t, l)| SongVerse::new(l.clone(), Some(t.clone()), None))
        .collect::<Vec<_>>();
    let data = SongData::new(0, title.clone(), song_verses);

    return Some(data);
}
