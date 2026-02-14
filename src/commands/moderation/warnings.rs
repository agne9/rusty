use std::sync::Arc;

use twilight_http::Client;
use twilight_model::{gateway::payload::incoming::MessageCreate, guild::Permissions};
use twilight_util::builder::embed::EmbedBuilder;

use crate::commands::CommandMeta;
use crate::embed::embed::DEFAULT_EMBED_COLOR;
use crate::services::parse::parse_target_user_id;
use crate::services::permissions::has_message_permission;
use crate::services::warnings::{now_unix_secs, warnings_since};

pub const META: CommandMeta = CommandMeta {
    name: "warnings",
    desc: "Show warning history for a user in a time window.",
    category: "moderation",
    usage: "!warnings <user> [days]",
};

const DEFAULT_DAYS: u64 = 30;

pub async fn run(
    http: Arc<Client>,
    msg: Box<MessageCreate>,
    arg1: Option<&str>,
    arg_tail: Option<&str>,
) -> anyhow::Result<()> {
    let Some(_guild_id) = msg.guild_id else {
        http.create_message(msg.channel_id)
            .content("This command only works in servers.")
            .await?;
        return Ok(());
    };

    if !has_message_permission(&http, &msg, Permissions::MANAGE_MESSAGES).await? {
        http.create_message(msg.channel_id)
            .content("You are not permitted to use this command.")
            .await?;
        return Ok(());
    }

    let Some(raw_target) = arg1 else {
        let usage = format!("Usage: `{}`", META.usage);
        http.create_message(msg.channel_id).content(&usage).await?;
        return Ok(());
    };

    let Some(target_user_id) = parse_target_user_id(raw_target) else {
        let usage = format!("Usage: `{}`", META.usage);
        http.create_message(msg.channel_id).content(&usage).await?;
        return Ok(());
    };

    let days = parse_days(arg_tail).unwrap_or(DEFAULT_DAYS);
    let since = now_unix_secs().saturating_sub(days.saturating_mul(86_400));

    let entries = warnings_since(target_user_id, since).await;
    let count = entries.len();

    let mut description = format!("Total warnings in last {} day(s): **{}**\n\n", days, count);

    if entries.is_empty() {
        description.push_str("No warnings in this period.");
    } else {
        let start = entries.len().saturating_sub(5);
        for (index, entry) in entries.iter().enumerate().skip(start) {
            let line = format!(
                "#{idx} • <t:{ts}:F> • by <@{mod_id}>\nReason: {reason}\n\n",
                idx = index + 1,
                ts = entry.warned_at,
                mod_id = entry.moderator_id,
                reason = sanitize_reason(&entry.reason)
            );
            description.push_str(&line);
        }
    }

    let embed = EmbedBuilder::new()
        .title(format!("Warnings for User {}", target_user_id.get()))
        .color(DEFAULT_EMBED_COLOR)
        .description(description)
        .validate()?
        .build();

    http.create_message(msg.channel_id).embeds(&[embed]).await?;

    Ok(())
}

fn parse_days(arg_tail: Option<&str>) -> Option<u64> {
    let raw = arg_tail?.split_whitespace().next()?;
    let days = raw.parse::<u64>().ok()?;
    if days == 0 {
        return None;
    }

    Some(days)
}

fn sanitize_reason(reason: &str) -> String {
    reason.replace('@', "@\u{200B}")
}
