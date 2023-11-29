use std::cmp::max;
use mongodb::bson::{doc, Document, to_document};
use mongodb::Collection;
use serenity::builder::CreateComponents;
use serenity::client::Context;
use serenity::model::application::command::Command;
use serenity::model::permissions::Permissions;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use crate::constants::{BAD_CHANNEL_ERROR, BAD_CHANNEL_ERROR_TITLE, BAD_PERMISSIONS_VALIDATION_CHARACTERS, BAD_PERMISSIONS_CHARACTERS_TITLE, CREATE_NEW_PLAYER_COMMAND_LOCALE_DESCRIPTION, CREATE_NEW_PLAYER_COMMAND_LOCALE_NAME, CREATE_PLAYER_ACCEPT_BUTTON_CUSTOM_ID, CREATE_PLAYER_ACCEPT_BUTTON_LOCALE, CREATE_PLAYER_CANCEL_BUTTON_CUSTOM_ID, CREATE_PLAYER_CANCEL_BUTTON_LOCALE, CREATE_PLAYER_FROM_PLAYER_CUSTOM_ID, CREATE_PLAYER_MODAL_TITLE, CREATE_PLAYER_MODIFY_BUTTON_CUSTOM_ID, CREATE_PLAYER_MODIFY_BUTTON_LOCALE, CREATE_PLAYER_REJECT_BUTTON_CUSTOM_ID, CREATE_PLAYER_REJECT_BUTTON_LOCALE, CREATE_PLAYER_VALIDATE_BUTTON_CUSTOM_ID, DEFAULT_LANG, LIGHT_BLUE_COLOR, MODERATOR_ROLE_ID, PLAYER_ALREADY_EXIST_ERROR, PLAYER_ALREADY_EXIST_ERROR_TITLE, PLAYER_COLLECTION, PLAYER_DESC_CUSTOM_ID, PLAYER_DESC_LOCALE_LABEL, PLAYER_DESC_LOCALE_PLACEHOLDER, PLAYER_ID, PLAYER_NAME_CUSTOM_ID, PLAYER_NAME_LOCALE_LABEL, PLAYER_NAME_LOCALE_PLACEHOLDER, PLAYER_STORY_CUSTOM_ID, PLAYER_STORY_LOCALE_LABEL, PLAYER_STORY_LOCALE_PLACEHOLDER, RP_PLAYER_CHARACTERS_CHANNEL_ID, RPBOT_BDD, SERVER_COLLECTION, SERVER_ID, UNIVERSE_ID, BAD_PERMISSIONS_REJECTION_CHARACTERS, REJECT_REASON_TITLE_FIELD, REJECT_REASON_INTERACTION_TITLE, REJECT_REASON_INTERACTION_ID, REJECT_REASON_CUSTOM_ID, REJECT_REASON_CONTENT_FORMATER, FINAL_SETUP_PLAYER_INTERACTION_TITLE, FINAL_SETUP_PLAYER_INTERACTION_CUSTOM_ID, FINAL_SETUP_PLAYER_INPUT_TEXT_ID, FINAL_SETUP_PLAYER_LABEL, DEFAULT_CUSTOM_ID, PLAYER_TITLE, GREEN_COLOR, NEW_PLAYER_INSERTED_SUCCESS_MESSAGE, PLAYER_DESTINATION_ID, PLAYER_CURRENT_POSITION_ID, PLAYER_START_TIMESTAMP, PLAYER_END_TIMESTAMP, PLAYER_IS_IN_MOVE, PLAYER_IS_DEAD, PLAYER_DESTINATION_SERVER_ID, PLAYER_STATS, PLAYER_POSITION_TIMESTAMP, PLAYER_NAME, PLAYER_PRIVILEGES, SPEED_STAT, MOUNT_SPEED_STAT};
use crate::lang::lang_loader::get_key;
use crate::{LANGS, MONGOCLIENT};
use crate::common_functions::{get_roles_id, get_server_stats, get_universe_id, log, ReportType, send_report_localized, verify_permission, verify_role};
use serenity::futures::StreamExt;
use serenity::model::application::component::{ActionRowComponent, ButtonStyle, InputTextStyle};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::message_component::MessageComponentInteraction;
use serenity::model::prelude::interaction::InteractionResponseType::Modal;
use serenity::model::prelude::interaction::modal::ModalSubmitInteraction;
use serenity::model::prelude::interaction_trait::InteractionResponse;
use serenity::model::prelude::InteractionResponseType::{ChannelMessageWithSource, UpdateMessage};
use crate::bdd::global::get_collection;
use crate::bdd::player::Player;
use crate::bdd::stats::{parse_stats, stat_input_to_hash};
use crate::common_functions::ReportType::{ERROR, SUCCESS};


