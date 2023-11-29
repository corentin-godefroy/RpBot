use std::{ io};
use std::io::Write;
use std::ops::Deref;
use colored::Colorize;
use mongodb::bson::{Document, to_document};
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use serenity::client::Context;
use serenity::futures::{TryStreamExt};
use serenity::model::application::command::CommandOptionType;
use serenity::model::Permissions;
use serenity::model::prelude::command::{Command, CommandType};
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::prelude::interaction_trait::InteractionResponse;
use tokio::join;
use crate::constants::{CREATE_UNIVERSE_COMMAND_LOCALE_DESCRIPTION, CREATE_UNIVERSE_COMMAND_LOCALE_NAME, CREATE_UNIVERSE_NAME_LOCALE_OPTION, CREATE_UNIVERSE_NAME_OPTION_LOCALE_DESCRIPTION, DEFAULT_LANG, RPBOT_BDD, UNIVERSE_COLLECTION, CREATE_UNIVERSE_ERROR_UNIVERSE_ALREADY_EXIST, UNIVERSE_NAME_FIELD, CREATE_UNIVERSE_SUCCESS, CREATE_UNIVERSE_SUCCESS_TITLE, CREATE_UNIVERSE_ERROR_ALREADY_EXIST_TITLE, UNIVERSE_ADMIN_ID, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION, CREATE_UNIVERSE_PARTIAL_SETUP_OPTION_DESCRIPTION, SERVER_COLLECTION, SERVER_ID, ADMIN_ROLE_ID, MODERATOR_ROLE_ID, PLAYER_ROLE_ID, ROAD_CATEGORY_ID, RP_CATEGORY_ID, NRP_CATEGORY_ID, ADMIN_CATEGORY_ID, GREEN_COLOR, SPECTATOR_ROLE_ID, TIME_MODIFIER_OPTION, TIME_MODIFIER_OPTION_DESCRIPTION, TIME_MODIFIER_VALUE_ERROR_TITLE, TIME_MODIFIER_VALUE_ERROR, TIME_MODIFIER, UNIVERSE_ID, MAIN_SERVER, SPEED_STAT, STATS_COLLECTION, UNIVERSAL_STATS, SERVER_DOC_NOT_INSERT_TITLE, SERVER_DOC_NOT_INSERT, MOUNT_SPEED_STAT, UNIVERSE_NOT_FOUND_TITLE, UNIVERSE_NOT_FOUND_MESSAGE, UNIVERSE_DEFAULT_LOCALE};
use crate::lang::lang_loader::get_key;
use crate::{LANGS, MONGOCLIENT};
use crate::bdd::server::Server;
use crate::bdd::stats::{UniverseStats, Stats};
use crate::bdd::universe::Universe;
use crate::common_functions::{get_guild_from_aci, log, send_report_localized};
use crate::common_functions::ReportType::{ERROR, SUCCESS};
use crate::universe::partial_setup::{partial_setup, player_role_setup, setup_road, setup_rp};
use crate::universe::full_setup::{full_setup};
use crate::universe::sort::{sort_channels, sort_roles};

pub async fn create_universe(ctx : &Context) {
    Command::create_global_application_command(&ctx, |command|{
        command.name(get_key(DEFAULT_LANG, CREATE_UNIVERSE_COMMAND_LOCALE_NAME))
            .dm_permission(false)
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .description(get_key(DEFAULT_LANG, CREATE_UNIVERSE_COMMAND_LOCALE_DESCRIPTION));

        for lang in LANGS.get().unwrap(){
            command.name_localized(lang.0, lang.1[CREATE_UNIVERSE_COMMAND_LOCALE_NAME].to_string())
                .description_localized(lang.0, get_key(lang.0, CREATE_UNIVERSE_COMMAND_LOCALE_DESCRIPTION));
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
                option.name(get_key(DEFAULT_LANG, TIME_MODIFIER_OPTION))
                    .description(get_key(DEFAULT_LANG, TIME_MODIFIER_OPTION_DESCRIPTION))
                    .kind(CommandOptionType::Number)
                    .required(false);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[TIME_MODIFIER_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, TIME_MODIFIER_OPTION_DESCRIPTION));
                }
                option
            })
    })
        .await
        .expect("Error on creation create_universe command.");
}

