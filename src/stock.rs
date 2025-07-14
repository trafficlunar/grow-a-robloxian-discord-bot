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

struct ItemType {
    name: &'static str,
    spawn_chance: f64,
    stock_limit: Option<u32>,
    emoji: &'static str,
    role: Option<&'static str>,
}

const EGGS: [ItemType; 9] = [
    ItemType {
        name: "Common Egg",
        spawn_chance: 1.0,
        stock_limit: None,
        emoji: "<:common_egg:1386442268700835960>",
        role: None,
    },
    ItemType {
        name: "Uncommon Egg",
        spawn_chance: 0.75,
        stock_limit: None,
        emoji: "<:uncommon_egg:1386442310480302100>",
        role: None,
    },
    ItemType {
        name: "Rare Egg",
        spawn_chance: 0.35,
        stock_limit: None,
        emoji: "<:rare_egg:1386442308391538778>",
        role: Some("<@&1387430341987401849>"),
    },
    ItemType {
        name: "Epic Egg",
        spawn_chance: 0.15,
        stock_limit: None,
        emoji: "<:epic_egg:1386442299478900766>",
        role: Some("<@&1387430557385883840>"),
    },
    ItemType {
        name: "Legendary Egg",
        spawn_chance: 0.05,
        stock_limit: None,
        emoji: "<:legendary_egg:1386442303429808258>",
        role: Some("<@&1387432053166968993>"),
    },
    ItemType {
        name: "Mythical Egg",
        spawn_chance: 0.012,
        stock_limit: Some(2),
        emoji: "<:mythical_egg:1386442306210627605>",
        role: Some("<@&1387432235560603688>"),
    },
    ItemType {
        name: "Celestial Egg",
        spawn_chance: 0.006,
        stock_limit: Some(1),
        emoji: "<:celestial_egg:1386442297943527485>",
        role: Some("<@&1387432340808269954>"),
    },
    ItemType {
        name: "Ethereal Egg",
        spawn_chance: 0.003,
        stock_limit: Some(1),
        emoji: "<:ethereal_egg:1394343048858828800>",
        role: Some("<@&1394343206577246278>"),
    },
    ItemType {
        name: "Prismatic Egg",
        spawn_chance: 0.0015,
        stock_limit: Some(1),
        emoji: "<:prismatic_egg:1394343067976204398>",
        role: Some("<@&1394343256938123384>"),
    },
];

const TOOLS: [ItemType; 3] = [
    ItemType {
        name: "Water Bot",
        spawn_chance: 0.6,
        stock_limit: None,
        emoji: "<:water_bot:1390676187679821844>",
        role: Some("<@&1390308127500406825>"),
    },
    ItemType {
        name: "Priest Bot",
        spawn_chance: 0.15,
        stock_limit: None,
        emoji: "<:priest_bot:1390676177504567436>",
        role: Some("<@&1390308201106378885>"),
    },
    ItemType {
        name: "Harvester Bot",
        spawn_chance: 0.04,
        stock_limit: None,
        emoji: "<:harvester_bot:1390676182487404686>",
        role: Some("<@&1390308245960265862>"),
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

            let mut i: u8 = 0;

            // Egg stock
            for (_, egg) in EGGS.iter().enumerate() {
                let mut rng = RobloxRng::new(seed + i as u64);
                i += 1;

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

            // Tool stock
            for (_, tool) in TOOLS.iter().enumerate() {
                let mut rng = RobloxRng::new(seed + i as u64);
                i += 1;

                let roll = rng.next_f64();
                if roll >= tool.spawn_chance {
                    stock.insert(tool.name, 0);
                    continue;
                }

                stock.insert(tool.name, 1);
            }

            // Format stock in embed and ping roles
            let mut egg_lines = String::new();
            let mut tool_lines = String::new();
            let mut role_mentions = String::new();

            let mut process_items = |items: &[ItemType], output: &mut String| {
                for item in items {
                    if let Some(&count) = stock.get(item.name) {
                        if count > 0 {
                            output.push_str(&format!(
                                "{} {} - **{}**\n",
                                item.emoji, item.name, count
                            ));

                            if let Some(role) = item.role {
                                if !role_mentions.contains(role) {
                                    if !role_mentions.is_empty() {
                                        role_mentions.push(' ');
                                    }
                                    role_mentions.push_str(role);
                                }
                            }
                        }
                    }
                }
            };

            process_items(&EGGS, &mut egg_lines);
            process_items(&TOOLS, &mut tool_lines);

            if tool_lines.is_empty() {
                tool_lines.push_str("Nothing");
            }

            // Create and send embed
            let embed = CreateEmbed::new()
                .title("Stock")
                .color(Color::RED)
                .field("Eggs", egg_lines, false)
                .field("Tools", tool_lines, false);

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
