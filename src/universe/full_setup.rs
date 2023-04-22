use serenity::client::Context;
use serenity::model::channel::{ChannelType, GuildChannel, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::guild::{PartialGuild, Role};
use serenity::model::Permissions;
use tokio::join;
use crate::common_functions::{create_channel_on_category, create_guild_category, create_role};
use crate::constants::{ADMIN_CATEGORY_NAME, ADMIN_COMMANDS_CHANNEL_NAME, ADMIN_MODERATION_CHANNEL_NAME, ADMIN_ROLE, ADMIN_ROLE_COLOR, MODO_ROLE, MODO_ROLE_COLOR, NRP_CATEGORY_NAME, NRP_GENERAL_CHANNEL_NAME, NRP_GENERAL_RULES_CHANNEL_NAME, NRP_GENERAL_VOICE_CHANNEL_NAME, NRP_RP_EXCHANGES_CHANNEL_NAME, SPECTATOR_ROLE, SPECTATOR_ROLE_COLOR};

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

    create_role(&ctx, &guild, MODO_ROLE, moderator_role_permissions, MODO_ROLE_COLOR).await
}

pub async fn spectator_role_setup(ctx: &Context, guild: &PartialGuild) -> Role{
    create_role(&ctx, &guild, SPECTATOR_ROLE, Permissions::default(), SPECTATOR_ROLE_COLOR).await
}

pub async fn admin_category_setup(ctx: &Context, guild: &PartialGuild, everyone: &Role, player_role: &Role) -> GuildChannel{
    let perms_admin = vec![
        PermissionOverwrite{
            allow: Permissions::default(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(everyone.id),
        },
        PermissionOverwrite{
            allow: Permissions::default(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];

    create_guild_category(&ctx, &guild, ADMIN_CATEGORY_NAME, perms_admin).await
}

pub async fn setup_admin_channels(ctx: &Context, guild: &PartialGuild, admin_category: &GuildChannel) -> Vec<GuildChannel>{
    let moderation = create_channel_on_category(&ctx, &guild, ADMIN_MODERATION_CHANNEL_NAME, &admin_category, vec![], ChannelType::Text);
    let commands = create_channel_on_category(&ctx, &guild, ADMIN_COMMANDS_CHANNEL_NAME, &admin_category, vec![], ChannelType::Text);
    let channels = join!(moderation, commands);
    vec![channels.0, channels.1]
}

pub async fn nrp_category_setup(ctx: &Context, guild: &PartialGuild, everyone: &Role, player_role: &Role) -> GuildChannel{
    let perms_nrp = vec![
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(everyone.id),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::default(),
            kind: PermissionOverwriteType::Role(player_role.id),
        },
    ];

    create_guild_category(&ctx, &guild, NRP_CATEGORY_NAME, perms_nrp).await
}

pub async fn setup_nrp_channels(ctx: &Context, guild: &PartialGuild, nrp_category: &GuildChannel) -> Vec<GuildChannel> {
    let general = create_channel_on_category(&ctx, &guild, NRP_GENERAL_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Text);
    let general_vocal = create_channel_on_category(&ctx, &guild, NRP_GENERAL_VOICE_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Voice);
    let general_rules = create_channel_on_category(&ctx, &guild, NRP_GENERAL_RULES_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Text);
    let rp_exchanges = create_channel_on_category(&ctx, &guild, NRP_RP_EXCHANGES_CHANNEL_NAME, &nrp_category, vec![], ChannelType::Text);

    let channels = join!(general, general_vocal, general_rules, rp_exchanges);

    vec![channels.0, channels.1, channels.2, channels.3]
}

pub async fn specific_full_setup(ctx: &Context, guild: &PartialGuild, everyone: &Role, player_role: &Role) -> (Role, Role, Role, GuildChannel, GuildChannel) {
    let moderator_role = moderator_role_setup(&ctx, &guild);
    let admin_role = admin_role_setup(&ctx, &guild);
    let spectator_role = spectator_role_setup(&ctx, &guild);

    let (admin_role, moderator_role, spectator_role) = join!(admin_role, moderator_role, spectator_role);

    let admin_category = admin_category_setup(&ctx, &guild, &everyone, &player_role);
    let nrp_category = nrp_category_setup(&ctx, &guild, &everyone, &player_role);
    let (admin_category, nrp_category) = join!(admin_category, nrp_category);

    let admin_channels = setup_admin_channels(&ctx, &guild, &admin_category);
    let nrp_channels = setup_nrp_channels(&ctx, &guild, &nrp_category);

    join!(admin_channels, nrp_channels);
    return (admin_role, moderator_role, spectator_role, admin_category, nrp_category);
}