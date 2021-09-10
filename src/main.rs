use serde::Deserialize;
use std::fs;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, context: Context, msg: Message) {
        if msg.content == "!me" {
            // If the `utils`-feature is enabled, then model structs will
            // have a lot of useful methods implemented, to avoid using an
            // often otherwise bulky Context, or even much lower-level `rest`
            // method.
            //
            // In this case, you can direct message a User directly by simply
            // calling a method on its instance, with the content of the
            // message.
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