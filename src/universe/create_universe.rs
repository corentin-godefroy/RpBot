use std::{future, io};
use std::collections::HashMap;
use std::io::Write;
use colored::Colorize;
use mongodb::bson::{Document};
use mongodb::bson::doc;
use serenity::client::Context;
use serenity::futures::{StreamExt};
use serenity::model::application::command::CommandOptionType;
use serenity::model::application::interaction::application_command::CommandDataOption;
use serenity::model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::Permissions;
use serenity::model::prelude::command::{Command, CommandType};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use tokio::join;
use crate::constants::{CREATE_UNIVERSE_CAMMAND_LOCALE_DESCRIPTION, CREATE_UNIVERSE_COMMAND_LOCALE_NAME, CREATE_UNIVERSE_NAME_LOCALE_OPTION, CREATE_UNIVERSE_NAME_OPTION_LOCALE_DESCRIPTION, DEFAULT_LANG, RPBOT_BDD, UNIVERSE_COLLECTION, SERVERS_UNIVERSE_FIELD, CREATE_UNIVERSE_ERROR_UNIVERSE_ALREADY_EXIST, UNIVERSE_NAME_FIELD, CREATE_UNIVERSE_SUCCESS, CREATE_UNIVERSE_SUCCESS_TITLE, CREATE_UNIVERSE_ERROR_ALREADY_EXIST_TITLE, UNIVERSE_ADMIN_ID, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION_DESCRIPTION, ADMIN_ROLE, MODO_ROLE, PLAYER_ROLE, PLAYER_ROLE_COLOR, SERVER_COLLECTION, SERVER_ID, ADMIN_ROLE_ID, MODERATOR_ROLE_ID, PLAYER_ROLE_ID, ADMIN_CATEGORY_NAME, NRP_CATEGORY_NAME, RP_CATEGORY_NAME, ROAD_CATEGORY_NAME, ROAD_CATEGORY_ID, RP_CATEGORY_ID, NRP_CATEGORY_ID, ADMIN_CATEGORY_ID, ADMIN_MODERATION_CHANNEL_NAME, ADMIN_COMMANDS_CHANNEL_NAME, NRP_GENERAL_CHANNEL_NAME, NRP_GENERAL_RULES_CHANNEL_NAME, RP_STORY_CHANNEL_NAME, RP_PLAYER_CHARACTERS_CHANNEL_NAME, RP_INDEX_CHANNEL_NAME, RP_RULES_CHANNEL_NAME, RP_QA_CHANNEL_NAME, NRP_RP_EXCHANGES_CHANNEL_NAME, GREEN_COLOR, SPECTATOR_ROLE, SPECTATOR_ROLE_ID, SPEED_MODIFIER_OPTION, SPEED_MODIFIER_OPTION_DESCRIPTION, SPEED_MODIFIER_VALUE_ERROR_TITLE, SPEED_MODIFIER_VALUE_ERROR, SPEED_MODIFIER};
use crate::lang::lang_loader::get_key;
use crate::{LANGS, MONGOCLIENT};
use crate::common_functions::{get_guild_from_aci, log, send_error_from_aci};
use crate::common_functions::LogType::SUCCESS;
use crate::universe::partial_setup::{player_role_setup, setup_road_category, setup_rp_category, setup_rp_channels};
use crate::universe::full_setup::specific_full_setup;
use crate::universe::sort::{sort_channels, sort_roles};

pub async fn create_universe(ctx : &Context) {
    Command::create_global_application_command(&ctx, |command|{
        command.name(get_key(DEFAULT_LANG, CREATE_UNIVERSE_COMMAND_LOCALE_NAME))
            .dm_permission(false)
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .description(get_key(DEFAULT_LANG, CREATE_UNIVERSE_CAMMAND_LOCALE_DESCRIPTION));

        for lang in LANGS.get().unwrap(){
            command.name_localized(lang.0, lang.1[CREATE_UNIVERSE_COMMAND_LOCALE_NAME].to_string())
                .description_localized(lang.0, get_key(lang.0, CREATE_UNIVERSE_CAMMAND_LOCALE_DESCRIPTION));
        }

        command.kind(CommandType::ChatInput)
            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, CREATE_UNIVERSE_NAME_LOCALE_OPTION))
                    .description(get_key(DEFAULT_LANG, CREATE_UNIVERSE_NAME_OPTION_LOCALE_DESCRIPTION))
                    .kind(CommandOptionType::String)
                    .required(true);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[CREATE_UNIVERSE_NAME_LOCALE_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, CREATE_UNIVERSE_NAME_OPTION_LOCALE_DESCRIPTION));
                }
                option
            })

            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION))
                    .description(get_key(DEFAULT_LANG, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION_DESCRIPTION))
                    .kind(CommandOptionType::Boolean)
                    .required(true);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[CREATE_UNIVERSE_PARTIAL_SETUP_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION_DESCRIPTION));
                }
                option
            })

            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, SPEED_MODIFIER_OPTION))
                    .description(get_key(DEFAULT_LANG, SPEED_MODIFIER_OPTION_DESCRIPTION))
                    .kind(CommandOptionType::Number)
                    .required(false);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[SPEED_MODIFIER_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, SPEED_MODIFIER_OPTION_DESCRIPTION));
                }
                option
            })
    })
        .await
        .expect("Error on creationg create_universe command.");
}

