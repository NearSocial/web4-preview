use crate::*;

use near_contract_standards::non_fungible_token::metadata::NFTContractMetadata;
use near_sdk::json_types::Base64VecU8;
use std::str::FromStr;

const DEFAULT_TITLE: &str = "Near Social";
const DEFAULT_DESCRIPTION: &str = "Open Web protocol built on NEAR";
const DEFAULT_IMAGE: &str = "https://near.social/assets/logo.png";

const DEFAULT_ACCOUNT_DESCRIPTION: &str = "User profile on Near Social";
const DEFAULT_ACCOUNT_IMAGE: &str =
    "https://ipfs.near.social/ipfs/bafkreibmiy4ozblcgv3fm3gc6q62s55em33vconbavfd2ekkuliznaq3zm";

const DEFAULT_POST_DESCRIPTION: &str = "";
const DEFAULT_POST_IMAGE: &str = "https://near.social/assets/logo.png";
const IPFS_PREFIX: &str = "https://ipfs.near.social/ipfs";

#[allow(dead_code)]
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Request {
    #[serde(rename = "accountId")]
    account_id: Option<AccountId>,
    path: Option<String>,
    params: Option<HashMap<String, String>>,
    query: Option<HashMap<String, Vec<String>>>,
    preloads: Option<HashMap<String, Web4Response>>,
}

impl Web4Request {
    pub fn preload(&self, url: &str) -> Option<&Web4Response> {
        self.preloads
            .as_ref()
            .and_then(|preloads| preloads.get(url))
    }
}

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct Web4Response {
    #[serde(rename = "contentType")]
    content_type: Option<String>,
    status: Option<u32>,
    body: Option<Base64VecU8>,
    #[serde(rename = "bodyUrl")]
    body_url: Option<String>,
    #[serde(rename = "preloadUrls")]
    preload_urls: Option<Vec<String>>,
}

impl Web4Response {
    pub fn html_response(text: String) -> Self {
        Self {
            content_type: Some(String::from("text/html; charset=UTF-8")),
            body: Some(text.into_bytes().into()),
            ..Default::default()
        }
    }

    pub fn plain_response(text: String) -> Self {
        Self {
            content_type: Some(String::from("text/plain; charset=UTF-8")),
            body: Some(text.into_bytes().into()),
            ..Default::default()
        }
    }

    pub fn preload_urls(urls: Vec<String>) -> Self {
        Self {
            preload_urls: Some(urls),
            ..Default::default()
        }
    }

    pub fn body_url(url: String) -> Self {
        Self {
            body_url: Some(url),
            ..Default::default()
        }
    }

    pub fn status(status: u32) -> Self {
        Self {
            status: Some(status),
            ..Default::default()
        }
    }
}

fn filter_string(s: String) -> String {
    s.chars()
        .into_iter()
        .take(250)
        .filter_map(|c| match c {
            '\n' => Some(' '),
            ' ' | '_' | '.' | '-' | ',' | '!' | '(' | ')' | '/' | ':' | '?' => Some(c),
            _ if c.is_alphanumeric() => Some(c),
            _ => None,
        })
        .collect()
}

fn render(
    title: String,
    description: String,
    image: String,
    url: String,
    large: bool,
) -> Web4Response {
    let body = format!(
        include_str!("../template/index.html"),
        card = if large {
            "summary_large_image"
        } else {
            "summary"
        },
        title = title,
        description = description,
        image = format!("https://i.near.social/large/{}", image),
        url = url,
    );
    Web4Response::html_response(body)
}

fn make_account_url(account_id: &AccountId) -> String {
    format!("https://api.near.social/get?keys={}/profile/**", account_id)
}

fn make_post_url(account_id: &AccountId, block_height: u64, is_post: bool) -> String {
    format!(
        "https://api.near.social/get?keys={}/post/{}&blockHeight={}",
        account_id,
        if is_post { "main" } else { "comment" },
        block_height
    )
}

fn get_profile(account_id: &AccountId, request: &Web4Request) -> Option<Profile> {
    let account_url = make_account_url(account_id);
    if let Some(response) = request.preload(&account_url) {
        if let Some(body) = &response.body {
            let social: Social = near_sdk::serde_json::from_slice(&body.0).unwrap_or_default();
            return Some(
                social
                    .0
                    .get(account_id)
                    .cloned()
                    .unwrap_or_default()
                    .profile
                    .unwrap_or_default(),
            );
        }
    }
    None
}

