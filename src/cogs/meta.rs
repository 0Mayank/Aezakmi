use core::default::Default;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndUpdateOptions, FindOneOptions, InsertOneOptions},
};

use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::{oauth2::OAuth2Scope, prelude::*, Permissions},
    prelude::*,
    utils::{content_safe, Color, ContentSafeOptions},
};

use crate::initialize_guild;

#[group]
#[commands(ping, say, botinvite, enable, disable, prefix)]
pub struct Meta;

#[command]
async fn ping(ctx: &Context, msg: &Message) -> CommandResult {
    if let Err(e) = msg.reply(ctx, "Pong!").await {
        println!("error in msg: {:?}", e);
    }

    Ok(())
}

#[command]
async fn say(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let settings = if let Some(guild_id) = msg.guild_id {
        // By default roles, users, and channel mentions are cleaned.
        ContentSafeOptions::default()
            // We do not want to clean channal mentions as they
            // do not ping users.
            .clean_channel(false)
            // If it's a guild channel, we want mentioned users to be displayed
            // as their display name.
            .display_as_member_from(guild_id)
    } else {
        ContentSafeOptions::default()
            .clean_channel(false)
            .clean_role(false)
    };

    let content = content_safe(&ctx.cache, &args.rest(), &settings).await;

    msg.channel_id.say(&ctx.http, &content).await?;

    Ok(())
}

#[command]
#[aliases("invite")]
async fn botinvite(ctx: &Context, msg: &Message) -> CommandResult {
    let scopes = vec![OAuth2Scope::Bot];
    let url = ctx
        .cache
        .current_user()
        .await
        .invite_url_with_oauth2_scopes(&ctx.http, Permissions::ADMINISTRATOR, &scopes)
        .await?;

    msg.channel_id
        .send_message(&ctx.http, |m| {
            m.reference_message(msg).embed(|e| {
                e.title("Bot Invite Link")
                    .url(url)
                    .color(Color::new(64141))
                    .description("Click the title to invite me to your server")
            })
        })
        .await?;

    Ok(())
}

#[command]
#[only_in(guild)]
async fn enable(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<crate::Db>().unwrap();
    let collection = db.collection::<Document>("guilds");

    Ok(())
}

#[command]
#[only_in(guild)]
async fn disable(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<crate::Db>().unwrap();
    let collection = db.collection::<Document>("guilds");

    Ok(())
}

#[command]
#[max_args(1)]
#[only_in(guild)]
async fn prefix(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let data = ctx.data.read().await;
    let db = data.get::<crate::Db>().unwrap();
    let collection = db.collection::<Document>("guilds");

    let guild_id = i64::from(msg.guild_id.unwrap());

    let prefix = match args.single_quoted::<String>() {
        Ok(prefix) => prefix,
        Err(_) => {
            msg.reply(
                ctx,
                format!(
                    "current prefix is {}",
                    collection
                        .find_one(doc! {"_id": guild_id}, FindOneOptions::builder().build())
                        .await
                        .unwrap()
                        .unwrap()
                        .get("prefix")
                        .unwrap()
                ),
            )
            .await?;
            return Ok(());
        }
    };

    let filter = doc! {"_id": guild_id};
    let update = doc! {"$set": {"prefix": &prefix}};
    let options = FindOneAndUpdateOptions::builder().build();

    let result = collection
        .find_one_and_update(filter, update, options)
        .await?;

    if let None = result {
        initialize_guild(&collection, guild_id, &prefix).await?;
    }

    msg.reply(ctx, format!("Prefix changed to \"{}\"", prefix))
        .await?;

    Ok(())
}
