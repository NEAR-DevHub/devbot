use chrono::DateTime;
use octocrab::models::{activity::Notification, pulls::PullRequest};

use crate::commands::BotCommand;

mod types;
pub use types::*;

#[derive(Clone)]
pub struct GithubClient {
    octocrab: octocrab::Octocrab,
    pub user_handle: String,
}

impl GithubClient {
    pub fn new(github_token: String, user_handle: String) -> anyhow::Result<Self> {
        let octocrab = octocrab::Octocrab::builder()
            .personal_token(github_token)
            .build()?;

        Ok(Self {
            octocrab,
            user_handle,
        })
    }

    pub async fn get_events(
        &self,
        since: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<(Vec<Event>, DateTime<chrono::Utc>)> {
        log::debug!("Getting mentions since: {:?}", since);
        let page = self
            .octocrab
            .activity()
            .notifications()
            .list()
            .all(true)
            .participating(true)
            .per_page(50)
            .since(since)
            .page(0)
            .send()
            .await?;

        let mut updated_at = since;
        let events = self.octocrab.all_pages(page).await?;
        let interested_events = events.into_iter().filter(|notification| {
            updated_at = updated_at.max(notification.updated_at);

            notification.subject.r#type == "PullRequest"
                && (notification.reason == "mention" || notification.reason == "state_change")
        });

        let mut results = Vec::new();

        for event in interested_events {
            let pr = self.get_pull_request(&event).await;
            if pr.is_err() {
                log::warn!("Failed to get PR: {:?}", pr.err());
                continue;
            }
            let pr = pr.unwrap();
            let pr_metadata = types::PrMetadata::try_from(pr);
            if pr_metadata.is_err() {
                log::warn!("Failed to convert PR: {:?}", pr_metadata.err());
                continue;
            }
            let pr_metadata = pr_metadata.unwrap();

            let comments = self
                .octocrab
                .issues(&pr_metadata.owner, &pr_metadata.repo)
                .list_comments(pr_metadata.number)
                .per_page(100)
                .send()
                .await;

            if comments.is_err() {
                log::warn!("Failed to get comments: {:?}", comments.err());
                continue;
            }
            let comments = comments.unwrap();

            // TODO: think if we can avoid this and just load from the last page
            let comments = self.octocrab.all_pages(comments).await;
            if comments.is_err() {
                log::warn!("Failed to get all comments: {:?}", comments.err());
                continue;
            }
            let comments = comments.unwrap();

            for comment in comments.into_iter().rev() {
                if comment.user.login == self.user_handle {
                    // We have replied to last ask
                    break;
                }

                let event = Event::parse_comment(&self.user_handle, &event, &pr_metadata, &comment);
                if event.is_none() {
                    continue;
                }
                results.push(event.unwrap());
            }

            match event.reason.as_str() {
                // We already handled it above
                "mention" => {}
                "state_change" => {
                    if pr_metadata.merged.is_some() {
                        results.push(Event::PullRequestMerged(PullRequestMerged {
                            pr_metadata,
                            timestamp: event.updated_at,
                        }));
                    }
                }
                _ => unreachable!("Checked in the filter above"),
            }
        }

        Ok((results, updated_at))
    }

    pub async fn get_pull_request(
        &self,
        notification: &Notification,
    ) -> anyhow::Result<PullRequest> {
        assert_eq!(notification.subject.r#type, "PullRequest");
        log::debug!(
            "Getting PR: {:?}",
            notification.subject.url.as_ref().unwrap()
        );
        let pull_request = self
            .octocrab
            .get(
                notification
                    .subject
                    .url
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("No PR url"))?,
                None::<&()>,
            )
            .await?;

        Ok(pull_request)
    }

    pub async fn reply(&self, owner: &str, repo: &str, id: u64, text: &str) -> anyhow::Result<()> {
        self.octocrab
            .issues(owner, repo)
            .create_comment(id, text)
            .await?;

        Ok(())
    }

    pub async fn like_comment(
        &self,
        owner: &str,
        repo: &str,
        comment_id: u64,
    ) -> anyhow::Result<()> {
        self.octocrab
            .issues(owner, repo)
            .create_comment_reaction(
                comment_id,
                octocrab::models::reactions::ReactionContent::PlusOne,
            )
            .await?;

        Ok(())
    }

    pub async fn like_pr(&self, owner: &str, repo: &str, pr_number: u64) -> anyhow::Result<()> {
        self.octocrab
            .issues(owner, repo)
            .create_reaction(
                pr_number,
                octocrab::models::reactions::ReactionContent::PlusOne,
            )
            .await?;

        Ok(())
    }
}