pub async fn create_universe_reactor(ctx: &Context, aci: &ApplicationCommandInteraction){
    let client = MONGOCLIENT.get().unwrap();
    let filter = doc!{
        SERVERS_UNIVERSE_FIELD: &aci.guild_id.unwrap().0.to_string()
    };
    let collection = client.database(RPBOT_BDD).collection::<Document>(UNIVERSE_COLLECTION);
    let results = collection.find(filter, None).await.unwrap();
    let mut docs: Vec<Document> = Vec::new();
    let _ = results.for_each(|doc| {
        docs.push(doc.unwrap());
        future::ready(())
    }).await;

    if !docs.is_empty(){
        send_error_from_aci(&ctx, &aci, CREATE_UNIVERSE_ERROR_ALREADY_EXIST_TITLE, CREATE_UNIVERSE_ERROR_UNIVERSE_ALREADY_EXIST).await;
        return;
    }

    let full_setup = aci.data.options.get(1).unwrap().value.as_ref().unwrap().as_bool().unwrap();
    //envoyer un message succès et proposer d'initialiser le serveur partiellement (uniquement catégorie route, section rp, hors rp) ou totalement (salons dans la section rp, et hors rp)
    //ATTENTION, prendre en compte dans le futur que le serveur n'est peut être pas initialisé totalement !
    let speed_modifier : f64 = match aci.data.options.get(2) {
        None => { 1.0 }
        Some(option) => {
            match option.value.as_ref().unwrap().as_f64(){
                None => { send_error_from_aci(&ctx, &aci, SPEED_MODIFIER_VALUE_ERROR_TITLE, SPEED_MODIFIER_VALUE_ERROR).await; return;}
                Some(speed_modifier) => {
                    if speed_modifier < 0.0{
                        send_error_from_aci(&ctx, &aci, SPEED_MODIFIER_VALUE_ERROR_TITLE, SPEED_MODIFIER_VALUE_ERROR).await;
                        return;
                    }
                    speed_modifier
                }
            }
        }
    };

    let guild = get_guild_from_aci(&ctx, &aci).await;
    let everyone = guild.role_by_name("@everyone").unwrap();

    aci.defer(&ctx.http).await.unwrap();

    let mut server_doc = doc! {
        SERVER_ID: &aci.guild_id.unwrap().0.to_string()
    };

    let player_role = player_role_setup(&ctx, &guild).await;
    server_doc.insert(PLAYER_ROLE_ID, player_role.id.0 as i64);

    if full_setup{
        let (admin_role, moderator_role, spectator_role, admin_category, nrp_category) =
            specific_full_setup(&ctx, &guild, &everyone, &player_role).await;
        server_doc.insert(ADMIN_ROLE_ID, admin_role.id.0 as i64);
        server_doc.insert(MODERATOR_ROLE_ID, moderator_role.id.0 as i64);
        server_doc.insert(SPECTATOR_ROLE_ID, spectator_role.id.0 as i64);
        server_doc.insert(ADMIN_CATEGORY_ID, admin_category.id.0 as i64);
        server_doc.insert(NRP_CATEGORY_ID, nrp_category.id.0 as i64);
    }

    let rp_category = setup_rp_category(&ctx, &guild, &everyone, &player_role);
    let road_category = setup_road_category(&ctx, &guild, &everyone, &player_role);
    let (rp_category, road_category) = join!(rp_category, road_category);

    let rp_channels = setup_rp_channels(&ctx, &aci, &guild, &rp_category, &everyone, &player_role).await;
    for (channel_id_field, channel) in rp_channels{
        server_doc.insert(channel_id_field, channel.id.0 as i64);
    }

    server_doc.insert(RP_CATEGORY_ID, rp_category.id.0 as i64);
    server_doc.insert(ROAD_CATEGORY_ID, road_category.id.0 as i64);


    //input dans la bdd
    let servers = client.database(RPBOT_BDD).collection::<Document>(SERVER_COLLECTION);

    match servers.insert_one(server_doc, None).await{
        Ok(_) => {}
        Err(_e) => {}
    }

    let universe_name = aci.data.options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap();

    let insert = doc!{
        UNIVERSE_NAME_FIELD: &universe_name,
        UNIVERSE_ADMIN_ID: aci.user.id.0.to_string(),
        SPEED_MODIFIER: speed_modifier,
        SERVERS_UNIVERSE_FIELD: vec![&aci.guild_id.unwrap().0.to_string()]
    };
    collection.insert_one(insert, None).await.unwrap();

    aci.edit_original_interaction_response(&ctx.http, |message|{
        message.embed(|embed|{
            embed.colour(GREEN_COLOR)
                .title(get_key(&aci.locale.as_str(), CREATE_UNIVERSE_SUCCESS_TITLE))
                .description(get_key(&aci.locale.as_str(), CREATE_UNIVERSE_SUCCESS))
        })
    }).await.unwrap();

    log(SUCCESS, format!("Universe \"{}\" for {} successfully added", universe_name.underline(),  aci.user.name.as_str().underline()).as_str());
    io::stdout().flush().unwrap();

    //tris des salons
    sort_channels(&ctx, &guild).await;

    //tris des rôles
    sort_roles(&ctx, &guild).await;
}