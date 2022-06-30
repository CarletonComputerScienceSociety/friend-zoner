use dotenv::dotenv;
use futures::future;
use handler::Handler;
use rand::prelude::SliceRandom;
use serenity::model::channel::{Channel, ChannelCategory, ChannelType, GuildChannel};
use serenity::model::guild::Member;
use serenity::model::interactions::application_command::{
    ApplicationCommandInteractionDataOptionValue, ApplicationCommandOptionType,
};
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{Interaction, InteractionResponseType},
    },
    prelude::*,
};
use std::convert::TryInto;
use std::env;
use std::str::FromStr;
use strum_macros::EnumString;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, Level};
use tracing_subscriber::EnvFilter;

mod handler;

#[tokio::main]
async fn main() {
    init_tracing();

    // Load an dotenv file if it exists.
    dotenv().ok();

    // Configure the client with your Discord bot token in the environment.
    let bot_token =
        env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN token in the environment");

    let application_id: u64 = env::var("APPLICATION_ID")
        .expect("Expected APPLICATION_ID token in the environment")
        .parse()
        .expect("Failed to parse APPLICATION_ID");

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_INTEGRATIONS;

    let mut client = Client::builder(&bot_token, intents)
        .application_id(application_id)
        .event_handler(Handler {
            shuffle_mutex: Mutex::new(()),
        })
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

fn init_tracing() {
    let append_info = |mut f: EnvFilter, list: &[&str], level: &str| {
        for l in list {
            f = f.add_directive(format!("{}={}", l, level).parse().unwrap());
        }
        f
    };

    let list = &[
        "tokio_util",
        "h2",
        "rustls",
        "serenity",
        "tungstenite",
        "async_tungstenite",
        "hyper",
        "trust_dns_resolver",
        "trust_dns_proto",
        "reqwest",
        "mio",
        "want",
        "kube",
        "tower",
    ];

    let filter = EnvFilter::from_default_env();
    let filter = append_info(filter.add_directive(Level::TRACE.into()), list, "info");

    tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_env_filter(filter)
        .try_init()
        .unwrap();

    debug!("tracing initialized");
}