fn get_post(
    account_id: &AccountId,
    block_height: u64,
    is_post: bool,
    request: &Web4Request,
) -> Option<String> {
    let post_url = make_post_url(account_id, block_height, is_post);
    if let Some(response) = request.preload(&post_url) {
        if let Some(body) = &response.body {
            let social: Social = near_sdk::serde_json::from_slice(&body.0).unwrap_or_default();
            let post = social
                .0
                .get(&account_id)
                .cloned()
                .unwrap_or_default()
                .post
                .unwrap_or_default();

            return Some(if is_post {
                post.main.unwrap_or_default()
            } else {
                post.comment.unwrap_or_default()
            });
        }
    }
    None
}

fn render_with_image(
    title: String,
    description: String,
    image: Option<UrlOrNft>,
    image_fallback_url: &str,
    url: String,
    mut preload_urls: Vec<String>,
    request: &Web4Request,
) -> Web4Response {
    if let Some(image) = image {
        match image {
            UrlOrNft::Url(image_url) => render(title, description, image_url, url, true),
            UrlOrNft::Nft(contract_id, token_id) => {
                let nft_url = format!("/nft/{}/{}", contract_id, token_id);
                if let Some(preload) = request.preload(&nft_url) {
                    if let Some(body) = &preload.body {
                        let image_url = String::from_utf8(body.0.clone()).unwrap();
                        if (image_url.starts_with("http://") || image_url.starts_with("https://"))
                            && image_url.len() < 1000
                        {
                            return render(title, description, image_url, url, true);
                        }
                    }
                    render(
                        title,
                        description,
                        image_fallback_url.to_string(),
                        url,
                        false,
                    )
                } else {
                    preload_urls.push(nft_url);
                    Web4Response::preload_urls(preload_urls)
                }
            }
        }
    } else {
        render(
            title,
            description,
            image_fallback_url.to_string(),
            url,
            false,
        )
    }
}

