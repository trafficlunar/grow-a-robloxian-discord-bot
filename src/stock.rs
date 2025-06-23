use std::{
    collections::HashMap,
    env,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use serenity::all::{ChannelId, Color, Context, CreateEmbed, CreateMessage};
use tokio::time::{sleep_until, Instant};

use crate::rng::RobloxRng;

const UPDATE_INTERVAL: u64 = 300;
const EGG_TYPES: [(&str, f64); 7] = [
    ("Common Egg", 1.0),
    ("Uncommon Egg", 0.75),
    ("Rare Egg", 0.35),
    ("Epic Egg", 0.15),
    ("Legendary Egg", 0.05),
    ("Mythical Egg", 0.0015),
    ("Celestial Egg", 0.0001),
];

const EMOJI_MAP: [(&str, &str); 7] = [
    ("Common Egg", "<:common_egg:1386442268700835960>"),
    ("Uncommon Egg", "<:uncommon_egg:1386442310480302100>"),
    ("Rare Egg", "<:rare_egg:1386442308391538778>"),
    ("Epic Egg", "<:epic_egg:1386442299478900766>"),
    ("Legendary Egg", "<:legendary_egg:1386442303429808258>"),
    ("Mythical Egg", "<:mythical_egg:1386442306210627605>"),
    ("Celestial Egg", "<:celestial_egg:1386442297943527485>"),
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

            sleep_until(Instant::now() + wait_duration).await;

            // Generate stock
            let seed: u64 = secs - (secs % UPDATE_INTERVAL);
            let mut stock = HashMap::new();

            for (i, (name, chance)) in EGG_TYPES.iter().enumerate() {
                let mut egg_rng = RobloxRng::new(seed + i as u64);

                let roll = egg_rng.next_f64();
                let quantity = egg_rng.next_range(1, 5);

                let in_stock = if roll < *chance { quantity } else { 0 };

                stock.insert(*name, in_stock);
            }

            let mut stock_lines = String::new();

            // Sort EGG_TYPES based on their order
            for (name, _) in EGG_TYPES.iter() {
                if let Some(count) = stock.get(name) {
                    if *count == 0 {
                        continue;
                    }

                    let emoji = EMOJI_MAP
                        .iter()
                        .find(|(key, _)| key == name)
                        .map(|(_, emoji)| *emoji)
                        .unwrap_or(name);

                    stock_lines += &format!("{} {} - **{}**\n", emoji, name, count);
                }
            }

            let embed = CreateEmbed::new()
                .title("Egg Stock")
                .color(Color::RED)
                .description(stock_lines);
            let builder = CreateMessage::new().embed(embed);

            if let Err(err) = channel.send_message(&ctx.http, builder).await {
                eprintln!("Failed to send message: {:?}", err);
            }

            println!("Generated and sent stock message");
        }
    });
}
