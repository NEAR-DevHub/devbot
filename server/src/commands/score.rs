use self::api::github::{BotScored, PrMetadata, User};

use super::*;

#[async_trait::async_trait]
impl Execute for api::github::BotScored {
    async fn execute(&self, context: Context) -> anyhow::Result<()> {
        let info = context.check_info(&self.pr_metadata).await?;
        if !info.allowed || !info.exist || info.executed {
            return context
                .github
                .mark_notification_as_read(self.notification_id)
                .await;
        }

        let score = self.score.parse::<u8>()?;
        if score < 1 || score > 10 {
            context
                .reply_with_error(
                    &self.pr_metadata.owner,
                    &self.pr_metadata.repo,
                    self.pr_metadata.number,
                    "Score should be between 1 and 10",
                )
                .await?;
            return context
                .github
                .mark_notification_as_read(self.notification_id)
                .await;
        }

        if self.pr_metadata.author.login == self.sender.login || !self.sender.is_maintainer() {
            context
                .reply_with_error(
                    &self.pr_metadata.owner,
                    &self.pr_metadata.repo,
                    self.pr_metadata.number,
                    "Only maintainers can score PRs, and you can't score your own PRs.",
                )
                .await?;
            return context
                .github
                .mark_notification_as_read(self.notification_id)
                .await;
        }

        context
            .near
            .send_scored(&self.pr_metadata, &self.sender.login, score as u64)
            .await?;

        context
            .github
            .reply(
                &self.pr_metadata.owner,
                &self.pr_metadata.repo,
                self.pr_metadata.number,
                "Thanks for submitting your score for the Sloth race.",
            )
            .await?;
        context
            .github
            .like_comment(
                &self.pr_metadata.owner,
                &self.pr_metadata.repo,
                self.comment_id,
            )
            .await?;
        context
            .github
            .mark_notification_as_read(self.notification_id)
            .await
    }
}

impl ParseComment for api::github::BotScored {
    type Command = api::github::BotScored;

    fn parse_comment(
        bot_name: &str,
        notification: &Notification,
        pr_metadata: &PrMetadata,
        comment: &Comment,
    ) -> Option<Self::Command> {
        let body = comment
            .body
            .as_ref()
            .or(comment.body_html.as_ref())
            .or(comment.body_text.as_ref())
            .cloned()
            .unwrap_or_default();

        let phrase = format!("@{} score", bot_name);
        if let Some(result) = body.find(&phrase) {
            Some(BotScored::new(
                User {
                    login: comment.user.login.clone(),
                    contributor_type: comment.author_association.clone(),
                },
                pr_metadata.clone(),
                body[result + phrase.len()..].trim().to_string(),
                notification.updated_at,
                comment.id.0,
                notification.id.0,
            ))
        } else {
            None
        }
    }
}
