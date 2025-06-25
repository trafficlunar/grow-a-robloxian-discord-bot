use std::{env, fs};

use poise::serenity_prelude::{ChannelId, CreateMessage};

use crate::{Context, Error};

const RULES_FILE_PATH: &str = "data/rules.md";

#[poise::command(prefix_command, required_permissions = "ADMINISTRATOR")]
pub async fn rules(ctx: Context<'_>) -> Result<(), Error> {
    // Read the rules from the file
    let rules_text = match fs::read_to_string(RULES_FILE_PATH) {
        Ok(text) => text,
        Err(err) => {
            ctx.say(format!("Failed to read rules file: {}", err))
                .await?;
            return Ok(());
        }
    };

    // Send to the target channel
    let channel_id: u64 = env::var("RULES_CHANNEL")
        .expect("No RULES_CHANNEL found")
        .parse()
        .expect("RULES_CHANNEL must be a valid Discord channel ID");

    let channel = ChannelId::new(channel_id);
    let builder = CreateMessage::new().content(rules_text);

    if let Err(err) = channel.send_message(ctx.http(), builder).await {
        eprintln!("Failed to send message: {:?}", err);
    } else {
        ctx.say("Rules posted successfully.").await?;
    }

    Ok(())
}
