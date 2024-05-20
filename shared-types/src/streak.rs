use super::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub enum StreakType {
    PRsOpened(u32),
    PRsMerged(u32),
    TotalScore(u32),
    AverageScoreGreaterThan(u32),
}

impl StreakType {
    pub fn from_prs_opened(value: u32) -> Self {
        Self::PRsOpened(value)
    }

    pub fn from_prs_merged(value: u32) -> Self {
        Self::PRsMerged(value)
    }

    pub fn is_streak_achieved(&self, user_period_data: &UserPeriodData) -> bool {
        match self {
            Self::PRsOpened(value) => user_period_data.prs_opened >= *value,
            Self::PRsMerged(value) => user_period_data.prs_merged >= *value,
            Self::TotalScore(score) => user_period_data.total_score >= *score,
            Self::AverageScoreGreaterThan(score) => {
                user_period_data.total_score / user_period_data.executed_prs > *score
            }
        }
    }
}

pub type StreakId = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct Streak {
    pub id: StreakId,
    pub time_period: TimePeriod,
    pub streak_criterias: Vec<StreakType>,
    pub is_active: bool,
}

impl Streak {
    pub fn new(streak_id: u64, time_period: TimePeriod, streak_criterias: Vec<StreakType>) -> Self {
        assert!(
            !streak_criterias.is_empty(),
            "Streak criteria should not be empty"
        );
        assert_ne!(
            time_period,
            TimePeriod::AllTime,
            "All time is not allowed for streaks"
        );

        Self {
            id: streak_id,
            time_period,
            streak_criterias,
            is_active: true,
        }
    }

    pub fn is_streak_achieved(&self, user_period_data: &UserPeriodData) -> bool {
        self.streak_criterias
            .iter()
            .all(|criteria| criteria.is_streak_achieved(user_period_data))
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, Default)]
#[serde(crate = "near_sdk::serde")]
#[borsh(crate = "near_sdk::borsh")]
pub struct StreakUserData {
    pub amount: u32,
    pub latest_time_string: TimePeriodString,
}
