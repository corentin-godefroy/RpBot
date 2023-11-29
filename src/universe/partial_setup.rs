use serenity::client::Context;
use serenity::model::channel::{ChannelType, GuildChannel, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::guild::{PartialGuild, Role};
use serenity::model::id::RoleId;
use serenity::model::Permissions;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use tokio::join;
use crate::bdd::server::Server;
use crate::common_functions::{create_channel_on_category, create_guild_category, create_role};
use crate::constants::{PLAYER_ROLE, PLAYER_ROLE_COLOR, ROAD_CATEGORY_NAME, RP_CATEGORY_NAME, RP_INDEX_CHANNEL_NAME, RP_PLAYER_CHARACTERS_CHANNEL_NAME, RP_QA_CHANNEL_NAME, RP_RULES_CHANNEL_NAME, RP_STORY_CHANNEL_NAME};

pub async fn player_role_setup(ctx: &Context, guild: &PartialGuild) -> Role{
    let player_role_permissions: Permissions =
        Permissions::empty() |
            Permissions::SEND_MESSAGES |
            Permissions::SEND_MESSAGES_IN_THREADS |
            Permissions::USE_SLASH_COMMANDS |
            Permissions::CONNECT |
            Permissions::SPEAK;

    create_role(&ctx, &guild, PLAYER_ROLE, player_role_permissions, PLAYER_ROLE_COLOR).await
}

pub async fn partial_setup(ctx: &Context, aci : &ApplicationCommandInteraction, guild: &PartialGuild, server: &mut Server){
    let player_role = player_role_setup(&ctx, &guild).await;
    let everyone = guild.role_by_name("@everyone").unwrap();
    server.player_role_id = player_role.id.0;
    server.everyone_role_id = everyone.id.0;

    setup_road(ctx, guild, server).await;
    setup_rp(ctx, aci, guild, server).await;
}

pub async fn setup_road(ctx: &Context, guild: &PartialGuild, server: &mut Server){
    let perms_roads = vec![
        PermissionOverwrite{
            allow: Permissions::default(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::default(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id)),
        },
    ];
    let road_category = create_guild_category(&ctx, &guild, ROAD_CATEGORY_NAME, perms_roads).await;
    server.road_category_id = road_category.id.0;
}

pub async fn setup_rp(ctx: &Context, aci : &ApplicationCommandInteraction, guild : &PartialGuild, server: &mut Server){
    let perms_rp = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id)),
        },
    ];
    let category = create_guild_category(&ctx, &guild, RP_CATEGORY_NAME, perms_rp).await;
    server.rp_category_id = category.id.0;
    setup_story_channel(&ctx, &guild, &category, server).await;
    setup_rp_rules_channel(&ctx, &guild, &category, server).await;
    setup_character_channel(&ctx, &guild, &category, server).await;
    if guild.features.contains(&"COMMUNITY".to_string()) {
        setup_rp_index(&ctx, &guild, &category, server).await;
        setup_qa_channel(&ctx, &guild, &category, server).await;
    } else {
        aci.channel_id.send_message(&ctx, |m| {
            m.content("Le serveur n'est pas communautaire, les channels index et Q&A n'ont pas pu être créés.")
        })
            .await.unwrap();
    }
}

async fn setup_story_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, server : &Server) -> GuildChannel {
    let perms_story = vec![
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id)),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_STORY_CHANNEL_NAME, &rp_category, perms_story, ChannelType::Text).await
}

async fn setup_rp_rules_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, server : &Server){
    let perms_rules = vec![
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id)),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_RULES_CHANNEL_NAME, &rp_category, perms_rules, ChannelType::Text).await;
}

async fn setup_character_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, server : &mut Server){
    let perms_player_character = vec![
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL |
                Permissions::SEND_MESSAGES,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id)),
        },
    ];
    let channel = create_channel_on_category(&ctx, &guild, RP_PLAYER_CHARACTERS_CHANNEL_NAME, &rp_category, perms_player_character, ChannelType::Text).await;
    server.character_channel_id = channel.id.0;
}

async fn setup_rp_index(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, server : &mut Server) {
    let perms_index = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id)),
        },
    ];
    let channel = create_channel_on_category(&ctx, &guild, RP_INDEX_CHANNEL_NAME, &rp_category, perms_index, ChannelType::Forum).await;
    server.index_forum_id = channel.id.0;
}

pub async fn setup_qa_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, server : &Server){
    let perms_qa = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY |
                Permissions::SEND_MESSAGES,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY |
                Permissions::SEND_MESSAGES,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id)),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_QA_CHANNEL_NAME, &rp_category, perms_qa, ChannelType::Forum).await;
}