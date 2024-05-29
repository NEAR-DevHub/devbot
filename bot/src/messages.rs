use rand::seq::SliceRandom;
use rand::thread_rng;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use tracing::error;

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MsgCategory {
    IncludeBasicMessage,
    CorrectNonzeroScoringMessage,
    CorrectZeroScoringMessage,
    CorrectableScoringMessage,
    ExcludeMessages,
    PauseMessage,
    UnpauseMessage,
    MergeWithoutScoreMessage,
    FinalMessage,
    StaleMessage,
    ErrorUnknownCommandMessage,
    ErrorRightsViolationMessage,
    ErrorLateIncludeMessage,
    ErrorPausePausedMessage,
    ErrorUnpauseUnpausedMessage,
    // TODO: currently unused
    ErrorPausedMessage,
    ErrorLateScoringMessage,
    ErrorSelfScore,
    ErrorOrgNotInAllowedListMessage,
}

impl std::fmt::Display for MsgCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Messages {
    message: Vec<String>,
    variables: HashSet<String>,
}

impl Messages {
    pub fn new(message: Vec<String>, variables: HashSet<String>) -> Self {
        Self { message, variables }
    }

    pub fn format(&self, values: HashMap<String, String>) -> anyhow::Result<String> {
        let mut formatted_message = self
            .message
            .choose(&mut thread_rng())
            .ok_or_else(|| anyhow::anyhow!("Failed to choose randomly an message"))?
            .clone();
        for key in self.variables.iter() {
            if let Some(value) = values.get(key.as_str()) {
                formatted_message = formatted_message.replace(&format!("{{{}}}", key), value);
            } else {
                error!(
                    "The message expects a variable: {}, but it wasn't provided",
                    key
                );
            }
        }
        Ok(formatted_message)
    }

    fn partial_format(&mut self, values: &HashMap<String, String>) {
        for message in self.message.iter_mut() {
            for (key, value) in values {
                *message = message.replace(&format!("{{{key}}}"), value);
                self.variables.remove(key);
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MessageLoader {
    pub link: String,
    pub leaderboard_link: String,
    pub form: String,
    pub picture_api_link: String,

    pub include_basic_messages: Messages,
    pub correct_nonzero_scoring_messages: Messages,
    pub correct_zero_scoring_messages: Messages,
    pub correctable_scoring_messages: Messages,
    pub exclude_messages: Messages,
    pub pause_messages: Messages,
    pub unpause_messages: Messages,
    pub merge_without_score_messages: Messages,
    pub final_messages: Messages,
    pub stale_messages: Messages,
    pub error_unknown_command_messages: Messages,
    pub error_rights_violation_messages: Messages,
    pub error_late_include_messages: Messages,
    pub error_late_scoring_messages: Messages,
    pub error_pause_paused_messages: Messages,
    pub error_unpause_unpaused_messages: Messages,
    pub error_paused_messages: Messages,
    pub error_selfscore_messages: Messages,
    pub error_org_not_in_allowed_list_messages: Messages,
}

impl MessageLoader {
    pub fn load_from_file(file_path: &PathBuf, bot_name: &str) -> anyhow::Result<Self> {
        let file_content = fs::read_to_string(file_path)?;
        let mut result: Self = toml::from_str(&file_content)?;
        result.postprocess_messages_with_link(bot_name);
        tracing::trace!("Loaded messages: {:#?}", result);
        Ok(result)
    }

    fn postprocess_messages_with_link(&mut self, bot_name: &str) {
        let values = vec![
            ("link".to_string(), self.link.clone()),
            (
                "leaderboard_link".to_string(),
                self.leaderboard_link.clone(),
            ),
            ("bot_name".to_string(), bot_name.to_string()),
            ("form".to_string(), self.form.clone()),
            (
                "picture_api_link".to_string(),
                self.picture_api_link.clone(),
            ),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        let array_of_messages = vec![
            &mut self.include_basic_messages,
            &mut self.correct_nonzero_scoring_messages,
            &mut self.correct_zero_scoring_messages,
            &mut self.correctable_scoring_messages,
            &mut self.exclude_messages,
            &mut self.pause_messages,
            &mut self.unpause_messages,
            &mut self.merge_without_score_messages,
            &mut self.final_messages,
            &mut self.stale_messages,
            &mut self.error_unknown_command_messages,
            &mut self.error_rights_violation_messages,
            &mut self.error_late_include_messages,
            &mut self.error_late_scoring_messages,
            &mut self.error_pause_paused_messages,
            &mut self.error_unpause_unpaused_messages,
            &mut self.error_paused_messages,
            &mut self.error_selfscore_messages,
            &mut self.error_org_not_in_allowed_list_messages,
        ];
        for message in array_of_messages {
            message.partial_format(&values);
        }
    }

    pub fn get_message(&self, category: MsgCategory) -> Messages {
        let elem = match category {
            MsgCategory::IncludeBasicMessage => &self.include_basic_messages,
            MsgCategory::CorrectNonzeroScoringMessage => &self.correct_nonzero_scoring_messages,
            MsgCategory::CorrectZeroScoringMessage => &self.correct_zero_scoring_messages,
            MsgCategory::CorrectableScoringMessage => &self.correctable_scoring_messages,
            MsgCategory::ExcludeMessages => &self.exclude_messages,
            MsgCategory::PauseMessage => &self.pause_messages,
            MsgCategory::UnpauseMessage => &self.unpause_messages,
            MsgCategory::MergeWithoutScoreMessage => &self.merge_without_score_messages,
            MsgCategory::FinalMessage => &self.final_messages,
            MsgCategory::StaleMessage => &self.stale_messages,
            MsgCategory::ErrorUnknownCommandMessage => &self.error_unknown_command_messages,
            MsgCategory::ErrorRightsViolationMessage => &self.error_rights_violation_messages,
            MsgCategory::ErrorLateIncludeMessage => &self.error_late_include_messages,
            MsgCategory::ErrorLateScoringMessage => &self.error_late_scoring_messages,
            MsgCategory::ErrorSelfScore => &self.error_selfscore_messages,
            MsgCategory::ErrorOrgNotInAllowedListMessage => {
                &self.error_org_not_in_allowed_list_messages
            }
            MsgCategory::ErrorPausePausedMessage => &self.error_pause_paused_messages,
            MsgCategory::ErrorUnpauseUnpausedMessage => &self.error_unpause_unpaused_messages,
            MsgCategory::ErrorPausedMessage => &self.error_paused_messages,
        };
        elem.clone()
    }
}
