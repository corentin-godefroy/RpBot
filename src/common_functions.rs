use std::ops::BitAnd;
use std::time::SystemTime;
use colored::Colorize;
use serenity::client::Context;
use serenity::model::application::interaction::InteractionResponseType::ChannelMessageWithSource;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use crate::lang::lang_loader::get_key;
use crate::constants::{GREEN_COLOR, ORANGE_COLOR, RED_COLOR, RPBOT_BDD, SERVER_COLLECTION, SERVER_ID, STATS_COLLECTION, UNIVERSAL_STATS, UNIVERSE_ID};
use chrono::{Timelike, Local, Datelike};

use serenity::model::channel::{ChannelType, GuildChannel, PermissionOverwrite};
use serenity::model::guild::{Member, PartialGuild, Role};
use serenity::model::Permissions;


use crate::common_functions::ReportType::{ERROR, SUCCESS, WARNING};

use mongodb::bson::{Array, Bson, doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use serenity::model::id::{ChannelId, GuildId, RoleId};
use crate::MONGOCLIENT;
use serenity::futures::StreamExt;
use serenity::model::prelude::interaction_trait::InteractionResponse;
use crate::bdd::stats::Stats;


pub enum ReportType {
    SUCCESS,
    WARNING,
    ERROR
}

fn get_report_color(report_type: &ReportType) -> u64 {
    match report_type {
        SUCCESS => {GREEN_COLOR}
        ReportType::WARNING => {ORANGE_COLOR}
        ReportType::ERROR => {RED_COLOR}
    }
}

pub fn log(log_type: ReportType, message: &str){
    let log_color;
    let log_type_str = match log_type{
        SUCCESS => {
            log_color = colored::Color::Green;
            "SUCCESS"
        }
        WARNING => {
            log_color = colored::Color::TrueColor { r: 255, g: 160, b: 0 };
            "WARNING"
        }
        ERROR => {
            log_color = colored::Color::Red;
            "ERROR"}
    };
    let current = Local::now();
    let date = format!("d{:02} m{:02} {:02}:{:02}:{:02}", current.day() ,current.month(), current.hour(), current.minute(), current.second());
    let message = format!("[{}] {:07} : {}", date, log_type_str.color(log_color), message);
    println!("{}", message);
}

pub async fn send_report_localized<'a, R: InteractionResponse + std::marker::Sync>(ctx : &Context, interaction: &'a R, report_type : ReportType, title_key: &str, message_key: &str, ephemeral : bool) -> serenity::Result<()>
{
    let color = get_report_color(&report_type);

    interaction.create_interaction_response(&ctx, |response|{
        response.kind(ChannelMessageWithSource)
            .interaction_response_data(|data|{
                data.embed(|embed|{
                    embed.colour(color)
                        .title(get_key(interaction.get_locale(), title_key))
                        .description(get_key(interaction.get_locale(), message_key))
                })
                    .ephemeral(ephemeral)
            })
    }).await
}

