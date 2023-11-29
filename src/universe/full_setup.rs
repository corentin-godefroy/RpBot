use serenity::client::Context;
use serenity::model::channel::{ChannelType, GuildChannel, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::guild::{PartialGuild, Role};
use serenity::model::id::RoleId;
use serenity::model::Permissions;
use serenity::model::prelude::application_command::ApplicationCommandInteraction;
use tokio::join;
use crate::bdd::server::Server;
use crate::common_functions::{create_channel_on_category, create_guild_category, create_role, get_guild_from_aci};
use crate::constants::{ADMIN_CATEGORY_NAME, ADMIN_COMMANDS_CHANNEL_NAME, ADMIN_MODERATION_CHANNEL_NAME, ADMIN_ROLE, ADMIN_ROLE_COLOR, MODERATOR_ROLE, MODO_ROLE_COLOR, NRP_CATEGORY_NAME, NRP_GENERAL_CHANNEL_NAME, NRP_GENERAL_RULES_CHANNEL_NAME, NRP_GENERAL_VOICE_CHANNEL_NAME, NRP_RP_EXCHANGES_CHANNEL_NAME, PLAYER_ROLE, PLAYER_ROLE_COLOR, SPECTATOR_ROLE, SPECTATOR_ROLE_COLOR};
use crate::universe::partial_setup::partial_setup;

pub async fn full_setup(ctx: &Context, aci : &ApplicationCommandInteraction, guild: &PartialGuild, server: &mut Server){
    partial_setup(&ctx, aci, &guild, server).await;

    let admin_role = admin_role_setup(&ctx, &guild);
    let moderator_role = moderator_role_setup(&ctx, &guild);
    let spectator_role = spectator_role_setup(&ctx, &guild);

    let guild = get_guild_from_aci(&ctx, &aci).await;


    let (admin, moderator, spectator) = join!(admin_role, moderator_role, spectator_role);
    server.admin_role_id = admin.id.0;
    server.moderator_role_id = moderator.id.0;
    server.spectator_role_id = spectator.id.0;

    admin_setup(&ctx, &guild, server).await;
    nrp_setup(&ctx, &guild, server).await;
}

pub async fn admin_role_setup(ctx: &Context, guild: &PartialGuild) -> Role{
    let admin_role_permissions: Permissions = Permissions::ADMINISTRATOR;
    create_role(&ctx, &guild, ADMIN_ROLE, admin_role_permissions, ADMIN_ROLE_COLOR).await
}
pub async fn moderator_role_setup(ctx : &Context, guild : &PartialGuild) -> Role{
    let moderator_role_permissions: Permissions =
        Permissions::KICK_MEMBERS |
            Permissions::VIEW_CHANNEL |
            Permissions::USE_SLASH_COMMANDS |
            Permissions::SEND_MESSAGES |
            Permissions::BAN_MEMBERS |
            Permissions::MANAGE_MESSAGES |
            Permissions::MANAGE_ROLES;

    create_role(&ctx, &guild, MODERATOR_ROLE, moderator_role_permissions, MODO_ROLE_COLOR).await
}
pub async fn spectator_role_setup(ctx: &Context, guild: &PartialGuild) -> Role{
    create_role(&ctx, &guild, SPECTATOR_ROLE, Permissions::default(), SPECTATOR_ROLE_COLOR).await
}


pub async fn admin_setup(ctx: &Context, guild: &PartialGuild, server: &mut Server){
    let perms_admin = vec![
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

    let admin_category = create_guild_category(&ctx, &guild, ADMIN_CATEGORY_NAME, perms_admin).await;
    setup_admin_channels(&ctx, &guild, &admin_category).await;
    server.admin_category_id = admin_category.id.0;
}
pub async fn setup_admin_channels(ctx: &Context, guild: &PartialGuild, admin_category: &GuildChannel){
    let moderation = create_channel_on_category(&ctx, &guild, ADMIN_MODERATION_CHANNEL_NAME, &admin_category, vec![], ChannelType::Text);
    let commands = create_channel_on_category(&ctx, &guild, ADMIN_COMMANDS_CHANNEL_NAME, &admin_category, vec![], ChannelType::Text);
    join!(moderation, commands);
}

pub async fn nrp_setup(ctx: &Context, guild: &PartialGuild, server: &mut Server){
    let perms_nrp = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(RoleId::from(server.everyone_role_id)),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(RoleId::from(server.player_role_id))
        },
    ];

    let nrp_category = create_guild_category(&ctx, &guild, NRP_CATEGORY_NAME, perms_nrp).await;
    setup_nrp_channels(&ctx, &guild, &nrp_category, server).await;
    server.nrp_category_id = nrp_category.id.0;
}

pub async fn setup_nrp_channels(ctx: &Context, guild: &PartialGuild, nrp_category: &GuildChannel, server: &mut Server){
    let general = create_channel_on_category(&ctx, &guild, NRP_GENERAL_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Text);
    let general_vocal = create_channel_on_category(&ctx, &guild, NRP_GENERAL_VOICE_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Voice);
    let general_rules = create_channel_on_category(&ctx, &guild, NRP_GENERAL_RULES_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Text);
    let rp_exchanges = create_channel_on_category(&ctx, &guild, NRP_RP_EXCHANGES_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Text);

    join!(general, general_vocal, general_rules, rp_exchanges);
}