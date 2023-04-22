
use colored::Colorize;
use serenity::client::Context;
use serenity::model::application::interaction::InteractionResponseType::ChannelMessageWithSource;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use crate::lang::lang_loader::get_key;
use crate::constants::{GREEN_COLOR, RED_COLOR};
use chrono::{Timelike, Local};
use serenity::model::channel::{ChannelType, GuildChannel, PermissionOverwrite};
use serenity::model::guild::{PartialGuild, Role};
use serenity::model::Permissions;
use crate::common_functions::LogType::{SUCCESS};

pub enum LogType {
    SUCCESS,
    WARNING,
    ERROR
}

pub fn log(log_type: LogType, message: &str){
    let log_color;
    let log_type_str = match log_type{
        LogType::SUCCESS => {
            log_color = colored::Color::Green;
            "SUCCESS"
        }
        LogType::WARNING => {
            log_color = colored::Color::TrueColor { r: 255, g: 160, b: 0 };
            "WARNING"
        }
        LogType::ERROR => {
            log_color = colored::Color::Red;
            "ERROR"}
    };
    let current = Local::now();
    let date = format!("{:02}:{:02}:{:02}", current.hour(), current.minute(), current.second());
    let message = format!("[{}] {:07} : {}", date, log_type_str.color(log_color), message);
    println!("{}", message);
}

pub async fn send_success_from_aci(ctx : &Context, aci: &ApplicationCommandInteraction,title_key: &str, message_key: &str){
    aci.create_interaction_response(&ctx, |response|{
        response.kind(ChannelMessageWithSource)
            .interaction_response_data(|data|{
                data.embed(|embed|{
                    embed.colour(GREEN_COLOR)
                        .title(get_key(&aci.locale.as_str(), title_key))
                        .description(get_key(&aci.locale.as_str(), message_key))
                })
            })
    }).await.unwrap();
}

pub async fn send_error_from_aci(ctx : &Context, aci: &ApplicationCommandInteraction,title_key: &str, message_key: &str){
    aci.create_interaction_response(&ctx, |response|{
        response.kind(ChannelMessageWithSource)
            .interaction_response_data(|data|{
                data.embed(|embed|{
                    embed.colour(RED_COLOR)
                        .title(get_key(&aci.locale.as_str(), title_key))
                        .description(get_key(&aci.locale.as_str(), message_key))
                })
            })
    }).await.unwrap();
}

pub async fn get_guild_from_aci(ctx: &Context, aci: &ApplicationCommandInteraction) -> PartialGuild {
    let guild_id = aci.guild_id.expect(format!("Guild can't being obtained from this interaction {}", &aci.id).as_str());
    ctx.http.get_guild(guild_id.0).await.expect(format!("Guild \"{}\" not found", guild_id).as_str())
}

pub async fn create_role(ctx: &Context, guild: &PartialGuild, role_name: &str, permissions: Permissions, color: u64) -> Role {
    let admin = guild.create_role(&ctx.http, |role|{
        role.name(role_name)
            .permissions(permissions)
            .colour(color)
    })
        .await.expect(format!("Creation of {} role failed for the server named {}", role_name, guild.name.as_str()).as_str());
    log(SUCCESS, format!("{} role created for the server named {}", role_name, guild.name.as_str()).as_str());
    return admin;
}

pub async fn create_guild_category(ctx: &Context, guild: &PartialGuild, category_name: &str, permissions: Vec<PermissionOverwrite>) -> GuildChannel {
    let channel = guild.create_channel( &ctx, |channel|{
        channel.kind(ChannelType::Category)
            .name(category_name)
            .permissions(permissions)
    })
        .await.expect(format!("{} category for the server {} can't be created", category_name, guild.name).as_str());

    log(SUCCESS, format!("{} category successfully created for the guild {}", category_name, guild.name).as_str());
    return channel;
}

pub async fn create_channel_on_category(ctx: &Context, guild: &PartialGuild, channel_name: &str, category: &GuildChannel, permissions: Vec<PermissionOverwrite>, channel_type: ChannelType) -> GuildChannel {
    if !((channel_type == ChannelType::Text) || (channel_type == ChannelType::Voice) || (channel_type == ChannelType::Forum)){
        panic!("Channel type must be text or voice");
    }

    let channel = guild.create_channel( &ctx, |channel|{
        channel.kind(channel_type)
            .name(channel_name)
            .permissions(permissions)
            .category(category.id)
    })
        .await.expect(format!("\"{}\" channel  can't be created on the \"{}\" category for the server \"{}\"", channel_name, category.name, guild.name).as_str());

    log(SUCCESS, format!("\"{}\" channel successfully created on the \"{}\" category for the guild \"{}\"", channel_name, category.name, guild.name).as_str());
    return channel;
}