#[near_bindgen]
impl Contract {
    #[allow(unused_variables)]
    pub fn web4_get(&self, request: Web4Request) -> Web4Response {
        let path = request.path.clone().expect("Path expected");

        if path == "/robots.txt" {
            return Web4Response::plain_response("User-agent: *\nDisallow:".to_string());
        }

        if path.starts_with("/nft/") {
            // NFT image
            let path = path[5..].split_once('/');
            if path.is_none() {
                return Web4Response::status(404);
            }
            let (contract_id, token_id) = path.unwrap();
            let contract_id = AccountId::from_str(contract_id).expect("Invalid NFT account ID");
            let nft_metadata_url =
                format!("/web4/contract/{}/nft_metadata", contract_id.to_string());
            let token_url = format!(
                "/web4/contract/{}/nft_token?token_id={}",
                contract_id.to_string(),
                token_id
            );
            let mut preload_urls = vec![nft_metadata_url.clone(), token_url.clone()];
            return if let Some(preloads) = request.preloads {
                let token: Token = near_sdk::serde_json::from_slice(
                    &preloads
                        .get(&token_url)
                        .unwrap()
                        .body
                        .as_ref()
                        .expect("Token not found")
                        .0,
                )
                .expect("Failed to parse token");
                let nft_metadata: NFTContractMetadata = near_sdk::serde_json::from_slice(
                    &preloads
                        .get(&nft_metadata_url)
                        .unwrap()
                        .body
                        .as_ref()
                        .expect("NFT Metadata doesn't exist")
                        .0,
                )
                .unwrap_or_else(|_| NFTContractMetadata {
                    spec: "nft-1.0.0".to_string(),
                    name: "NFT".to_string(),
                    symbol: "NFT".to_string(),
                    icon: None,
                    base_uri: None,
                    reference: None,
                    reference_hash: None,
                });
                let token_metadata = token.metadata.expect("Token metadata is missing");
                let token_media = token_metadata.media.clone().unwrap_or_default();

                if token_media.is_empty() && token_metadata.reference.is_some() {
                    let reference = token_metadata.reference.unwrap();
                    let url = if nft_metadata.base_uri == Some("https://arweave.net".to_string())
                        && !reference.starts_with("https://")
                    {
                        format!("https://arweave.net/{}", reference)
                    } else if reference.starts_with("https://") || reference.starts_with("http://")
                    {
                        reference
                    } else if reference.starts_with("ar://") {
                        format!("https://arweave.net/{}", &reference[5..])
                    } else {
                        "".to_string()
                    };
                    if !url.is_empty() {
                        if let Some(preload) = preloads.get(&url) {
                            if let Some(body) = &preload.body {
                                let reference: Reference =
                                    near_sdk::serde_json::from_slice(&body.0).unwrap_or_default();
                                if let Some(media) = reference.media {
                                    return Web4Response::plain_response(media);
                                }
                            }
                        } else {
                            preload_urls.push(url);
                            return Web4Response::preload_urls(preload_urls);
                        }
                    }
                }

                let image_url = if token_media.starts_with("https://")
                    || token_media.starts_with("http://")
                    || token_media.starts_with("data:image")
                {
                    token_media
                } else if let Some(base_uri) = &nft_metadata.base_uri {
                    format!("{}/{}", base_uri, token_media)
                } else if token_media.starts_with("Qm") || token_media.starts_with("ba") {
                    format!("{}/{}", IPFS_PREFIX, token_media)
                } else {
                    token_media
                };

                Web4Response::plain_response(image_url)
            } else {
                Web4Response::preload_urls(vec![nft_metadata_url, token_url])
            };
        }

        if path.starts_with("/u/") {
            // user profile
            let account_id = AccountId::from_str(&path[3..]).expect("Invalid account ID");
            let preload_urls = vec![make_account_url(&account_id)];
            return if let Some(profile) = get_profile(&account_id, &request) {
                let url = format!(
                    "https://near.social/#/mob.near/widget/ProfilePage?accountId={}",
                    account_id
                );
                let title = format!(
                    "{} | Near Social",
                    filter_string(profile.name.unwrap_or(account_id.to_string()))
                );
                let description = filter_string(
                    profile
                        .description
                        .unwrap_or(DEFAULT_ACCOUNT_DESCRIPTION.to_string()),
                );
                let image = profile.image.and_then(image_to_url);
                render_with_image(
                    title,
                    description,
                    image,
                    DEFAULT_ACCOUNT_IMAGE,
                    url,
                    preload_urls,
                    &request,
                )
            } else {
                Web4Response::preload_urls(preload_urls)
            };
        }

        if path.starts_with("/p/") || path.starts_with("/c/") {
            let is_post = path.starts_with("/p/");
            // user profile
            let path = path[3..].split_once('/');
            if path.is_none() {
                return Web4Response::status(404);
            }
            let (account_id, block_height) = path.unwrap();
            let account_id = AccountId::from_str(account_id).expect("Invalid account ID");
            let block_height = u64::from_str(block_height).expect("Invalid block height");
            let preload_urls = vec![
                make_account_url(&account_id),
                make_post_url(&account_id, block_height, is_post),
            ];
            return if let Some(post) = get_post(&account_id, block_height, is_post, &request) {
                let content: Option<Content> = near_sdk::serde_json::from_str(&post).ok();
                if let Some(content) = content {
                    let url = format!(
                        "https://near.social/#/mob.near/widget/{}?accountId={}&blockHeight={}",
                        if is_post {
                            "MainPage.Post.Page"
                        } else {
                            "MainPage.Comment.Page"
                        },
                        account_id,
                        block_height
                    );

                    let Profile {
                        name,
                        image: profile_image,
                        ..
                    } = get_profile(&account_id, &request).unwrap_or_default();

                    let title = format!(
                        "{} by {} | Near Social",
                        if is_post { "Post" } else { "Comment" },
                        filter_string(name.unwrap_or(account_id.to_string()))
                    );
                    let description =
                        filter_string(content.text.unwrap_or(DEFAULT_POST_DESCRIPTION.to_string()));
                    let image = content.image.or(profile_image).and_then(image_to_url);
                    render_with_image(
                        title,
                        description,
                        image,
                        DEFAULT_POST_IMAGE,
                        url,
                        preload_urls,
                        &request,
                    )
                } else {
                    Web4Response::status(404)
                }
            } else {
                Web4Response::preload_urls(preload_urls)
            };
        }

        let title = DEFAULT_TITLE.to_string();
        let description = DEFAULT_DESCRIPTION.to_string();
        let image = DEFAULT_IMAGE.to_string();

        render(
            title,
            description,
            image,
            "https://near.social".to_string(),
            true,
        )
    }
}
