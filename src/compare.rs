use std::error::Error;
use crate::{anilist::{get_anilist_entries, MediaList, MediaEntries}, mangadex::{mangadex_find_id, mangadex_latest_chapter_from_id}};
use tokio::task;

#[derive(Clone)]
pub struct UnreadManga {
    id: String,
    title: String,
    chapter: u16,
    latest: f32
}

impl std::fmt::Display for UnreadManga {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO make the output nicer
        write!(f, "{}: current: {}, latest: {}", generate_terminal_hyperlink(mangadex_title_url(&self.id), &self.title), self.chapter, self.latest)
    }
}

fn generate_terminal_hyperlink(link: String, text: &str) -> String {
    // Escape special characters in the link
    let escaped_link = link.replace("\x1b", "\\e").replace('\n', "\\n");

    // Format the terminal hyperlink
    format!("\x1b]8;;{}\x07{}\x1b]8;;\x07", escaped_link, text)
}

#[inline(always)]
fn mangadex_title_url(title_id: &str) -> String {
    format!("https://mangadex.org/title/{}", title_id)
}

async fn filter_entries(entries: &[MediaList], language: &str) -> Result<Vec<UnreadManga>, Box<dyn Error>> {
    let mut unread_mangas: Vec<UnreadManga> = vec![];

    for entry in entries.iter() {
        let title = &entry.media.title.romaji;
        let id = match mangadex_find_id(title.clone(), entry.media.id).await? {

            // Ignore mangas that are on anilist but not mangadex
            None => continue,

            Some(data) => data
        };
        
        let latest_chapter = match mangadex_latest_chapter_from_id(id.clone(), language).await? {
            
            // No translation exists for the selected language
            None => continue,

            Some(data) => data
        };

        if entry.progress < (latest_chapter as u64).try_into().unwrap() {
            unread_mangas.push(UnreadManga {
                id,
                title: title.to_string(),
                chapter: entry.progress,
                latest: latest_chapter
            });
        }
    }

    Ok(unread_mangas)
}

async fn get_mangas_from_list_parallel(list: MediaEntries, language: &str, mut workers: usize) -> Result<Vec<UnreadManga>, Box<dyn Error>> {
    let mut unread_mangas: Vec<UnreadManga> = vec![];
    let mut handles = vec![];
    let entries_size = list.entries.len();

    // Only allow one worker per entry
    workers = std::cmp::min(entries_size, workers);

    let slice_size: usize = entries_size / workers;
    for worker_no in 0..workers {
        let start = worker_no * slice_size;
        let end = if worker_no == workers - 1 {
            entries_size
        } else {
            (worker_no + 1) * slice_size
        };

        let entries_slice = list.entries[start..end].to_vec();
        let language_clone = language.to_string();

        let handle = task::spawn(async move {
            filter_entries(&entries_slice, &language_clone).await.unwrap()
        });

        handles.push(handle); 
    }

    for handle in handles {
        unread_mangas.append(&mut handle.await.unwrap());
    }

    Ok(unread_mangas)
}

pub async fn filter_unread_manga(username: String, language: &str, workers: usize) -> Result<Vec<UnreadManga>, Box<dyn Error>> {
    let mangas = get_anilist_entries(username).await?;
    let mut unread_mangas: Vec<UnreadManga> = vec![];

    for list in mangas.lists {
        unread_mangas.append(&mut get_mangas_from_list_parallel(list, language, workers).await?);
    }

    Ok(unread_mangas)
}
