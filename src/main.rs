use dotenv::dotenv;

use handler::Handler;

use serenity::prelude::*;

use std::env;

use tracing::{debug, Level};
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

    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::GUILD_INTEGRATIONS
        | GatewayIntents::GUILD_VOICE_STATES;

    let mut client = Client::builder(&bot_token, intents)
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