pub async fn create_universe_reactor(ctx: &Context, aci: &ApplicationCommandInteraction){
    let client = MONGOCLIENT.get().unwrap();
    let filter = doc!{
        SERVER_ID: *&aci.guild_id.unwrap().0 as i64
    };

    let collection = client.database(RPBOT_BDD).collection::<Server>(SERVER_COLLECTION);
    let cursor = collection.find(filter, None).await.unwrap();
    let docs: Result<Vec<Server>, _> = cursor.try_collect().await;
    let docs = docs.unwrap();


    if !docs.is_empty(){
        send_report_localized(&ctx, aci.deref(), ERROR, CREATE_UNIVERSE_ERROR_ALREADY_EXIST_TITLE, CREATE_UNIVERSE_ERROR_UNIVERSE_ALREADY_EXIST, false).await.unwrap();
        return;
    }

    let is_full_setup = aci.data.options.get(1).unwrap().value.as_ref().unwrap().as_bool().unwrap();
    //envoyer un message succès et proposer d'initialiser le serveur partiellement (uniquement catégorie route, section rp, hors rp) ou totalement (salons dans la section rp, et hors rp)
    //ATTENTION, prendre en compte dans le futur que le serveur n'est peut être pas initialisé totalement !
    let time_modifier: u64 = match aci.data.options.get(2) {
        None => { 60 }
        Some(option) => {
            match option.value.as_ref().unwrap().as_u64(){
                None => { send_report_localized(&ctx, aci.deref(), ERROR, TIME_MODIFIER_VALUE_ERROR_TITLE, TIME_MODIFIER_VALUE_ERROR, false).await.unwrap(); return;}
                Some(speed_modifier) => { speed_modifier }
            }
        }
    };

    let collection = client.database(RPBOT_BDD).collection::<Universe>(UNIVERSE_COLLECTION);
    let universe_name = aci.data.options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap();
    let mut universe = Universe::default();
    universe.name = universe_name.to_string();
    universe.time_modifier = time_modifier;
    universe.creator = aci.user.id.0;
    universe.default_locale = aci.locale.clone();

    let universe = collection.insert_one(universe, None).await.unwrap();


    let guild = get_guild_from_aci(&ctx, &aci).await;

    aci.defer(&ctx.http).await.unwrap();

    let mut server = Server::default();
    server.server_id = aci.guild_id.unwrap().0;

    if is_full_setup{
        full_setup(&ctx, &aci, &guild, &mut server).await;
    }
    else {
        partial_setup(&ctx, &aci, &guild, &mut server).await;
    }

    server.is_main_server = true;
    server.universe_id = universe.inserted_id.clone().as_object_id().unwrap().clone();

    let mut speed_stat = Stats::default();
    speed_stat.name = SPEED_STAT.to_string();
    speed_stat.base_value = 1.0;
    speed_stat.hide = false;

    let mut mount_speed_stat = Stats::default();
    mount_speed_stat.name = MOUNT_SPEED_STAT.to_string();
    mount_speed_stat.base_value = 0.0;
    mount_speed_stat.hide = true;

    let mut server_stat = UniverseStats::default();
    server_stat.universe_id = universe.inserted_id.clone().as_object_id().unwrap();
    server_stat.universal_stats = vec![speed_stat.clone(), mount_speed_stat.clone()];
    let collection = client.database(RPBOT_BDD).collection::<Document>(STATS_COLLECTION);
    collection.insert_one(server_stat.to_doc(), None).await.unwrap();


    //input dans la bdd
    let servers = client.database(RPBOT_BDD).collection::<Document>(SERVER_COLLECTION);

    match servers.insert_one(server.to_doc(), None).await{
        Ok(_) => {}
        Err(_e) => {
            send_report_localized(
                &ctx,aci, ERROR,
                SERVER_DOC_NOT_INSERT_TITLE, SERVER_DOC_NOT_INSERT, true)
                .await.unwrap();
            return;
        }
    }


    aci.edit_original_interaction_response(&ctx.http, |message|{
        message.embed(|embed|{
            embed.colour(GREEN_COLOR)
                .title(get_key(&aci.locale.as_str(), CREATE_UNIVERSE_SUCCESS_TITLE))
                .description(get_key(&aci.locale.as_str(), CREATE_UNIVERSE_SUCCESS))
        })
    }).await.unwrap();

    log(SUCCESS, format!("Universe \"{}\" for {} successfully added", universe_name.underline(),  aci.user.name.as_str().underline()).as_str());
    io::stdout().flush().unwrap();

    //tris des salons et des roles
    let channel_sort = sort_channels(&ctx, &guild);
    let role_sort = sort_roles(&ctx, &guild);
    join!(channel_sort, role_sort);
}



async fn get_universe_id(guild_id : u64) -> Option<ObjectId> {
    let client = MONGOCLIENT.get().unwrap();
    let collection : Collection<Document> = client.database(RPBOT_BDD).collection(SERVER_COLLECTION);

    return match collection.find_one(doc!{SERVER_ID : guild_id as i64}, None).await {
        Ok(doc) => { Some(doc.unwrap().get(UNIVERSE_ID).unwrap().as_object_id().unwrap()) }
        Err(_) => { log(ERROR, format!("No universe found for guild {}", guild_id).as_str());
            None
        }
    }
}

pub async fn get_universe_id_resolved<R : InteractionResponse + std::marker::Sync>(ctx : &Context, interaction : R, guild_id : u64) -> ObjectId{
    match get_universe_id(guild_id).await{
        Some(id) => {id}
        None => {
            send_report_localized(
                &ctx, &interaction, ERROR,
                UNIVERSE_NOT_FOUND_TITLE, UNIVERSE_NOT_FOUND_MESSAGE, true
            ).await.unwrap();
            panic!("Universe not found");
        }
    }
}