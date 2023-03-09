use crate::*;

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct Social(pub HashMap<AccountId, AccountData>);

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountData {
    pub profile: Option<Profile>,
    pub post: Option<Post>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Profile {
    pub name: Option<String>,
    pub description: Option<String>,
    pub image: Option<Image>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Post {
    pub main: Option<String>,
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Content {
    pub text: Option<String>,
    pub image: Option<Image>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Image {
    pub url: Option<String>,
    pub ipfs_cid: Option<String>,
    pub nft: Option<Nft>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Nft {
    #[serde(rename = "tokenId")]
    pub token_id: Option<String>,
    #[serde(rename = "contractId")]
    pub contract_id: Option<String>,
}

pub enum UrlOrNft {
    Url(String),
    Nft(String, String),
}

pub(crate) fn image_to_url(image: Image) -> Option<UrlOrNft> {
    if let Some(url) = &image.url {
        Some(UrlOrNft::Url(url.clone()))
    } else if let Some(ipfs_cid) = &image.ipfs_cid {
        Some(UrlOrNft::Url(format!(
            "https://ipfs.near.social/ipfs/{}",
            ipfs_cid
        )))
    } else if let Some(nft) = &image.nft {
        let token_id = nft.token_id.clone()?;
        let contract_id = nft.contract_id.clone()?;
        Some(UrlOrNft::Nft(contract_id, token_id))
    } else {
        None
    }
}