pub async fn create_new_player(ctx : &Context){
    Command::create_global_application_command(&ctx, |command|{
        command.name(get_key(DEFAULT_LANG, CREATE_NEW_PLAYER_COMMAND_LOCALE_NAME))
            .dm_permission(false)
            .description(get_key(DEFAULT_LANG, CREATE_NEW_PLAYER_COMMAND_LOCALE_DESCRIPTION));

        for lang in LANGS.get().unwrap(){
            command.name_localized(lang.0, lang.1[CREATE_NEW_PLAYER_COMMAND_LOCALE_NAME].to_string())
                .description_localized(lang.0, get_key(lang.0, CREATE_NEW_PLAYER_COMMAND_LOCALE_DESCRIPTION));
        }
        command
    })
        .await
        .expect("Error on creation create_new_player command.");
}

pub async fn create_new_player_reactor(ctx : &Context, aci : &ApplicationCommandInteraction){
    let client = MONGOCLIENT.get().unwrap();
    let collection : Collection<Document> = client.database(RPBOT_BDD).collection(SERVER_COLLECTION);

    let universes: Vec<Document> = collection.aggregate(
        vec![
            doc!{
                "$match": doc!{
                    SERVER_ID : *&aci.guild_id.unwrap().0 as i64
                }
            },
            doc!{
                "$project": doc!{
                    UNIVERSE_ID : 1,
                    RP_PLAYER_CHARACTERS_CHANNEL_ID : 1
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

    if universes.len() != 1{
        panic!("erreur plusieurs univers pour un même serveur : {}", *&aci.guild_id.unwrap().0);
    }
    let universe = universes.get(0).unwrap();
    let universe_id = universe.get(UNIVERSE_ID).unwrap();
    let player_character_channel = universe.get(RP_PLAYER_CHARACTERS_CHANNEL_ID).unwrap();

    if aci.channel_id.0 != player_character_channel.as_i64().unwrap() as u64{
        send_report_localized(&ctx, aci, ERROR, BAD_CHANNEL_ERROR_TITLE, BAD_CHANNEL_ERROR, true).await.unwrap();
        return;
    }

    let filter = doc!{
        UNIVERSE_ID : universe_id,
        PLAYER_ID : *&aci.user.id.0 as i64
    };
    let collection = client.database(RPBOT_BDD).collection::<Document>(PLAYER_COLLECTION);
    let results = collection.find_one(filter, None).await.unwrap();
    if results.is_some(){
        send_report_localized(&ctx, aci, ERROR, PLAYER_ALREADY_EXIST_ERROR_TITLE, PLAYER_ALREADY_EXIST_ERROR, true).await.unwrap();
        return;
    }

    aci.create_interaction_response(&ctx, |interaction|{
        interaction.kind(InteractionResponseType::Modal)
            .interaction_response_data(|modal|{
                modal.components(|components|{
                    components
                        .create_action_row(|action_row| {
                            action_row.create_input_text(|input_text| {
                                input_text.custom_id(PLAYER_NAME_CUSTOM_ID)
                                    .required(true)
                                    .label(get_key(&aci.locale, PLAYER_NAME_LOCALE_LABEL))
                                    .placeholder(get_key(&aci.locale, PLAYER_NAME_LOCALE_PLACEHOLDER))
                                    .style(InputTextStyle::Short)
                                    .min_length(4)
                                    .max_length(128)
                            })
                        })
                        .create_action_row(|action_row| {
                            action_row.create_input_text(|input_text| {
                                input_text.custom_id(PLAYER_STORY_CUSTOM_ID)
                                    .required(true)
                                    .label(get_key(&aci.locale, PLAYER_STORY_LOCALE_LABEL))
                                    .placeholder(get_key(&aci.locale, PLAYER_STORY_LOCALE_PLACEHOLDER))
                                    .style(InputTextStyle::Paragraph)
                                    .max_length(4000)
                            })
                        })
                        .create_action_row(|action_row| {
                            action_row.create_input_text(|input_text|{
                            input_text.custom_id(PLAYER_DESC_CUSTOM_ID)
                                .required(true)
                                .label(get_key(&aci.locale, PLAYER_DESC_LOCALE_LABEL))
                                .placeholder(get_key(&aci.locale, PLAYER_DESC_LOCALE_PLACEHOLDER))
                                .style(InputTextStyle::Paragraph)
                                .max_length(4000)
                        })
                    })
                })
                    .title(get_key(&aci.locale, CREATE_PLAYER_MODAL_TITLE))
                    .custom_id(CREATE_PLAYER_FROM_PLAYER_CUSTOM_ID)
            })
    })
        .await
        .expect("Erreur à l'envoi du modal")
}

pub async fn create_player_modal_from_player(ctx : &Context, msi : &ModalSubmitInteraction){
    let player_name = match msi
        .data
        .components
        .get(0)
        .unwrap()
        .components
        .get(0)
        .unwrap()
    {
        ActionRowComponent::InputText(it) => it,
        _ => return,
    };

    let player_story = match msi
        .data
        .components
        .get(1)
        .unwrap()
        .components
        .get(0)
        .unwrap()
    {
        ActionRowComponent::InputText(it) => it,
        _ => return,
    };

    let player_description = match msi
        .data
        .components
        .get(2)
        .unwrap()
        .components
        .get(0)
        .unwrap()
    {
        ActionRowComponent::InputText(it) => it,
        _ => return,
    };

    let mut response_type = ChannelMessageWithSource;
    if msi.message.is_some(){
        response_type = UpdateMessage;
    }

    msi.create_interaction_response(&ctx, |response|{
        response.kind(response_type)
            .interaction_response_data(|data|{
                data.title(get_key(&msi.locale, PLAYER_TITLE))
                    .custom_id(DEFAULT_CUSTOM_ID)
                    .embed(|embed|{
                        embed.colour(LIGHT_BLUE_COLOR)
                            .title(&player_name.value)
                            .field(get_key(&msi.locale, PLAYER_STORY_LOCALE_LABEL), &player_story.value, false)
                            .field(get_key(&msi.locale, PLAYER_DESC_LOCALE_LABEL), &player_description.value, false)
                    })
                    .components(|components|{
                        components
                            .create_action_row(|action_row|{
                                action_row.create_button(|button|{
                                    button.custom_id(CREATE_PLAYER_VALIDATE_BUTTON_CUSTOM_ID)
                                        .style(ButtonStyle::Success)
                                        .label(get_key(&msi.locale, CREATE_PLAYER_ACCEPT_BUTTON_LOCALE))
                                })
                            })
                            .create_action_row(|action_row|{
                                action_row.create_button(|button|{
                                    button.custom_id(CREATE_PLAYER_MODIFY_BUTTON_CUSTOM_ID)
                                        .style(ButtonStyle::Secondary)
                                        .label(get_key(&msi.locale, CREATE_PLAYER_MODIFY_BUTTON_LOCALE))
                                })
                            })
                            .create_action_row(|action_row|{
                                action_row.create_button(|button|{
                                    button.custom_id(CREATE_PLAYER_CANCEL_BUTTON_CUSTOM_ID)
                                        .style(ButtonStyle::Danger)
                                        .label(get_key(&msi.locale, CREATE_PLAYER_CANCEL_BUTTON_LOCALE))
                                })
                            })
                    })
            })
    }).await.expect("error on sending embed");
}

pub async fn create_player_validate_button_trigger(ctx : &Context, mci: &MessageComponentInteraction){
    if mci.user.id.0 != mci.message.interaction.as_ref().unwrap().user.id.0 {
        return;
    }

    let role_id: Vec<Document> = get_roles_id(mci.guild_id.as_ref().unwrap().0 as i64, MODERATOR_ROLE_ID).await;

    let role_id = role_id.get(0).unwrap().get(MODERATOR_ROLE_ID).unwrap().as_i64().unwrap().to_string();

    let component = CreateComponents::create_action_row(&mut Default::default(), |action_row| {
        action_row.create_button(|button| {
            button.custom_id(CREATE_PLAYER_ACCEPT_BUTTON_CUSTOM_ID)
                .style(ButtonStyle::Success)
                .label(get_key(&mci.locale, CREATE_PLAYER_ACCEPT_BUTTON_LOCALE))
        })
            .create_button(|button|{
                button.custom_id(CREATE_PLAYER_REJECT_BUTTON_CUSTOM_ID)
                    .style(ButtonStyle::Danger)
                    .label(get_key(&mci.locale, CREATE_PLAYER_REJECT_BUTTON_LOCALE))
            })
    }).clone();

    mci.create_interaction_response(&ctx.http,|response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|data| {
                data.custom_id(DEFAULT_CUSTOM_ID)
                    .set_components(
                        component
                    )
                    .content(format!("<@&{}>", role_id).as_str())
            })
    }).await.unwrap();
}

pub async fn create_player_modify_button_trigger(ctx : &Context, mci: &MessageComponentInteraction){
    if mci.user.id.0 != mci.message.interaction.as_ref().unwrap().user.id.0 {
        return;
    }
    let mut name = "";
    let mut story = "";
    let mut description = "";
    let message = mci.clone().message;
    if !message.embeds.is_empty(){
        name = message.embeds.get(0).unwrap().title.as_ref().unwrap().as_str();
        story = message.embeds.get(0).unwrap().fields[0].value.as_str().clone();
        description = message.embeds.get(0).unwrap().fields[1].value.as_str().clone();
    }

    mci.create_interaction_response(&ctx, |interaction|{
        interaction.kind(InteractionResponseType::Modal)
            .interaction_response_data(|modal|{
                modal.components(|components|{
                    components
                        .create_action_row(|action_row| {
                            action_row.create_input_text(|input_text| {
                                input_text.custom_id(PLAYER_NAME_CUSTOM_ID)
                                    .required(true)
                                    .label(get_key(&mci.locale, PLAYER_NAME_LOCALE_LABEL))
                                    .placeholder(get_key(&mci.locale, PLAYER_NAME_LOCALE_PLACEHOLDER))
                                    .style(InputTextStyle::Short)
                                    .min_length(4)
                                    .max_length(128)
                                    .value(name)
                            })
                        })
                        .create_action_row(|action_row| {
                            action_row.create_input_text(|input_text| {
                                input_text.custom_id(PLAYER_STORY_CUSTOM_ID)
                                    .required(true)
                                    .label(get_key(&mci.locale, PLAYER_STORY_LOCALE_LABEL))
                                    .placeholder(get_key(&mci.locale, PLAYER_STORY_LOCALE_PLACEHOLDER))
                                    .style(InputTextStyle::Paragraph)
                                    .max_length(4000)
                                    .value(story)
                            })
                        })
                        .create_action_row(|action_row| {
                            action_row.create_input_text(|input_text|{
                                input_text.custom_id(PLAYER_DESC_CUSTOM_ID)
                                    .required(true)
                                    .label(get_key(&mci.locale, PLAYER_DESC_LOCALE_LABEL))
                                    .placeholder(get_key(&mci.locale, PLAYER_DESC_LOCALE_PLACEHOLDER))
                                    .style(InputTextStyle::Paragraph)
                                    .max_length(4000)
                                    .value(description)
                            })
                        })
                })
                    .title(get_key(&mci.locale, CREATE_PLAYER_MODAL_TITLE))
                    .custom_id(CREATE_PLAYER_FROM_PLAYER_CUSTOM_ID)
            })
    })
        .await
        .expect("Erreur à l'envoi du modal")
}

pub async fn create_player_cancel_button_trigger(ctx : &Context, mci: &MessageComponentInteraction){
    if mci.user.id.0 != mci.message.interaction.as_ref().unwrap().user.id.0 {
        return;
    }
    mci.message.delete(&ctx.http).await.unwrap();
}

pub async fn create_player_reject_button_trigger(ctx : &Context, mci: &MessageComponentInteraction){
    let verified_role = verify_role(MODERATOR_ROLE_ID, mci.guild_id.as_ref().unwrap().0 as i64, &mci.member.as_ref().unwrap()).await;
    let verified_permission = verify_permission(&mci.member.as_ref().unwrap(), &Permissions::ADMINISTRATOR);
    if !verified_role && !verified_permission {
        send_report_localized(&ctx, mci, ReportType::ERROR, BAD_PERMISSIONS_CHARACTERS_TITLE, BAD_PERMISSIONS_REJECTION_CHARACTERS, true).await.unwrap();
        return;
    }

    mci.create_interaction_response(&ctx.http, |interaction|{
        interaction.kind(InteractionResponseType::Modal)
            .interaction_response_data(|data|{
                data.custom_id(REJECT_REASON_INTERACTION_ID)
                    .title(get_key(&mci.locale.as_str(), REJECT_REASON_INTERACTION_TITLE))
                    .components(|component|{
                        component.create_action_row(|action_row|{
                            action_row.create_input_text(|input_text|{
                                input_text.custom_id(REJECT_REASON_CUSTOM_ID)
                                    .style(InputTextStyle::Paragraph)
                                    .label(get_key(&mci.locale.as_str(), REJECT_REASON_TITLE_FIELD))
                                    .required(true)
                            })
                        })
                    })
            })
    })
        .await
        .unwrap();
}

pub async fn modify_after_reject(ctx : &Context, msi: &ModalSubmitInteraction){
    let reason = match msi
        .data
        .components
        .get(0)
        .unwrap()
        .components
        .get(0)
        .unwrap()
    {
        ActionRowComponent::InputText(it) => it,
        _ => return,
    };

    let player_tag = msi.user.id.0;

    if msi.user.id.0 != msi.message.as_ref().unwrap().interaction.as_ref().unwrap().user.id.0 {
        return;
    }

    let component = CreateComponents::create_action_row(&mut Default::default(), |action_row| {
        action_row.create_button(|button|{
            button.custom_id(CREATE_PLAYER_VALIDATE_BUTTON_CUSTOM_ID)
                .style(ButtonStyle::Success)
                .label(get_key(&msi.locale, CREATE_PLAYER_ACCEPT_BUTTON_LOCALE))
        })
            .create_button(|button|{
                button.custom_id(CREATE_PLAYER_MODIFY_BUTTON_CUSTOM_ID)
                    .style(ButtonStyle::Secondary)
                    .label(get_key(&msi.locale, CREATE_PLAYER_MODIFY_BUTTON_LOCALE))
            })
            .create_button(|button|{
                button.custom_id(CREATE_PLAYER_CANCEL_BUTTON_CUSTOM_ID)
                    .style(ButtonStyle::Danger)
                    .label(get_key(&msi.locale, CREATE_PLAYER_CANCEL_BUTTON_LOCALE))
            })
    }).clone();

    msi.create_interaction_response(&ctx.http,|response| {
        response.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|data| {
                data.custom_id(DEFAULT_CUSTOM_ID)
                    .set_components(
                        component
                    )
                    .content(format!("<@{}> {} {}", player_tag, get_key(&msi.locale, REJECT_REASON_CONTENT_FORMATER), reason.value).as_str())
            })
    }).await.unwrap();
}

pub async fn create_player_accept_button_trigger(ctx : &Context, mci: &MessageComponentInteraction){
    let verified_role = verify_role(MODERATOR_ROLE_ID, mci.guild_id.as_ref().unwrap().0 as i64, &mci.member.as_ref().unwrap()).await;
    let verified_permission = verify_permission(&mci.member.as_ref().unwrap(), &Permissions::ADMINISTRATOR);
    if !verified_role && !verified_permission{
        send_report_localized(&ctx, mci, ReportType::ERROR, BAD_PERMISSIONS_CHARACTERS_TITLE, BAD_PERMISSIONS_VALIDATION_CHARACTERS, true).await.unwrap();
        return;
    }

    let stats = get_server_stats(&mci.guild_id.unwrap()).await;
    println!("{:?}", stats);
    let mut prefilling = String::new();

    for stat in stats {
        if !stat.hide{
            prefilling = prefilling.to_owned() + stat.name.as_str() + " : " + stat.base_value.to_string().as_str() + "\n";
        }
    }

    mci.create_interaction_response(&ctx, |interaction|{
        interaction.kind(Modal)
            .interaction_response_data(|data|{
                data.custom_id(FINAL_SETUP_PLAYER_INTERACTION_CUSTOM_ID)
                    .title(get_key(&mci.locale, FINAL_SETUP_PLAYER_INTERACTION_TITLE))
                    .components(|component|{
                        component.create_action_row(|action_row|{
                            action_row.create_input_text(|input_text|{
                                input_text.custom_id(FINAL_SETUP_PLAYER_INPUT_TEXT_ID)
                                    .style(InputTextStyle::Paragraph)
                                    .value(prefilling)
                                    .label(get_key(&mci.locale, FINAL_SETUP_PLAYER_LABEL))
                                    .required(true)
                            })
                        })
                    })
            })
    })
        .await
        .expect("erreur à l'envoi du message")
}

pub async fn finalise_player_creation(ctx: &Context, msi: &ModalSubmitInteraction){
    let stats_values = match msi
        .data
        .components
        .get(0)
        .unwrap()
        .components
        .get(0)
        .unwrap()
    {
        ActionRowComponent::InputText(it) => it,
        _ => return,
    };

    let stats_input = stat_input_to_hash(stats_values);
    let server_stats = get_server_stats(&msi.guild_id.unwrap()).await;
    let mut player_stats = parse_stats(server_stats.clone());
    let mut player = Player::default();
    for stat in stats_input{
        player_stats.insert(stat.0.clone(), stat.1.clone());
    }

    player.stats = player_stats.clone().into_values().collect();

    player.id = msi.message.as_ref().unwrap().interaction.as_ref().unwrap().user.id.0;
    let universe_id = get_universe_id(msi.guild_id.unwrap().0).await;
    player.universe_id = universe_id;

    let original_embed = msi.message.as_ref().unwrap().embeds.get(0).unwrap();
    player.name = original_embed.title.as_ref().unwrap().to_string();
    let players = get_collection(PLAYER_COLLECTION);
    players.insert_one(to_document(&player).unwrap(), None).await.unwrap();

    msi.create_interaction_response(&ctx, |interaction|{
        interaction.kind(InteractionResponseType::UpdateMessage)
            .interaction_response_data(|data|{
                data.embed(|embed|{
                    embed.title(&original_embed.title.as_ref().unwrap());
                    for field in &original_embed.fields{
                        embed.field(&field.name, &field.value, field.inline);
                    }
                    embed.colour(GREEN_COLOR)
                })
                    .content(format!("<@{}>, {}", player.id, get_key(&msi.locale, NEW_PLAYER_INSERTED_SUCCESS_MESSAGE)))
                    .set_components(CreateComponents::default())
            })
    })
        .await
        .expect("Erreur à l'envoi du modal");

    let server_name = ctx.http.get_guild(msi.guild_id.unwrap().0).await.unwrap().name;
    log(SUCCESS, format!("Personnage {} créé avec succès pour le serveur {}", &msi.message.as_ref().unwrap().interaction.as_ref().unwrap().user.name, server_name).as_str());
}