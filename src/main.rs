extern crate itertools;

extern crate pest;
#[macro_use]
extern crate pest_derive;
#[macro_use]
extern crate serenity;

use std::env;

use serenity::client::Client;
use serenity::prelude::EventHandler;
use serenity::framework::standard::StandardFramework;

pub mod roll_parse;

struct Handler;

impl EventHandler for Handler {}

fn main() {
    // Login with a bot token from the environment
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("/")) // set the bot's prefix to "~"
        .cmd("gr", roll));

    // start listening for events by starting a single shard
    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

command!(roll(_context, msg, args) {
    match roll_parse::parse_roll(args.full()) {
        Ok((result_str, _)) => {
            msg.reply(&result_str);
        }
        Err(_) => {
            msg.reply("Bad format.");
        }
    };
});