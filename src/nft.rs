use crate::*;

use near_contract_standards::non_fungible_token::metadata::TokenMetadata;

#[derive(Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub owner_id: AccountId,
    pub metadata: Option<TokenMetadata>,
}

#[derive(Deserialize, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct Reference {
    pub media: Option<String>,
}
