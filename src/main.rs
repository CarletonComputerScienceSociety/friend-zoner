use std::env;

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

#[tokio::main]
async fn main() {
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

struct Handler;

#[derive(strum_macros::Display)]
pub enum Commands {
    Start,
    Shuffle,
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(
        &self,
        ctx: Context,
        interaction: Interaction,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Interaction::ApplicationCommand(command) = interaction {
            // START ROOMS COMMANDS
            // look for category with certain name to put channels in
            // If it doesn't exist, tell them to create it
            // if it exsists, create channels depending on number of users

            // SET ROOM MAX
            // decides how many users can be in a room

            // SHUFFLE ROOMS
            // see if it's the shuffle command
            // get all users in rooms that are owned by the bot
            // shuffle them into rooms based on how many users there are
            // add or remove rooms depending on size max

            // STATUS
            // gives info about room max and stuff
            // category info

            // Get the slash command, or return if it's not a slash command.
            let slash_command = if let Some(slash_command) = interaction.application_command() {
                slash_command
            } else {
                return Ok(());
            };

            match &slash_command.data.name[..] {
                Commands::Start.into() => {},
                Commands::Shuffle.into() => {}
            }
        }
        Ok(())
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
