use reqwest::{header::USER_AGENT, RequestBuilder, Response, StatusCode};
use serde::{Serialize, Deserialize};
use std::thread::sleep;
use std::time::Duration;
use async_recursion::async_recursion;

use crate::config::ANIDEX_USER_AGENT;

#[derive(Serialize, Deserialize, Clone)]
pub struct MediaTitle<'a> {
    pub romaji: &'a str
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Media<'a> {
    pub id: u64,
    #[serde(borrow)]
    pub title: MediaTitle<'a>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MediaList<'a> {
    pub progress: u16,
    #[serde(borrow)]
    pub media: Media<'a>
}

#[derive(Serialize, Deserialize)]
pub struct MediaEntries<'a> {
    #[serde(borrow)]
    pub entries: Vec<MediaList<'a>>
}
    
#[derive(Serialize, Deserialize)]
pub struct MediaLists<'a> {
    #[serde(borrow)]
    pub lists: Vec<MediaEntries<'a>>
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct AniListData<'a> {
    #[serde(borrow)]
    pub media_list_collection: MediaLists<'a>
}

#[derive(Serialize, Deserialize)]
pub struct GraphQLResponse<'a> {
    #[serde(borrow)]
    pub data: AniListData<'a>
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

    let response = client.try_clone().unwrap().header(USER_AGENT, ANIDEX_USER_AGENT)
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

pub async fn get_anilist_entries<'a, S: Into<String>>(user_name: S) -> Result<MediaLists<'a>, reqwest::Error> {
    let post_body = PostBody {
        variables: GraphQLVariables {
            user_name: &user_name.into(),
        },
        query: GRAPHQL_QUERY
    };
    
    let request = reqwest::Client::new()
        .post(ANILIST_API_URL)
        .json(&post_body);

    let response = anilist_post_request(request).await?;
    let res: GraphQLResponse = response.json().await?;
    Ok(res.data.media_list_collection)
}
