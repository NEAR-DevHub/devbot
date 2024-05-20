use near_sdk::{
    borsh::{BorshDeserialize, BorshSerialize},
    serde::{Deserialize, Serialize},
    NearSchema,
};

mod pr;
mod streak;
mod timeperiod;

pub use pr::*;
pub use streak::*;
pub use timeperiod::*;

pub type GithubHandle = String;

#[derive(
    Debug,
    Clone,
    BorshDeserialize,
    BorshSerialize,
    Serialize,
    Deserialize,
    NearSchema,
    PartialEq,
    Default,
)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct UserPeriodData {
    pub total_score: u32,
    pub executed_prs: u32,
    pub prs_opened: u32,
    pub prs_merged: u32,
}
