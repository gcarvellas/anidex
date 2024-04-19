use reqwest::{header::USER_AGENT, RequestBuilder, Response, StatusCode};
use serde::{Serialize, Deserialize};
use std::thread::sleep;
use std::time::Duration;
use async_recursion::async_recursion;

use crate::config::ANIDEX_USER_AGENT;

#[derive(Serialize, Deserialize)]
pub struct MediaTitle {
    pub romaji: String
}

#[derive(Serialize, Deserialize)]
pub struct Media {
    pub id: u64,
    pub title: MediaTitle,
}

#[derive(Serialize, Deserialize)]
pub struct MediaList {
    pub progress: u16,
    pub media: Media
}

#[derive(Serialize, Deserialize)]
pub struct MediaEntries {
    pub entries: Vec<MediaList>
}
    
#[derive(Serialize, Deserialize)]
pub struct MediaLists {
    pub lists: Vec<MediaEntries>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AniListData {
    pub media_list_collection: MediaLists
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLResponse {
    pub data: AniListData
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphQLVariables<'a> {
    user_name: &'a str
}

#[derive(Serialize, Deserialize)]
struct PostBody<'a> {
    variables: GraphQLVariables<'a>,
    query: &'a str
}

// TODO filter out light novels
const GRAPHQL_QUERY: &str = r#"
    query ($userName: String) {
        MediaListCollection (userName: $userName, type: MANGA, status: CURRENT) {
            lists {
                entries {
                    progress
                    media {
                        id
                        title {
                            romaji
                        }
                    }
                }
            }
        }
    }
"#;
const ANILIST_API_URL: &str = "https://graphql.anilist.co";

#[async_recursion(?Send)]
async fn anilist_post_request(client: RequestBuilder) -> Result<Response, reqwest::Error> {

    let response = client.try_clone().unwrap()
                         .send()
                         .await?;

    return match response.status() {
        StatusCode::OK => {
            Ok(response)
        },
        StatusCode::TOO_MANY_REQUESTS => {
            let delay = response.headers().get("Retry-After").unwrap().to_str().unwrap().parse::<u64>().unwrap();
            sleep(Duration::from_secs(delay)); 
            anilist_post_request(client).await
        },
        _ => response.error_for_status(),
    };
}

pub async fn get_anilist_entries<'a, S: Into<String>>(user_name: S) -> Result<MediaLists, reqwest::Error> {
    let post_body = PostBody {
        variables: GraphQLVariables {
            user_name: &user_name.into(),
        },
        query: GRAPHQL_QUERY
    };
    
    let request = reqwest::Client::new()
        .post(ANILIST_API_URL)
        .header(USER_AGENT, ANIDEX_USER_AGENT)
        .json(&post_body);

    let response = anilist_post_request(request).await?;
    let res: GraphQLResponse = response.json().await?;
    Ok(res.data.media_list_collection)
}
