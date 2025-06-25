use std::{
    cmp::min,
    collections::HashMap,
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use poise::serenity_prelude::{ChannelId, Color, Context, CreateEmbed, CreateMessage};
use tokio::time::{sleep_until, Instant};

use crate::rng::RobloxRng;

const UPDATE_INTERVAL: u64 = 300;

struct EggType {
    name: &'static str,
    spawn_chance: f64,
    stock_limit: Option<u32>,
    emoji: &'static str,
    role: Option<&'static str>,
}

const EGG_TYPES: [EggType; 7] = [
    EggType {
        name: "Common Egg",
        spawn_chance: 1.0,
        stock_limit: None,
        emoji: "<:common_egg:1386442268700835960>",
        role: None,
    },
    EggType {
        name: "Uncommon Egg",
        spawn_chance: 0.75,
        stock_limit: None,
        emoji: "<:uncommon_egg:1386442310480302100>",
        role: None,
    },
    EggType {
        name: "Rare Egg",
        spawn_chance: 0.35,
        stock_limit: None,
        emoji: "<:rare_egg:1386442308391538778>",
        role: Some("<@&1387430341987401849>"),
    },
    EggType {
        name: "Epic Egg",
        spawn_chance: 0.15,
        stock_limit: None,
        emoji: "<:epic_egg:1386442299478900766>",
        role: Some("<@&1387430557385883840>"),
    },
    EggType {
        name: "Legendary Egg",
        spawn_chance: 0.05,
        stock_limit: None,
        emoji: "<:legendary_egg:1386442303429808258>",
        role: Some("<@&1387432053166968993>"),
    },
    EggType {
        name: "Mythical Egg",
        spawn_chance: 0.0015,
        stock_limit: Some(2),
        emoji: "<:mythical_egg:1386442306210627605>",
        role: Some("<@&1387432235560603688>"),
    },
    EggType {
        name: "Celestial Egg",
        spawn_chance: 0.0001,
        stock_limit: Some(1),
        emoji: "<:celestial_egg:1386442297943527485>",
        role: Some("<@&1387432340808269954>"),
    },
];

pub fn start_stock_loop(ctx: Context) {
    tokio::spawn(async move {
        let channel_id: u64 = env::var("STOCK_CHANNEL")
            .expect("No STOCK_CHANNEL found")
            .parse()
            .expect("STOCK_CHANNEL must be a valid Discord channel ID");

        let channel = ChannelId::new(channel_id);

        loop {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards..?");

            let secs = now.as_secs();
            let next_interval = (secs / UPDATE_INTERVAL + 1) * UPDATE_INTERVAL;
            let wait_duration = Duration::from_secs(next_interval - secs);

            // Wait until the current minute of the clock is a multiple of 5
            sleep_until(Instant::now() + wait_duration).await;

            // Get current time to compute the seed for this iteration
            let now_after = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards..?");
            let after_secs = now_after.as_secs();

            // Generate stock
            let seed: u64 = after_secs - (after_secs % UPDATE_INTERVAL);
            let mut stock = HashMap::new();

            for (i, egg) in EGG_TYPES.iter().enumerate() {
                let mut rng = RobloxRng::new(seed + i as u64);

                let roll = rng.next_f64();
                if roll >= egg.spawn_chance {
                    stock.insert(egg.name, 0);
                    continue;
                }

                let base_quantity = rng.next_range(1, 5);
                let quantity = match egg.stock_limit {
                    Some(limit) => min(base_quantity, limit),
                    None => base_quantity,
                };

                stock.insert(egg.name, quantity);
            }

            // Format stock in embed and ping roles
            let mut stock_lines = String::new();
            let mut role_mentions = String::new();

            for egg in &EGG_TYPES {
                if let Some(&count) = stock.get(egg.name) {
                    if count > 0 {
                        stock_lines
                            .push_str(&format!("{} {} - **{}**\n", egg.emoji, egg.name, count));

                        // Add roles to ping in the message
                        if let Some(role) = egg.role {
                            if !role_mentions.is_empty() {
                                role_mentions.push(' ');
                            }
                            role_mentions.push_str(role);
                        }
                    }
                }
            }

            // Create and send embed
            let embed = CreateEmbed::new()
                .title("🥚 Egg Stock")
                .color(Color::RED)
                .description(stock_lines);

            let mut builder = CreateMessage::new().embed(embed);
            if !role_mentions.is_empty() {
                builder = builder.content(role_mentions);
            }

            if let Err(err) = channel.send_message(&ctx.http, builder).await {
                eprintln!("Failed to send message: {:?}", err);
            } else {
                println!("Generated and sent stock message");
            }
        }
    });
}
