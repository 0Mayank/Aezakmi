use serde::Deserialize;
use std::{
    fs,
    collections::{HashMap, HashSet},
    fmt::Write,
    sync::Arc,
};

use serenity::{
    async_trait,
    prelude::*,
    utils::MessageBuilder,
    framework::standard::{
        buckets::{LimitedFor, RevertBucket},
        help_commands,
        macros::{check, command, group, help, hook},
        Args,
        CommandGroup,
        CommandOptions,
        CommandResult,
        DispatchError,
        HelpOptions,
        Reason,
        StandardFramework,
    },
    model::{
        channel::{Channel, Message},
        gateway::Ready,
        id::UserId,
        permissions::Permissions,
    },
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

#[group]
#[commands(ping)]
pub struct Meta;

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

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(e) = msg.reply(ctx, "Pong!").await {
        println!("error in msg: {:?}", e);
    }

    Ok(())
}


#[derive(Deserialize)]
struct Config {
    token: String,
    prefix: String
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    let config: Config = toml::from_str(
        &fs::read_to_string("./config.toml")
        .expect("Cannot find config.toml"))
        .expect("Cannot parse config.toml");
    
    let token = &config.token;
    
    let framework = StandardFramework::new()
        .configure(|c| c
            .prefix(&config.prefix)
        )
        .help(&MY_HELP)
        .group(&META_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}