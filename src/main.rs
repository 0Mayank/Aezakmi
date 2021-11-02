#![feature(async_closure)]

mod cogs;
use cogs::{meta::*, music::*};

use mongodb::{
    bson::{doc, Document},
    options::{ClientOptions, FindOneOptions, InsertOneOptions},
    Collection,
};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs,
    future::{self, Future},
};

use serenity::{
    async_trait,
    framework::standard::{
        help_commands,
        macros::{help, hook},
        Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
    },
    model::{channel::Message, gateway::Ready, id::UserId},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

struct Db;

impl TypeMapKey for Db {
    type Value = mongodb::Database;
}

#[derive(Deserialize)]
struct Config {
    token: String,
    prefix: String,
    user_id: u64,
}

impl TypeMapKey for Config {
    type Value = Self;
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

#[hook]
async fn dynamic_prefix(ctx: &Context, msg: &Message) -> Option<String> {
    let data = ctx.data.read().await;
    let db = data.get::<Db>().unwrap();
    let collection = db.collection::<Document>("guilds");
    let config = data.get::<Config>().unwrap();

    let mut guild_id = 0;
    if let Some(id) = msg.guild_id {
        guild_id = i64::from(id)
    } else if let None = msg.guild_id {
        return Some("ae".to_string());
    };

    let prefix = async_unwrap_or_else(
        collection
            .find_one(doc! {"_id": guild_id}, FindOneOptions::builder().build())
            .await
            .unwrap(),
        async || {
            initialize_guild(&collection, guild_id, &config.prefix)
                .await
                .unwrap()
        },
    )
    .await
    .get("prefix")
    .unwrap()
    .to_string();

    Some(prefix[1..(prefix.len() - 1)].to_string())
}

#[tokio::main]
async fn main() {
    let db = connect_to_db()
        .await
        .expect("Cannot connect to the database");

    // Configure the client with Discord bot token
    let config: Config =
        toml::from_str(&fs::read_to_string("./config.toml").expect("Cannot find config.toml"))
            .expect("Cannot parse config.toml");

    let token = &config.token;

    let framework = StandardFramework::new()
        .configure(|c| {
            c.dynamic_prefix(dynamic_prefix)
                .with_whitespace(true)
                .no_dm_prefix(true)
                .on_mention(Some(UserId(config.user_id)))
        })
        .help(&MY_HELP)
        .group(&META_GROUP)
        .group(&MUSIC_GROUP);

    let mut client = Client::builder(token)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<Db>(db)
        .type_map_insert::<Config>(config)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn connect_to_db() -> mongodb::error::Result<mongodb::Database> {
    // Parse a connection string into an options struct.
    let mut client_options = ClientOptions::parse("mongodb://localhost:27017").await?;

    // Manually set an option.
    client_options.app_name = Some("seoy".to_string());

    // Get a handle to the deployment.
    let client = mongodb::Client::with_options(client_options)?;

    // Ping the server to see if you can connect to the cluster
    client
        .database("aezakmi")
        .run_command(doc! {"ping": 1}, None)
        .await?;
    println!("Connected to database successfully.");

    // Get a handle to a database.
    Ok(client.database("aezakmi"))
}

async fn initialize_guild(
    collection: &Collection<Document>,
    guild_id: i64,
    prefix: &str,
) -> mongodb::error::Result<Document> {
    let options = InsertOneOptions::builder().build();
    println!("initializing");
    let the_doc = doc! {
        "_id": guild_id,
        "prefix": prefix,
        "commands": {
            "ping": {
                "server": false,
                "roles": [],
                "channels": [],
                "users": []
            }
        }
    };
    collection.insert_one(&the_doc, options).await?;

    Ok(the_doc)
}

async fn async_unwrap_or_else<T, Fut>(arg: Option<T>, def: impl FnOnce() -> Fut) -> T
where
    Fut: Future<Output = T>,
{
    match arg {
        Some(val) => val,
        None => def().await,
    }
}
