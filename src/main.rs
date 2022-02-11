mod commands;
mod db;

use crate::db::CrossoverDb;
use poise::serenity_prelude as serenity;
use tokio::sync::Mutex;
use tracing::{error, info};

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    db: Mutex<CrossoverDb>,
}

async fn event_listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    framework: &poise::Framework<Data, Error>,
    user_data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::Ready { data_about_bot } => {
            info!("{} is connected!", data_about_bot.user.name)
        }
        poise::Event::GuildCreate { guild, is_new: _ } => {
            let mut commands_builder = serenity::CreateApplicationCommands::default();
            let commands = &framework.options().commands;
            for command in commands {
                if let Some(slash) = command.create_as_slash_command() {
                    commands_builder.add_application_command(slash);
                }
                if let Some(context_menu_command) = command.create_as_context_menu_command() {
                    commands_builder.add_application_command(context_menu_command);
                }
            }
            let commands_builder = serenity::json::Value::Array(commands_builder.0);
            info!("Registering {} commands...", commands.len());
            ctx.http
                .create_guild_application_commands(guild.id.0, &commands_builder)
                .await?;
        }
        poise::Event::Message { new_message } => {
            if !&new_message.is_own(ctx) && !&new_message.author.bot {
                if let Some(guild_id) = new_message.guild_id {
                    let db = user_data.db.lock().await;
                    if let Some(target) = db
                        .get_crossover(guild_id, new_message.channel_id)
                        .unwrap_or(None)
                    {
                        target
                            .send_message(ctx, |m| {
                                m.embed(|e| {
                                    e.title(&format!("New post from {}", &new_message.author.name));
                                    e.description(&new_message.content);
                                    e.author(|a| {
                                        a.name(&new_message.author.name);
                                        if let Some(avatar) = &new_message.author.avatar_url() {
                                            a.icon_url(avatar);
                                        }
                                        a.url(&new_message.link());
                                        a
                                    });
                                    if let Some(attachment) = &new_message.attachments.first() {
                                        e.image(&attachment.url);
                                    }
                                    e
                                })
                            })
                            .await?;
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx } => {
            error!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                error!("Error handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt::init();
    let token = std::env::var("TOKEN")?;
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| String::from("crosses.db"));
    let db = CrossoverDb::new(&db_path)?;

    let options = poise::FrameworkOptions {
        commands: vec![poise::Command {
            subcommands: vec![commands::add(), commands::remove(), commands::list()],
            ..commands::cross()
        }],
        on_error: |error| Box::pin(on_error(error)),
        listener: |ctx, event, framework, user_data| {
            Box::pin(event_listener(ctx, event, framework, user_data))
        },
        ..Default::default()
    };

    poise::Framework::build()
        .token(token)
        .user_data_setup(move |_ctx, _ready, _framework| {
            Box::pin(async move { Ok(Data { db: Mutex::new(db) }) })
        })
        .options(options)
        .run()
        .await
        .unwrap();

    Ok(())
}
