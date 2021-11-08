use core::default::Default;
use mongodb::{
    bson::{doc, Document},
    options::{FindOneAndUpdateOptions, FindOneOptions},
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

use crate::{
    check_channel, check_command, check_role_or_user, get_guilds, update_guild, view_guild,
};

#[group]
#[commands(ping, say, botinvite, enable, disable, prefix)]
pub struct Meta;

#[derive(Debug)]
struct FilteredArgs<'a> {
    commands: Option<Vec<&'a str>>,
    channels: Option<Vec<&'a str>>,
    role_user: Option<Vec<&'a str>>,
}

#[macro_export]
macro_rules! filter_to_FilteredArgs {
    ($args:expr, $f:expr) => {
        let args = $args.rest().split_whitespace();

        let mut in_index = None;
        let mut for_index = None;

        for (index, arg) in args.clone().enumerate() {
            if arg == "in" && in_index.is_none() {
                in_index = Some(index);
            } else if arg == "for" && for_index.is_none() {
                for_index = Some(index);
            }

            if !in_index.is_none() && !for_index.is_none() {
                break;
            }
        }

        let args: Vec<_> = args.collect();

        let filtered_args = if in_index.is_none() && for_index.is_none() {
            FilteredArgs {
                commands: Some(Vec::from(&args[..])),
                channels: None,
                role_user: None,
            }
        } else if in_index.is_none() {
            let for_index = for_index.unwrap();
            FilteredArgs {
                commands: Some(Vec::from(&args[0..for_index])),
                channels: None,
                role_user: Some(Vec::from(&args[(for_index + 1)..])),
            }
        } else if for_index.is_none() {
            let in_index = in_index.unwrap();
            FilteredArgs {
                commands: Some(Vec::from(&args[0..in_index])),
                channels: Some(Vec::from(&args[(in_index + 1)..])),
                role_user: None,
            }
        } else if in_index > for_index {
            let in_index = in_index.unwrap();
            let for_index = for_index.unwrap();
            FilteredArgs {
                commands: Some(Vec::from(&args[0..for_index])),
                role_user: Some(Vec::from(&args[(for_index + 1)..in_index])),
                channels: Some(Vec::from(&args[(in_index + 1)..])),
            }
        } else {
            let in_index = in_index.unwrap();
            let for_index = for_index.unwrap();
            FilteredArgs {
                commands: Some(Vec::from(&args[0..in_index])),
                channels: Some(Vec::from(&args[(in_index + 1)..for_index])),
                role_user: Some(Vec::from(&args[(for_index + 1)..])),
            }
        };

        $f.commands = filtered_args.commands;
        $f.channels = filtered_args.channels;
        $f.role_user = filtered_args.role_user;

        $f.commands = Some(
            $f.commands
                .unwrap_or(vec!["all"]) // not sure if i should keep these or not
                .into_iter()
                .filter(|&e| check_command(e))
                .collect(),
        );

        $f.channels = Some(
            $f.channels
                .unwrap_or(vec!["all"])
                .into_iter()
                .filter(|&e| check_channel(e))
                .collect(),
        );

        $f.role_user = Some(
            $f.role_user
                .unwrap_or(vec!["all"])
                .into_iter()
                .filter(|&e| check_role_or_user(e))
                .collect(),
        );
    };
}

#[command]
#[description = "Check if I am working"]
#[usage = ""]
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
#[description = "Get a link to invite me to your server"]
#[usage = ""]
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
#[description = "Enable a command server-wide, or for a role, channel or user"]
#[usage = "<command name> [Channel|Role|User] ..."]
#[example = "ping"]
#[example = "ping #general"]
#[only_in(guild)]
async fn enable(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    // ae enable c1, c2, c3 .. in #ch1, #ch2, .. for r1, r2, u1, u2
    // ae enable c1, c2, c3 .. in #ch1, #ch2, ..
    let collection = get_guilds!(ctx);

    if args.is_empty() {
        return Ok(());
    }

    let mut f = FilteredArgs {
        commands: None,
        channels: None,
        role_user: None,
    };

    filter_to_FilteredArgs!(args, f);

    let mut update_doc = Document::new();

    let channels = f.channels.unwrap();

    for cmd in f.commands.unwrap() {
        update_doc.insert(
            "$set",
            doc!(format!("disabled_commands.{}.server", cmd): false), //? not working
        );
        for chan in &channels {
            update_doc.insert(
                "$set",
                doc!(format!("disabled_commands.{}.{}", cmd, chan): f.role_user.clone().unwrap()),
            );
        }
    }

    let result = update_guild!(&ctx, &msg, update_doc, collection);

    println!("{:?}", result);

    Ok(())
}

#[command]
#[description = "disable a command server-wide, or for a role, channel or user"]
#[usage = "d<command name> [Channel|Role|User] ..."]
#[example = "ping"]
#[example = "ping #general"]
#[only_in(guild)]
async fn disable(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let collection = get_guilds!(ctx);

    let guild_id = i64::from(msg.guild_id.unwrap());

    Ok(())
}

#[command]
#[max_args(1)]
#[description = "Changes or shows the prefix"]
#[usage = "<\"your prefix\">"]
#[example = "\"hello ae\""]
#[only_in(guild)]
async fn prefix(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let prefix = match args.remains() {
        Some(prefix) => prefix,
        None => {
            msg.reply(
                ctx,
                format!("current prefix is {}", view_guild!(ctx, msg, "prefix")),
            )
            .await?;
            return Ok(());
        }
    };

    let update = doc! {"$set": {"prefix": &prefix}};
    update_guild!(ctx, msg, update);

    msg.reply(ctx, format!("Prefix changed to \"{}\"", prefix))
        .await?;

    Ok(())
}
