use serenity::{
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::{oauth2::OAuth2Scope, prelude::*, Permissions},
    prelude::*,
    utils::{content_safe, Color, ContentSafeOptions},
};

#[group]
#[commands(ping, say, botinvite)]
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
