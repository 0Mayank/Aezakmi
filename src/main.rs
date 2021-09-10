use serde::Deserialize;
use std::fs;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    utils::MessageBuilder
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == "!me" {
            let dm = msg
                .author
                .dm(&context, |m| {
                    m.content("Hello!");

                    m
                })
                .await;

            if let Err(why) = dm {
                println!("Error when direct messaging user: {:?}", why);
            }
        }

        if msg.content == "!ping" {
            let channel = match msg.channel_id.to_channel(&context).await {
                Ok(channel) => channel,
                Err(why) => {
                    println!("Error getting channel: {:?}", why);

                    return;
                },
            };

            // The message builder allows for creating a message by
            // mentioning users dynamically, pushing "safe" versions of
            // content (such as bolding normalized content), displaying
            // emojis, and more.
            let response = MessageBuilder::new()
                .push("User ")
                .push_bold_safe(&msg.author.name)
                .push(" used the 'ping' command in the ")
                .mention(&channel)
                .push(" channel")
                .build();

            if let Err(why) = msg.channel_id.say(&context.http, &response).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[derive(Deserialize)]
struct Config {
    token: String
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let config: Config = toml::from_str(
        &fs::read_to_string("./config.toml")
        .expect("Cannot find config.toml"))
        .expect("Cannot parse config.toml");
    
    let token = config.token;
    let mut client = Client::builder(&token).event_handler(Handler).await.expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}