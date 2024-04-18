use std::error::Error;
use reqwest::{RequestBuilder, Response, header::USER_AGENT, StatusCode};
use serde::{Serialize, Deserialize};

use crate::config::ANIDEX_USER_AGENT;

const MANGADEX_MANGA_API_URL: &str = "https://api.mangadex.org/manga"; 

#[derive(Serialize, Deserialize)]
pub struct MangadexApiResponse {
    result: String,
    response: String,
    pub data: Vec<serde_json::Value>,
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
}

async fn mangadex_get_request(client: RequestBuilder) -> Result<Response, reqwest::Error> {

    let response = client.try_clone()
                         .unwrap()
                         .header(USER_AGENT, ANIDEX_USER_AGENT)
                         .send()
                         .await?;

    return match response.status() {
        StatusCode::OK => {
            Ok(response)
        },
        StatusCode::TOO_MANY_REQUESTS => {
            // TODO I was unable to trigger the rate limit on MangaDex's server
            unimplemented!("Rate limit handling for MangaDex is not supported")
        },
        _ => response.error_for_status(),
    };
}

pub async fn mangadex_find_id(title: &str, anilist_id: u64) -> Result<Option<String>, Box<dyn Error>> {

    // MangaDex by default orders by relevance on their website, but even that can be inaccurate.
    // followedCount usually requires less checks of the anilist ID
    let params = [("title", title), ("order[followedCount]", "desc")];

    let url = reqwest::Url::parse_with_params(MANGADEX_MANGA_API_URL, &params)?;
    let request = reqwest::Client::new()
        .get(url);

    let res: MangadexApiResponse = mangadex_get_request(request).await?.json().await?;

    // Find the entry that matches the anilist id
    for entry in res.data {
    
        // Each Manga contains attributers, which has a links section. 'al' is the AniList link
        let id = match entry.get("attributes").unwrap().get("links").unwrap().get("al") {

            // Entry doesn't have an anilist link
            None => continue,

            Some(data) => {
                data.as_str().unwrap().parse::<u64>()?
            }
        };

        if id == anilist_id {
            return Ok(Some(entry.get("id").unwrap().as_str().unwrap().to_string()));
        }
    }

    return Ok(None);
}

pub async fn mangadex_latest_chapter_from_id(id: &str, language: &str) -> Result<Option<f32>, Box<dyn Error>> {
    let params = [
        ("translatedLanguage[]", language),
        ("order[chapter]", "desc"),
        ("limit", "1")
    ];
    let manga_feed_url = format!("https://api.mangadex.org/manga/{}/feed", id);
    let url = reqwest::Url::parse_with_params(
        &manga_feed_url,
        &params
    )?;

    let request = reqwest::Client::new()
        .get(url);

    let res: MangadexApiResponse = mangadex_get_request(request).await?.json().await?;

    // No translation is available for the selected language
    if res.data.len() == 0 {
        return Ok(None);
    }

    // Since the chapters are in descending order, the first chapter will have the latest chapter
    // number in it's attributes section
    Ok(Some(match res.data.first().unwrap().get("attributes").unwrap().get("chapter").unwrap().as_str() {

        // The manga is a Oneshot so it only has one chapter
        None => 1.0,

        Some(s) => s.parse::<f32>()?
    }))
}
