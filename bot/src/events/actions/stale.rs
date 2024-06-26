use tracing::{instrument, warn};

use shared::{github::PrMetadata, PRInfo};

use crate::{events::Context, messages::MsgCategory};

#[derive(Debug, Clone)]
pub struct PullRequestStale {}

impl PullRequestStale {
    #[instrument(skip(self, context, check_info), fields(pr = pr.full_id))]
    pub async fn execute(
        &self,
        pr: &PrMetadata,
        context: Context,
        check_info: PRInfo,
    ) -> anyhow::Result<bool> {
        if check_info.merged {
            warn!("PR {} is already merged. Skipping", pr.full_id);
            return Ok(false);
        }

        context.near.send_stale(pr).await?;
        if !check_info.allowed_repo {
            return Ok(false);
        }

        if pr.closed {
            return Ok(true);
        }

        context
            .reply(pr, None, MsgCategory::StaleMessage, vec![])
            .await?;
        Ok(true)
    }
}
