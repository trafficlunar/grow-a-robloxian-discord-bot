use std::env;

use serenity::{
    all::{Context, EventHandler, GatewayIntents, Ready},
    async_trait, Client,
};

mod rng;
mod stock;
use stock::start_stock_loop;

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, _: Ready) {
        println!("Bot is online!");
        start_stock_loop(ctx);
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("No discord bot token found");
    let intents = GatewayIntents::GUILD_MESSAGES;

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
