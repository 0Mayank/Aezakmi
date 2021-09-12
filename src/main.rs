mod cogs;
use cogs::{
    meta::*,
    music::*
};


use serde::Deserialize;
use std::{
    fs,
    collections::{HashSet}
};

use serenity::{
    async_trait,
    prelude::*,
    framework::standard::{
        help_commands,
        macros::{help},
        Args,
        CommandGroup,
        CommandResult,
        HelpOptions,
        StandardFramework,
    },
    model::{
        channel::{Message},
        gateway::Ready,
        id::UserId,
    },
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}



#[help]
#[individual_command_tip = "If you want more information about a specific command, just pass the command as argument."]
#[command_not_found_text = "Could not find: `{}`."]
#[max_levenshtein_distance(3)]
#[indention_prefix = " "]
#[lacking_permissions = "Hide"]
#[lacking_role = "Nothing"]
#[wrong_channel = "Strike"]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    let _ = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[derive(Deserialize)]
struct Config {
    token: String,
    prefix: String
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token
    let config: Config = toml::from_str(
        &fs::read_to_string("./config.toml")
        .expect("Cannot find config.toml"))
        .expect("Cannot parse config.toml");
    
    let token = &config.token;
    
    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix(&config.prefix)
            .with_whitespace(true)
        )
        .help(&MY_HELP)
        .group(&META_GROUP)
        .group(&MUSIC_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}