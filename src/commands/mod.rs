use crate::{Context, Error};
use poise::serenity_prelude::ChannelId;
use tracing::info;

#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn cross(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Source channel"] source: ChannelId,
    #[description = "Target channel"] target: ChannelId,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    db.put_crossover(
        ctx.guild_id().ok_or("Unable to get GuildId!")?,
        source,
        target,
    )?;
    ctx.say("Added crossover!").await?;
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Source channel"] source: ChannelId,
    #[description = "Target channel"] target: ChannelId,
) -> Result<(), Error> {
    let db = ctx.data().db.lock().await;
    let result = db.remove_crossover(
        ctx.guild_id().ok_or("Unable to get GuildId!")?,
        source,
        target,
    )?;
    if result {
        ctx.say("Removed crossover!").await?;
    }
    Ok(())
}

#[poise::command(slash_command, required_permissions = "MANAGE_CHANNELS")]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let gid = ctx.guild_id().ok_or("Unable to get GuildId!")?;
    let db = ctx.data().db.lock().await;
    let results = db.get_all(gid)?;
    info!("{:?}", results);
    let mut str = String::new();
    for pair in results {
        str.push_str(&format!(
            "Source: <#{}> -> Target : <#{}>\n",
            pair.0, pair.1
        ));
    }
    ctx.say(str).await?;
    Ok(())
}
