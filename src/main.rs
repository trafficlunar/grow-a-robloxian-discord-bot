use std::env;

use poise::{
    serenity_prelude::{ActivityData, Client, GatewayIntents},
    Framework, FrameworkOptions, PrefixFrameworkOptions,
};

use crate::commands::rules::rules;

mod commands;
mod rng;
mod stock;

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,);
            let _ = ctx
                .say("❌ An error occurred while executing this command.")
                .await;
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let token = env::var("DISCORD_TOKEN").expect("No discord bot token found");
    let intents = GatewayIntents::GUILD_MESSAGES | GatewayIntents::MESSAGE_CONTENT;

    let framework = Framework::builder()
        .setup(move |ctx, _ready, framework| {
            Box::pin(async move {
                println!("Bot is online!");
                ctx.set_activity(Some(ActivityData::watching("stock")));

                stock::start_stock_loop(ctx.clone());

                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {})
            })
        })
        .options(FrameworkOptions {
            prefix_options: PrefixFrameworkOptions {
                prefix: Some("~".into()),
                ..Default::default()
            },
            commands: vec![rules()],
            on_error: |error| Box::pin(on_error(error)),
            ..Default::default()
        })
        .build();

    let mut client = Client::builder(&token, intents)
        .framework(framework)
        .await
        .expect("Error creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {why:?}");
    }
}
