use serenity::client::Context;
use serenity::model::channel::{ChannelType, GuildChannel, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::guild::{PartialGuild, Role};
use serenity::model::Permissions;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use tokio::join;
use crate::common_functions::{create_channel_on_category, create_guild_category, create_role};
use crate::constants::{PLAYER_ROLE, PLAYER_ROLE_COLOR, ROAD_CATEGORY_NAME, RP_CATEGORY_NAME, RP_INDEX_CHANNEL_ID, RP_INDEX_CHANNEL_NAME, RP_PLAYER_CHARACTERS_CHANNEL_ID, RP_PLAYER_CHARACTERS_CHANNEL_NAME, RP_QA_CHANNEL_ID, RP_QA_CHANNEL_NAME, RP_RULES_CHANNEL_ID, RP_RULES_CHANNEL_NAME, RP_STORY_CHANNEL_ID, RP_STORY_CHANNEL_NAME};

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
pub async fn setup_rp_category(ctx: &Context, guild: &PartialGuild, everyone_role: &Role, player_role: &Role) -> GuildChannel {
    let perms_rp = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];
    create_guild_category(&ctx, &guild, RP_CATEGORY_NAME, perms_rp).await
}

pub async fn setup_road_category(ctx: &Context, guild: &PartialGuild, everyone_role: &Role, player_role: &Role) -> GuildChannel {
    let perms_roads = vec![
        PermissionOverwrite{
            allow: Permissions::default(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::default(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];
    create_guild_category(&ctx, &guild, ROAD_CATEGORY_NAME, perms_roads).await
}

async fn setup_story_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, everyone_role: &Role, player_role : &Role) -> GuildChannel {
    let perms_story = vec![
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_STORY_CHANNEL_NAME, &rp_category, perms_story, ChannelType::Text).await
}

async fn setup_rp_rules_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, everyone_role: &Role, player_role : &Role) -> GuildChannel {
    let perms_rules = vec![
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_RULES_CHANNEL_NAME, &rp_category, perms_rules, ChannelType::Text).await
}

async fn setup_player_character_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, everyone_role: &Role, player_role : &Role) -> GuildChannel {
    let perms_player_character = vec![
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL |
                Permissions::SEND_MESSAGES,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::READ_MESSAGE_HISTORY |
                Permissions::VIEW_CHANNEL,
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_PLAYER_CHARACTERS_CHANNEL_NAME, &rp_category, perms_player_character, ChannelType::Text).await
}

async fn setup_rp_index(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, everyone_role: &Role, player_role : &Role) -> GuildChannel {
    let perms_index = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_INDEX_CHANNEL_NAME, &rp_category, perms_index, ChannelType::Forum).await
}

pub async fn setup_qa_channel(ctx : &Context, guild : &PartialGuild, rp_category : &GuildChannel, everyone_role: &Role, player_role : &Role) -> GuildChannel {
    let perms_qa = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY |
                Permissions::SEND_MESSAGES,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::READ_MESSAGE_HISTORY |
                Permissions::SEND_MESSAGES,
            deny: Permissions::CREATE_PUBLIC_THREADS |
                Permissions::SEND_MESSAGES_IN_THREADS,
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];
    create_channel_on_category(&ctx, &guild, RP_QA_CHANNEL_NAME, &rp_category, perms_qa, ChannelType::Forum).await
}

pub async fn setup_rp_channels<'a>(ctx : &'a Context, aci : &'a ApplicationCommandInteraction, guild : &'a PartialGuild, rp_category : &'a GuildChannel, everyone_role: &'a Role, player_role : &'a Role) -> Vec<(&'a str, GuildChannel)> {
    let story_channel = setup_story_channel(&ctx, &guild, &rp_category, &everyone_role, &player_role);
    let rp_rules_channel = setup_rp_rules_channel(&ctx, &guild, &rp_category, &everyone_role, &player_role);
    let player_characters_channel = setup_player_character_channel(&ctx, &guild, &rp_category, &everyone_role, &player_role);

    if guild.features.contains(&"COMMUNITY".to_string()) {
        let rp_index = setup_rp_index(&ctx, &guild, &rp_category, &everyone_role, &player_role);
        let rp_qa = setup_qa_channel(&ctx, &guild, &rp_category, &everyone_role, &player_role);
        let (rp_index, rp_qa, story_channel, rp_rules_channel, player_characters_channel ) = join!(rp_index, rp_qa, story_channel, rp_rules_channel, player_characters_channel);
        return vec![(RP_INDEX_CHANNEL_ID, rp_index), (RP_QA_CHANNEL_ID, rp_qa), (RP_STORY_CHANNEL_ID, story_channel), (RP_RULES_CHANNEL_ID, rp_rules_channel), ( RP_PLAYER_CHARACTERS_CHANNEL_ID, player_characters_channel)];
    } else {
        aci.channel_id.send_message(&ctx, |m| {
            m.content("Le serveur n'est pas communautaire, les channels index et Q&A n'ont pas pu être créés.")
        })
            .await.unwrap();

        let (story_channel, rp_rules_channel, player_characters_channel) = join!(story_channel, rp_rules_channel, player_characters_channel);
        return vec![(RP_STORY_CHANNEL_ID, story_channel), (RP_RULES_CHANNEL_ID, rp_rules_channel), ( RP_PLAYER_CHARACTERS_CHANNEL_ID, player_characters_channel)];
    };


}