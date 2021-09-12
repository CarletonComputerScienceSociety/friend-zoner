use std::env;

use serenity::model::channel::{ChannelType, GuildChannel};
use serenity::model::Permissions;
use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommand, ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction, InteractionResponseType,
        },
    },
    prelude::*,
};
use std::str::FromStr;
use strum_macros::EnumString;
use tracing::{debug, error, warn, Level};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    init_tracing();

    // Configure the client with your Discord bot token in the environment.
    let bot_token = "ODgyMDQwOTE1ODgyMDMzMjcy.YS1mnQ.MDIEMq0PSNKOifIZ6Sz4sBQ69vs";

    let mut client = Client::builder(&bot_token)
        .application_id(882040915882033272)
        .event_handler(Handler)
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

struct Handler;

#[derive(EnumString)]
pub enum Commands {
    #[strum(serialize = "start")]
    Start,
    #[strum(serialize = "shuffle")]
    Shuffle,
}

impl Handler {
    async fn interaction_create(
        &self,
        ctx: Context,
        interaction: Interaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        error!("Interaction created");
        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            // SET ROOM MAX
            // decides how many users can be in a room

            // STATUS
            // gives info about room max and stuff
            // category info

            // Get the slash command, or return if it's not a slash command.
            let slash_command = if let Some(slash_command) = interaction.application_command() {
                slash_command
            } else {
                return Ok(());
            };

            match Commands::from_str(&slash_command.data.name[..]).unwrap() {
                Commands::Start => {
                    warn!("Start command");

                    let categories = ctx.cache.categories().await;
                    let guild_id = command.guild_id.unwrap();
                    let guild = guild_id
                        .to_guild_cached(&ctx)
                        .await
                        .ok_or("Cannot get guild")?;

                    // Try to get the category to be used for speed friending
                    let speed_friend_category = guild
                        .channels
                        .iter()
                        .map(|(_, guild_channel)| guild_channel)
                        .filter(|guild_channel| {
                            guild_channel.kind == ChannelType::Category
                                && guild_channel.name.as_str().to_lowercase() == "speed friending"
                        })
                        .collect::<Vec<&GuildChannel>>();

                    // There should only be one category named "Speed Friending"

                    if speed_friend_category.len() == 0 {
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message
                                            .content(format!("No speed friending category found"))
                                    })
                            })
                            .await?;
                        return Ok(());
                    } else if speed_friend_category.len() > 1 {
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(format!(
                                            "Multiple speed friending categories found"
                                        ))
                                    })
                            })
                            .await?;
                        return Ok(());
                    }

                    warn!("TEST");

                    // Make sure there are no channels in the category
                    let existing_channels = guild
                        .channels
                        .iter()
                        .map(|(_, guild_channel)| guild_channel)
                        .filter(|guild_channel| match guild_channel.category_id {
                            Some(category_id) => category_id == speed_friend_category[0].id,
                            None => false,
                        })
                        .collect::<Vec<&GuildChannel>>();

                    if existing_channels.len() > 0 {
                        // command.create_interaction_response(&ctx.http, |response| {
                        //     response
                        //         .kind(InteractionResponseType::ChannelMessageWithSource)
                        //         .interaction_response_data(|message| {
                        //             message.content(format!("Channels already exist in speed friending category, please remove them first"))
                        //         })
                        // }).await?;
                        // return Ok(());
                        
                        for channel in existing_channels {
                            warn!("Deleting channel {}", channel.name);
                            channel.delete(&ctx.http).await?;
                        }
                        
                    }

                    // Create the channels
                    for i in 0..2 {
                        let channel_name = format!("{} {}", "Speed Friending", i + 1);
                        let channel_category = speed_friend_category[0].id;
                        let channel_type = ChannelType::Voice;
                        let channel_permissions = Permissions::empty();

                        let channel = guild
                            .create_channel(&ctx.http, |c| {
                                c.name(channel_name)
                                    .kind(channel_type)
                                    .category(channel_category)
                            })
                            .await?;
                    }

                    // create channels depending on number of users

                    // debug!(
                    //     "{:?}",
                    //     channels
                    //         .iter()
                    //         .filter(|(_, x)| x.kind == ChannelType::Category)
                    //         .map(|(_, x)| x.name.as_str())
                    //         .collect::<Vec<&str>>()
                    // );

                    // warn!("{:#?}", categories);
                    // START ROOMS COMMANDS
                    // look for category with certain name to put channels in
                    // If it doesn't exist, tell them to create it

                    command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content(format!("Channels created!"))
                                })
                        })
                        .await?;
                }
                Commands::Shuffle => {
                    warn!("Shuffle command");
                    // SHUFFLE ROOMS
                    // see if it's the shuffle command
                    // get all users in rooms that are owned by the bot
                    // shuffle them into rooms based on how many users there are
                    // add or remove rooms depending on size max
                }
            }
        }
        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, context: Context, interaction: Interaction) {
        if let Err(e) = self.interaction_create(context, interaction).await {
            error!(?e, "Error while processing message");
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        if let Err(e) = GuildId(882042113871732766)
            .set_application_commands(&ctx.http, |commands| {
                commands
                    .create_application_command(|command| {
                        command.name("shuffle").description("something")
                    })
                    .create_application_command(|command| {
                        command.name("start").description("something")
                    })
            })
            .await
        {
            println!("Error setting application commands: {}", e);
        }
    }
}
