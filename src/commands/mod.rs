pub mod moderation;
pub mod utility;

use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::payload::incoming::MessageCreate;

pub struct CommandMeta {
    pub name: &'static str,
    pub desc: &'static str,
    pub category: &'static str,
}

pub const COMMANDS: &[CommandMeta] = &[
    // Utility
    utility::ping::META,
    utility::help::META,
    // Add new commands here
];

const PREFIX: char = '!';

pub async fn handle_message(http: Arc<Client>, msg: Box<MessageCreate>) -> anyhow::Result<()> {
    if msg.author.bot {
        return Ok(());
    }

    if !msg.content.starts_with(PREFIX) {
        return Ok(());
    }

    let mut parts = msg.content.split_whitespace();
    let cmd = parts.next().unwrap_or("").to_ascii_lowercase();

    match cmd.as_str() {
        "!ping" => utility::ping::run(http, msg).await?,
        "!help" => utility::help::run(http, msg).await?,
        // Add new commands here

        _ => {}
    }

    Ok(())
}