pub async fn send_report<'a, R: InteractionResponse + std::marker::Sync>(ctx : &Context, interaction: &'a R, report_type : ReportType, title: &str, message: &str, ephemeral : bool) -> serenity::Result<()>
{
    let color = get_report_color(&report_type);
    interaction.create_interaction_response(&ctx, |response|{
        response.kind(ChannelMessageWithSource)
            .interaction_response_data(|data|{
                data.embed(|embed|{
                    embed.colour(color)
                        .title(title)
                        .description(message)
                })
                    .ephemeral(ephemeral)
            })
    }).await
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

pub fn verify_permission(member : &Member, permissions : &Permissions) -> bool {
    let res = member.permissions.unwrap().bitand(*permissions);
    if res == *permissions{
        return true;
    }
    return false;
}

pub async fn get_roles_id(server_id : i64, role_field : &str) -> Vec<Document> {
    let client = MONGOCLIENT.get().unwrap();
    let collection : Collection<Document> = client.database(RPBOT_BDD).collection(SERVER_COLLECTION);

    let role_id: Vec<Document> = collection.aggregate(
        vec![
            doc!{
                "$match": doc!{
                    SERVER_ID : server_id
                }
            },
            doc!{
                "$project": doc!{
                    role_field : 1
                }
            }
        ],
        None
    )
        .await
        .expect("Failed to aggregate")
        .map(|result| result.expect("Failed to get result"))
        .collect()
        .await;

    return role_id
}

pub async fn verify_role(role_field : &str, server_id : i64, member : &Member) -> bool {
    let role_id: Vec<Document> = get_roles_id(server_id, role_field).await;

    if role_id.len() != 1 {
        panic!("Bad number of role.");
    }

    let role_id = role_id.get(0).unwrap().get(role_field).unwrap().as_i64().unwrap() as u64;

    return match member.roles.contains(&RoleId(role_id)) {
        true => { true }
        false => { false }
    }
}

pub async fn get_server_stats(guild_id: &GuildId) -> Vec<Stats> {
    let collection : Collection<Document> = MONGOCLIENT.get().unwrap().database(RPBOT_BDD).collection(SERVER_COLLECTION);
    let server_stats: Vec<Stats> = collection.aggregate(
        [
            doc! {
                "$match": doc! {
                    "server_id": guild_id.0 as i64
                }
            },
            doc! {
                "$lookup": doc! {
                    "from": "stats",
                    "localField": "universe_id",
                    "foreignField": "universe_id",
                    "as": "stat"
                }
            },
            doc! {
                "$set": doc! {
                    "stat": doc! {
                        "$arrayElemAt": [
                            "$stat",
                            0
                        ]
                    }
                }
            },
            doc! {
                "$unwind": doc! {
                    "path": "$stat.universal_stats",
                    "preserveNullAndEmptyArrays": false
                }
            },
            doc! {
                "$replaceRoot": doc! {
                    "newRoot": "$stat.universal_stats"
                }
            }
        ],
        None
    )
        .await
        .expect("Failed to aggregate")
        .with_type()
        .map(|result| result.expect("Failed to get result"))
        .collect()
        .await;

    server_stats
}

pub async fn get_universe_id(guild_id : u64) -> ObjectId {
    let client = MONGOCLIENT.get().unwrap();
    let collection : Collection<Document> = client.database(RPBOT_BDD).collection(SERVER_COLLECTION);

    let servers: Vec<Document> = collection.aggregate(
        vec![
            doc!{
                "$match": doc!{
                    SERVER_ID : guild_id as i64
                }
            },
            doc!{
                "$project": doc!{
                    UNIVERSE_ID : 1
                }
            }
        ],
        None
    )
        .await
        .expect("Failed to aggregate")
        .map(|result| result.expect("Failed to get result"))
        .collect()
        .await;

    if servers.len() != 1{
        println!("erreur");
    }
    let server = servers.get(0).unwrap();
    return server.get(UNIVERSE_ID).unwrap().clone().as_object_id_mut().unwrap().clone();
}

pub fn get_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get current timestamp")
        .as_secs()
}

pub async fn get_parent_category(ctx: &Context, guild_id : u64, channel_id : u64) -> Option<ChannelId> {
    let guild = get_guild_from_id(&ctx, guild_id).await.unwrap();
    let channels = &guild.channels(&ctx.http).await.unwrap();
    match channels.get(&ChannelId(channel_id)){
        None => {
            log(ERROR, format!("Can't recover parent from channel id {}", channel_id).as_str());
            None
        }
        Some(channel) => {
            match channel.parent_id {
                None => {
                    log(ERROR, format!("Channel are not categorized {}", channel_id).as_str());
                    None
                }
                Some(category) => {Some(category)}
            }
        }
    }
}

pub async fn get_parent_category_resolved<R : InteractionResponse + std::marker::Sync>(ctx : &Context,interaction : R, guild_id : u64, channel_id : u64) -> ChannelId {
    match get_parent_category(&ctx, guild_id, channel_id).await {
        None => {
            send_report_localized(
                &ctx, &interaction, ERROR,
                "", "", false
            ).await.unwrap();
            panic!("");
        }
        Some(category) => { category }
    }
}

pub async fn get_guild_from_id(ctx : &Context, guild_id : u64) -> Option<PartialGuild> {
    match &ctx.http.get_guild(guild_id).await {
        Ok(guild) => {Some(guild.clone())}
        Err(_) => {
            log(ERROR, format!("Can't recover guild from id ({})", guild_id).as_str());
            None
        }
    }
}