use futures::future;
use rand::prelude::SliceRandom;
use serenity::model::channel::{ChannelType, GuildChannel};
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

#[tokio::main]
async fn main() {
    init_tracing();

    // Configure the client with your Discord bot token in the environment.
    let bot_token =
        env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN token in the environment");

    let mut client = Client::builder(&bot_token)
        // .application_id(882040915882033272) // CCSS
        .application_id(451862707746897961) // Testing
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

struct Handler {
    shuffle_mutex: Mutex<()>,
}

#[derive(EnumString)]
pub enum Commands {
    // #[strum(serialize = "start")]
    // Start,
    #[strum(serialize = "shuffle")]
    Shuffle,
}

impl Handler {
    async fn interaction_create(
        &self,
        ctx: Context,
        interaction: Interaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Interaction created");
        if let Interaction::ApplicationCommand(command) = interaction.clone() {
            // Get the slash command, or return if it's not a slash command.
            let slash_command = if let Some(slash_command) = interaction.application_command() {
                slash_command
            } else {
                return Ok(());
            };

            match Commands::from_str(&slash_command.data.name[..]).unwrap() {
                Commands::Shuffle => {
                    info!("Shuffle command");

                    let authorized_role_ids = vec![
                        672308385517142017, // CCSS BoD
                        858020772966170635, // WiCS exec
                        370243283244417024, // LameJam organizer
                        927950534986571847, // COMP 1501
                        360856758098329610, // Testing
                    ];

                    // Check that only a member with a authorized role can use this command
                    if !command
                        .clone()
                        .member
                        .unwrap()
                        .roles
                        .iter()
                        .any(|role_id| authorized_role_ids.contains(&role_id.0))
                    {
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(
                                            "You do not have an authorized role to use this command!"
                                        )
                                    })
                            })
                            .await?;
                        return Ok(());
                    }

                    // Get the values for category ID and room size
                    // TODO: Can probably clean this up a bit
                    let shuffle_category_id: Option<u64> = match command
                        .clone()
                        .data
                        .options
                        .iter()
                        .filter(|option| option.name == "category_id")
                        .next()
                    {
                        Some(option) => match &option.resolved {
                            Some(ApplicationCommandInteractionDataOptionValue::String(
                                category_id,
                            )) => Some(
                                category_id
                                    .parse::<u64>()
                                    .expect("Could not parse category ID"),
                            ),
                            _ => None,
                        },
                        None => None,
                    };

                    let room_size: Option<u64> = match command
                        .clone()
                        .data
                        .options
                        .iter()
                        .filter(|option| option.name == "room_size")
                        .next()
                    {
                        Some(option) => match option.resolved {
                            Some(ApplicationCommandInteractionDataOptionValue::Integer(
                                room_size,
                            )) => Some(room_size.try_into().expect("Could not convert room size")),
                            _ => None,
                        },
                        None => None,
                    };

                    dbg!(shuffle_category_id);

                    // Check that there isn't already a shuffle going on
                    let lock = self.shuffle_mutex.try_lock();

                    if lock.is_err() {
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content("A shuffle is already in progress")
                                    })
                            })
                            .await?;
                        return Ok(());
                    }

                    // Get the guild (Discord server)
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
                            if guild_channel.kind != ChannelType::Category {
                                return false;
                            }
                            match shuffle_category_id {
                                Some(category_id) => *guild_channel.id.as_u64() == category_id,
                                None => guild_channel.name == "Speed Friend",
                            }
                        })
                        .collect::<Vec<&GuildChannel>>();

                    // If there is no speed frinding category, return an error
                    if speed_friend_category.is_empty() {
                        command
                            .create_interaction_response(&ctx.http, |response| {
                                response
                                    .kind(InteractionResponseType::ChannelMessageWithSource)
                                    .interaction_response_data(|message| {
                                        message.content(
                                            "There is no category called 'speed friending'",
                                        )
                                    })
                            })
                            .await?;
                        return Ok(());
                    }

                    // Get all the channels in the category
                    let mut speed_friend_channels = guild
                        .channels
                        .iter()
                        .map(|(_, guild_channel)| guild_channel)
                        .filter(|guild_channel| match guild_channel.category_id {
                            Some(category_id) => category_id == speed_friend_category[0].id,
                            None => false,
                        } && guild_channel.kind == ChannelType::Voice)
                        .collect::<Vec<&GuildChannel>>();

                    // Find everyone in the voice channel
                    let mut speakers = future::try_join_all(
                        speed_friend_channels
                            .iter()
                            .map(|channel| channel.members(&ctx.cache)),
                    )
                    .await
                    .unwrap()
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Member>>();

                    // Remove the lobby channel from the list of channels
                    speed_friend_channels.retain(|channel| channel.name.to_lowercase() != "lobby");

                    // Reply with a message
                    command
                        .create_interaction_response(&ctx.http, |response| {
                            response
                                .kind(InteractionResponseType::ChannelMessageWithSource)
                                .interaction_response_data(|message| {
                                    message.content("Shuffling rooms!")
                                })
                        })
                        .await?;

                    // Shuffle the speakers
                    speakers.shuffle(&mut rand::thread_rng());
                    speed_friend_channels.shuffle(&mut rand::thread_rng());

                    for (i, speaker) in speakers.iter().enumerate() {
                        let channel = speed_friend_channels[i % speed_friend_channels.len()];

                        // TODO: Check that they're not already in the channel

                        let error = speaker.move_to_voice_channel(&ctx.http, channel).await;

                        sleep(Duration::from_millis(700)).await;
                        match error {
                            Ok(_) => info!("Moved {} to {}", speaker.user.name, channel.name),
                            Err(e) => {
                                info!(
                                    "Error moving {} to {}: {:?}",
                                    speaker.user.name, channel.name, e
                                )
                            }
                        }
                    }

                    info!("Done shuffle");
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

        for guild in ctx.cache.guilds().await.iter() {
            // Todo: Move this to every guild or something
            if let Err(e) = GuildId(guild.0)
                .set_application_commands(&ctx.http, |commands| {
                    commands.create_application_command(|command| {
                        command.name("shuffle").description(
                            "Shuffle everyone in voice channels in the speed friending category",
                        ).create_option(|option| {
                            option
                                .name("category_id")
                                .description("The ID of the category to shuffle")
                                .kind(ApplicationCommandOptionType::String)
                                .required(false)
                        }).create_option(|option| {
                            option
                                .name("room_size")
                                .description("The number of people in each room")
                                .kind(ApplicationCommandOptionType::Integer)
                                .required(false)
                        })
                    })
                })
                .await
            {
                println!("Error setting application commands: {}", e);
            }
        }
    }
}
