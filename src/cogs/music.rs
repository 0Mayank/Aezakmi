use serenity::framework::standard::{
    macros::{command, group},
    CommandResult,
    Args
};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
async fn play(_ctx: &Context, _msg: &Message, _args: Args) -> CommandResult {
    todo!();
}

#[group]
#[commands(play)]
pub struct Music;