use std::sync::Arc;

use twilight_http::Client;
use twilight_http::request::AuditLogReason as _;
use twilight_model::{gateway::payload::incoming::MessageCreate, guild::Permissions};

use crate::commands::CommandMeta;
use crate::services::parse::parse_target_user_id;
use crate::services::permissions::has_message_permission;

pub const META: CommandMeta = CommandMeta {
    name: "ban",
    desc: "Ban a user from the server.",
    category: "moderation",
    usage: "!ban <user> [reason]",
};

pub async fn run(
    http: Arc<Client>,
    msg: Box<MessageCreate>,
    arg1: Option<&str>,
    arg_tail: Option<&str>,
) -> anyhow::Result<()> {
    let Some(guild_id) = msg.guild_id else {
        http.create_message(msg.channel_id)
            .content("This command only works in servers.")
            .await?;
        return Ok(());
    };

    if !has_message_permission(&http, &msg, Permissions::BAN_MEMBERS).await? {
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

    if target_user_id == msg.author.id {
        http.create_message(msg.channel_id)
            .content("You can't ban yourself.")
            .await?;
        return Ok(());
    }

    let mut request = http.create_ban(guild_id, target_user_id);
    if let Some(reason) = arg_tail {
        request = request.reason(reason);
    }

    if request.await.is_err() {
        http.create_message(msg.channel_id)
            .content("I couldn't ban that user. Check role hierarchy and permissions.")
            .await?;
        return Ok(());
    }

    let out = format!("Banned <@{}>.", target_user_id.get());
    http.create_message(msg.channel_id).content(&out).await?;

    Ok(())
}
