use twilight_http::Client;
use twilight_model::id::{
    Id,
    marker::{ChannelMarker, UserMarker},
};

use crate::embed::embed::build_moderation_action_embed;

/// Send a standardized moderation-action embed without pinging the target user.
pub async fn send_moderation_action_embed(
    http: &Client,
    channel_id: Id<ChannelMarker>,
    target_user_id: Id<UserMarker>,
    action_past_tense: &str,
    reason: Option<&str>,
    duration: Option<&str>,
) -> anyhow::Result<()> {
    let target_profile = fetch_target_profile(http, target_user_id).await;
    let display_name = target_profile
        .as_ref()
        .map(|profile| profile.display_name.as_str())
        .unwrap_or("Unknown User");
    let header = format!("{} has been {}", display_name, action_past_tense);
    let reason = reason
        .map(sanitize_reason)
        .unwrap_or_else(|| "No reason provided".to_owned());
    let icon_url = target_profile
        .as_ref()
        .and_then(|profile| profile.avatar_url.as_deref());

    let embed = build_moderation_action_embed(&header, &reason, duration, icon_url)?;

    http.create_message(channel_id).embeds(&[embed]).await?;

    Ok(())
}

fn sanitize_reason(reason: &str) -> String {
    reason.replace('@', "@\u{200B}")
}

struct TargetProfile {
    display_name: String,
    avatar_url: Option<String>,
}

async fn fetch_target_profile(http: &Client, user_id: Id<UserMarker>) -> Option<TargetProfile> {
    let user = http.user(user_id).await.ok()?.model().await.ok()?;
    let display_name = user.global_name.unwrap_or(user.name);
    let avatar_url = Some(match user.avatar {
        Some(avatar) => format!(
            "https://cdn.discordapp.com/avatars/{}/{}.png?size=128",
            user_id.get(),
            avatar
        ),
        None => {
            let default_avatar_index = (user_id.get() >> 22) % 6;
            format!(
                "https://cdn.discordapp.com/embed/avatars/{}.png",
                default_avatar_index
            )
        }
    });

    Some(TargetProfile {
        display_name,
        avatar_url,
    })
}
