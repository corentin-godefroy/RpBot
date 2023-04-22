use std::collections::HashMap;
use serenity::client::Context;
use serenity::model::channel::ChannelType;
use serenity::model::guild::PartialGuild;
use crate::constants::{CREATE_UNIVERSE_CAMMAND_LOCALE_DESCRIPTION, CREATE_UNIVERSE_COMMAND_LOCALE_NAME, CREATE_UNIVERSE_NAME_LOCALE_OPTION, CREATE_UNIVERSE_NAME_OPTION_LOCALE_DESCRIPTION, DEFAULT_LANG, RPBOT_BDD, UNIVERSE_COLLECTION, SERVERS_UNIVERSE_FIELD, CREATE_UNIVERSE_ERROR_UNIVERSE_ALREADY_EXIST, UNIVERSE_NAME_FIELD, CREATE_UNIVERSE_SUCCESS, CREATE_UNIVERSE_SUCCESS_TITLE, CREATE_UNIVERSE_ERROR_ALREADY_EXIST_TITLE, UNIVERSE_ADMIN_ID, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION_DESCRIPTION, ADMIN_ROLE, MODO_ROLE, PLAYER_ROLE, PLAYER_ROLE_COLOR, SERVER_COLLECTION, SERVER_ID, ADMIN_ROLE_ID, MODERATOR_ROLE_ID, PLAYER_ROLE_ID, ADMIN_CATEGORY_NAME, NRP_CATEGORY_NAME, RP_CATEGORY_NAME, ROAD_CATEGORY_NAME, ROAD_CATEGORY_ID, RP_CATEGORY_ID, NRP_CATEGORY_ID, ADMIN_CATEGORY_ID, ADMIN_MODERATION_CHANNEL_NAME, ADMIN_COMMANDS_CHANNEL_NAME, NRP_GENERAL_CHANNEL_NAME, NRP_GENERAL_RULES_CHANNEL_NAME, RP_STORY_CHANNEL_NAME, RP_PLAYER_CHARACTERS_CHANNEL_NAME, RP_INDEX_CHANNEL_NAME, RP_RULES_CHANNEL_NAME, RP_QA_CHANNEL_NAME, NRP_RP_EXCHANGES_CHANNEL_NAME, GREEN_COLOR, SPECTATOR_ROLE, SPECTATOR_ROLE_ID};

pub async fn sort_channels(ctx : &Context, guild : &PartialGuild){
    let channels = guild.channels(&ctx.http).await.unwrap();
    let mut sorted_channels = HashMap::new();
    for channel in channels.values() {
        match channel.kind {
            ChannelType::Category => {
                match channel.name.as_str() {
                    ADMIN_CATEGORY_NAME => {
                        sorted_channels.insert(channel.id, 1);
                    }
                    NRP_CATEGORY_NAME => {
                        sorted_channels.insert(channel.id, 4);
                    }
                    RP_CATEGORY_NAME => {
                        sorted_channels.insert(channel.id, 10);
                    }
                    ROAD_CATEGORY_NAME => {
                        sorted_channels.insert(channel.id, 16);
                    }
                    _ => {}
                }
            }
            ChannelType::Text => {
                if channel.parent_id.is_none() {
                    continue;
                }
                let channel_parent = ctx.http.get_channel(channel.parent_id.unwrap().0).await.unwrap();

                match channel_parent.category().unwrap().name.as_str() {
                    ADMIN_CATEGORY_NAME => {
                        match channel.name.as_str() {
                            ADMIN_MODERATION_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 2);
                            }
                            ADMIN_COMMANDS_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 3);
                            }
                            _ => {}
                        }
                    }
                    NRP_CATEGORY_NAME => {
                        match channel.name.as_str() {
                            NRP_GENERAL_RULES_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 5);
                            }
                            NRP_GENERAL_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 6);
                            }
                            NRP_RP_EXCHANGES_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 7);
                            }
                            _ => {}
                        }
                    }
                    RP_CATEGORY_NAME => {
                        match channel.name.as_str() {
                            RP_STORY_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 11);
                            }
                            RP_PLAYER_CHARACTERS_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 13);
                            }
                            RP_RULES_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 12);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }

            }
            ChannelType::Voice => {
                let channel_parent = ctx.http.get_channel(channel.parent_id.unwrap().0).await.unwrap();
                match channel_parent.category().unwrap().name.as_str() {
                    NRP_CATEGORY_NAME => {
                        match channel.name.as_str() {
                            NRP_VOICE_CHANNEL_NAME => {
                                sorted_channels.insert(channel.id, 8);
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            ChannelType::Forum => {
                match channel.name.as_str() {
                    RP_INDEX_CHANNEL_NAME => {
                        sorted_channels.insert(channel.id, 14);
                    }
                    RP_QA_CHANNEL_NAME => {
                        sorted_channels.insert(channel.id, 15);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
    guild.reorder_channels(&ctx.http, sorted_channels).await.unwrap();
}

pub async fn sort_roles(ctx : &Context, guild : &PartialGuild){
    let roles = ctx.http.get_guild_roles(guild.id.0).await.unwrap();
    let mut sorted_roles = HashMap::new();
    for role in roles {
        match role.name.as_str() {
            ADMIN_ROLE => {
                sorted_roles.insert(role.id, 4);
            }
            MODO_ROLE => {
                sorted_roles.insert(role.id, 3);
            }
            SPECTATOR_ROLE => {
                sorted_roles.insert(role.id, 2);
            }
            PLAYER_ROLE => {
                sorted_roles.insert(role.id, 1);
            }
            _ => {}
        }
    }
    guild.edit_roles_positions(&ctx, sorted_roles).await.unwrap();
}