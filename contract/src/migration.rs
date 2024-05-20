use self::storage::StorageKey;

use super::*;

// We need to carefully think what we want to store in the contract storage
#[derive(Debug, Clone, BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct UserData {
    pub handle: String,
    pub total_prs_merged: u32,
    pub total_prs_opened: u32,
    // Created for the future, but we would need to think more about it
    pub account_id: Option<AccountId>,
}

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct OldState {
    sloth: AccountId,
    #[allow(deprecated)]
    sloths: UnorderedMap<String, UserData>,
    #[allow(deprecated)]
    sloths_per_month: UnorderedMap<(String, TimePeriodString), u32>,
    #[allow(deprecated)]
    organizations: UnorderedMap<String, Organization>,
    // We need to think about removing PRs that are stale for a long time
    #[allow(deprecated)]
    prs: UnorderedMap<String, PR>,
    #[allow(deprecated)]
    executed_prs: UnorderedMap<String, PR>,
    excluded_prs: LookupSet<String>,
}
#[near_bindgen]
impl Contract {
    #[init(ignore_state)]
    pub fn migrate() -> Self {
        let mut old_state = env::state_read::<OldState>().unwrap();
        old_state.sloths_per_month.clear();
        old_state.sloths.clear();

        let sloths_per_period = UnorderedMap::new(StorageKey::SlothsPerPeriod);
        let mut contract = Self {
            sloth: old_state.sloth,
            sloths_per_period,
            organizations: old_state.organizations,
            prs: old_state.prs,
            executed_prs: old_state.executed_prs,
            excluded_prs: old_state.excluded_prs,
            streaks: Vector::new(StorageKey::Streaks),
            user_streaks: UnorderedMap::new(StorageKey::UserStreaks),
        };

        for value in contract.executed_prs.values().cloned().collect::<Vec<_>>() {
            contract.count_score_to_periods(&value, value.score().unwrap());
        }

        for user in old_state.sloths.values() {
            contract.calculate_streak(&user.handle);
        }

        contract
    }
}
