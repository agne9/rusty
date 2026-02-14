pub mod moderation;
pub mod utility;

use std::sync::Arc;
use twilight_http::Client;
use twilight_model::{
    application::interaction::InteractionData,
    gateway::payload::incoming::{InteractionCreate, MessageCreate},
};

#[derive(Clone, Copy)]
enum InteractionRoute {
    PermissionsButtons,
    HelpButtons,
    PagetestButtons,
    PermissionsModal,
    HelpModal,
    PagetestModal,
}

fn route_interaction(custom_id: &str) -> Option<InteractionRoute> {
    const ROUTES: [(&str, InteractionRoute); 6] = [
        ("pg:permissions:", InteractionRoute::PermissionsButtons),
        ("pg:help", InteractionRoute::HelpButtons),
        ("pg:pagetest:", InteractionRoute::PagetestButtons),
        ("pgm:permissions:", InteractionRoute::PermissionsModal),
        ("pgm:help", InteractionRoute::HelpModal),
        ("pgm:pagetest:", InteractionRoute::PagetestModal),
    ];

    ROUTES
        .into_iter()
        .find_map(|(prefix, route)| custom_id.starts_with(prefix).then_some(route))
}

// Global command meta data
pub struct CommandMeta {
    pub name: &'static str,
    pub desc: &'static str,
    pub category: &'static str,
    pub usage: &'static str,
}

pub const COMMANDS: &[CommandMeta] = &[
    utility::ping::META,
    utility::universe::META,
    utility::help::META,
    utility::usage::META,
    utility::pagetest::META,
    moderation::purge::META,
    moderation::permissions::META,
    // Add new commands here
];

const PREFIX: char = '!'; // Command Prefix Character

pub async fn handle_message(http: Arc<Client>, msg: Box<MessageCreate>) -> anyhow::Result<()> {
    if msg.author.bot {
        return Ok(());
    }

    if !msg.content.starts_with(PREFIX) {
        return Ok(());
    }

    let content = msg.content.to_ascii_lowercase();
    let mut parts = content.split_whitespace(); // Split msg into parts based on it's whitespaces
    let raw = parts.next().unwrap_or(""); // Take the first piece (command), or empty string if missing
    let cmd = raw.trim_start_matches('!'); // Remove prefix
    let arg1 = parts.next(); // Take first arg after command

    match cmd {
        "ping" => utility::ping::run(http, msg).await?,
        "universe" => utility::universe::run(http, msg).await?,
        "help" => utility::help::run(http, msg, arg1).await?,
        "usage" => utility::usage::run(http, msg, arg1).await?,
        "pagetest" => utility::pagetest::run(http, msg, arg1).await?,

        "permissions" => moderation::permissions::run(http, msg, arg1).await?,
        "purge" => moderation::purge::run(http, msg, arg1).await?,
        // Add new commands here
        _ => {}
    }

    Ok(())
}

pub async fn handle_interaction(
    http: Arc<Client>,
    interaction: Box<InteractionCreate>,
) -> anyhow::Result<()> {
    let custom_id = match interaction.data.as_ref() {
        Some(InteractionData::MessageComponent(data)) => data.custom_id.clone(),
        Some(InteractionData::ModalSubmit(data)) => data.custom_id.clone(),
        _ => return Ok(()),
    };

    let Some(route) = route_interaction(&custom_id) else {
        return Ok(());
    };

    match route {
        InteractionRoute::PermissionsButtons => {
            let _handled =
                moderation::permissions::handle_pagination_interaction(http, interaction).await?;
        }
        InteractionRoute::HelpButtons => {
            let _handled = utility::help::handle_pagination_interaction(http, interaction).await?;
        }
        InteractionRoute::PagetestButtons => {
            let _handled =
                utility::pagetest::handle_pagination_interaction(http, interaction).await?;
        }
        InteractionRoute::PermissionsModal => {
            let _handled =
                moderation::permissions::handle_pagination_modal_interaction(http, interaction)
                    .await?;
        }
        InteractionRoute::HelpModal => {
            let _handled =
                utility::help::handle_pagination_modal_interaction(http, interaction).await?;
        }
        InteractionRoute::PagetestModal => {
            let _handled =
                utility::pagetest::handle_pagination_modal_interaction(http, interaction).await?;
        }
    }

    Ok(())
}
