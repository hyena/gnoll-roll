extern crate itertools;

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate serenity;

use serenity::client::Client;
use serenity::model::channel::Message;
use serenity::prelude::{EventHandler, Context};
use serenity::framework::standard::{
    Args,
    StandardFramework,
    CommandResult,
    macros::{
        command,
        group
    }
};

group!({
    name: "general",
    options: {},
    commands: [gr]
});

use std::env;
pub mod roll_parse;

struct Handler;

impl EventHandler for Handler {}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("/")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

#[command]
fn gr(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    match roll_parse::parse_roll(args.rest()) {
        Ok((result_str, _)) => {
            msg.reply(&ctx, &format!("`{}` {}", args.message(), &result_str));
        }
        Err(_) => {
            msg.reply(&ctx, "Bad format.");
        }
    };

    Ok(())
}