use std::collections::HashMap;
use serenity::client::Context;
use serenity::model::channel::ChannelType;
use serenity::model::guild::PartialGuild;
use crate::constants::{NRP_GENERAL_VOICE_CHANNEL_NAME, ADMIN_ROLE, MODERATOR_ROLE, PLAYER_ROLE, ADMIN_CATEGORY_NAME, NRP_CATEGORY_NAME, RP_CATEGORY_NAME, ROAD_CATEGORY_NAME, ADMIN_MODERATION_CHANNEL_NAME, ADMIN_COMMANDS_CHANNEL_NAME, NRP_GENERAL_CHANNEL_NAME, NRP_GENERAL_RULES_CHANNEL_NAME, RP_STORY_CHANNEL_NAME, RP_PLAYER_CHARACTERS_CHANNEL_NAME, RP_INDEX_CHANNEL_NAME, RP_RULES_CHANNEL_NAME, RP_QA_CHANNEL_NAME, NRP_RP_EXCHANGES_CHANNEL_NAME, SPECTATOR_ROLE};

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
                            NRP_GENERAL_VOICE_CHANNEL_NAME => {
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
            MODERATOR_ROLE => {
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
    guild.reorder_roles(&ctx, sorted_roles).await.unwrap();
}