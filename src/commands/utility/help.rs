use std::sync::Arc;
use twilight_http::Client;
use twilight_model::gateway::payload::incoming::MessageCreate;

use crate::commands::{CommandMeta, COMMANDS};

pub const META: CommandMeta = CommandMeta {
    name: "help",
    desc: "Lists out all available commands.",
    category: "utility",
};

pub async fn run(http: Arc<Client>, msg: Box<MessageCreate>) -> anyhow::Result<()> {
    let mut out = String::from("**Available commands:**\n");

    for cmd in COMMANDS {
        // Output example: "!help - Lists out all available commands. - (utility)"
        out.push_str(&format!("!{} - {} ({})\n", cmd.name, cmd.desc, cmd.category));
    }
    
    http.create_message(msg.channel_id)
    .content(&out)
    .await?;

    Ok(())
}
