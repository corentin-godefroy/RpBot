use mongodb::bson::{doc, Document, to_document};
use mongodb::Collection;
use serenity::client::Context;
use serenity::futures::stream::Collect;
use serenity::futures::StreamExt;
use serenity::model::application::command::{Command, CommandOptionType, CommandType};
use serenity::model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType};
use serenity::model::id::ChannelId;
use serenity::model::Permissions;
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, CommandDataOption};

use crate::constants::{ROAD_ID, COLLECTABLES, COLLECTABLES_DELAY, CREATE_ROAD_ALREADY_EXIST_MASSAGE, CREATE_ROAD_ALREADY_EXIST_TITLE, CREATE_ROAD_BAD_PLACE_INPUT_MESSAGE, CREATE_ROAD_BAD_PLACE_INPUT_TITLE, CREATE_ROAD_COMMAND_LOCALE_DESCRIPTION, CREATE_ROAD_COMMAND_NAME, CREATE_ROAD_DISTANCE_LOCALE_DESCRIPTION, CREATE_ROAD_DISTANCE_LOCALE_OPTION, CREATE_ROAD_ERROR_MESSAGE, CREATE_ROAD_ERROR_TITLE, CREATE_ROAD_MAXIMUM_HIT_MESSAGE1, CREATE_ROAD_MAXIMUM_HIT_MESSAGE2, CREATE_ROAD_MAXIMUM_HIT_TITLE, CREATE_ROAD_PLACE1_LOCALE_DESCRIPTION, CREATE_ROAD_PLACE1_LOCALE_OPTION, CREATE_ROAD_PLACE2_LOCALE_DESCRIPTION, CREATE_ROAD_PLACE2_LOCALE_OPTION, CREATE_ROAD_SECRET_LOCALE_DESCRIPTION, CREATE_ROAD_SECRET_LOCALE_OPTION, CREATE_ROAD_SUCCESS_MESSAGE, CREATE_ROAD_SUCCESS_TITLE, DEFAULT_LANG, DELAY_BEFORE_RECOLLECT, DISTANCE, PLACE_ID, PLACE_IN_DIFFERENT_UNIVERSE_MESSAGE, PLACE_IN_DIFFERENT_UNIVERSE_TITLE, PLACES_COLLECTION, PRIVILEGES, ROAD_CATEGORY_ID, ROAD_PLACE1, ROAD_PLACE2, ROADS_COLLECTION, ROLE_ID, RPBOT_BDD, SECRET_ROAD, SERVER_COLLECTION, SERVER_ID, UNIVERSE_ID};
use crate::lang::lang_loader::get_key;
use crate::{LANGS, MONGOCLIENT};
use crate::bdd::global::get_collection;
use crate::bdd::place::Place;
use crate::bdd::road::Road;
use crate::common_functions::ReportType::{ERROR, SUCCESS};
use crate::common_functions::{log, send_report_localized};
use crate::items::collectables::Collectables;

pub async fn create_road(ctx : &Context) {
    Command::create_global_application_command(&ctx, |command|{
        command.name(get_key(DEFAULT_LANG, CREATE_ROAD_COMMAND_NAME))
            .dm_permission(false)
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .description(get_key(DEFAULT_LANG, CREATE_ROAD_COMMAND_LOCALE_DESCRIPTION));

        for lang in LANGS.get().unwrap(){
            command.name_localized(lang.0, lang.1[CREATE_ROAD_COMMAND_NAME].to_string())
                .description_localized(lang.0, get_key(lang.0, CREATE_ROAD_COMMAND_LOCALE_DESCRIPTION));
        }

        command.kind(CommandType::ChatInput)
            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, CREATE_ROAD_PLACE1_LOCALE_OPTION))
                    .description(get_key(DEFAULT_LANG, CREATE_ROAD_PLACE1_LOCALE_DESCRIPTION))
                    .kind(CommandOptionType::String)
                    .required(true);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[CREATE_ROAD_PLACE1_LOCALE_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, CREATE_ROAD_PLACE1_LOCALE_DESCRIPTION));
                }
                option
            })

            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, CREATE_ROAD_PLACE2_LOCALE_OPTION))
                    .description(get_key(DEFAULT_LANG, CREATE_ROAD_PLACE2_LOCALE_DESCRIPTION))
                    .kind(CommandOptionType::String)
                    .required(true);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[CREATE_ROAD_PLACE2_LOCALE_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, CREATE_ROAD_PLACE2_LOCALE_DESCRIPTION));
                }
                option
            })

            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, CREATE_ROAD_DISTANCE_LOCALE_OPTION))
                    .description(get_key(DEFAULT_LANG, CREATE_ROAD_DISTANCE_LOCALE_DESCRIPTION))
                    .kind(CommandOptionType::Integer)
                    .required(true);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[CREATE_ROAD_DISTANCE_LOCALE_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, CREATE_ROAD_DISTANCE_LOCALE_DESCRIPTION));
                }
                option
            })

            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, CREATE_ROAD_SECRET_LOCALE_OPTION))
                    .description(get_key(DEFAULT_LANG, CREATE_ROAD_SECRET_LOCALE_DESCRIPTION))
                    .kind(CommandOptionType::Boolean)
                    .required(true);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[CREATE_ROAD_SECRET_LOCALE_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, CREATE_ROAD_SECRET_LOCALE_DESCRIPTION));
                }
                option
            })
    })
        .await
        .expect("Error on creation create_universe command.");
}

pub async fn create_road_reactor(ctx: &Context, aci: &ApplicationCommandInteraction){
    let options: Vec<CommandDataOption> = aci.data.options.to_vec();
    let place1_id: u64 = options.get(0).unwrap().value.as_ref().unwrap().as_str().unwrap().replace("\"", "").parse().unwrap();
    let place2_id: u64 = options.get(1).unwrap().value.as_ref().unwrap().as_str().unwrap().replace("\"", "").parse().unwrap();
    let distance = options.get(2).unwrap().value.as_ref().unwrap().as_u64().unwrap();
    let secret = options.get(3).unwrap().value.as_ref().unwrap().as_bool().unwrap();

    let client = MONGOCLIENT.get().expect("MONGOCLIENT not initialized").clone();
    let collection: Collection<Place> = client.database(RPBOT_BDD).collection::<Document>(PLACES_COLLECTION).clone_with_type();

    //Verification que les routes sont bien des places enregistrés
    let filter = doc! {
        "$or": [
            { PLACE_ID: place1_id as i64 },
            { PLACE_ID: place2_id as i64 }
        ]
    };

    let valid_categories_number = collection.find(filter, None).await.unwrap().count().await;
    if !((valid_categories_number == 2) || ((place1_id == place2_id) && valid_categories_number == 1)) {
        send_report_localized(&ctx, aci, ERROR, CREATE_ROAD_BAD_PLACE_INPUT_TITLE, CREATE_ROAD_BAD_PLACE_INPUT_MESSAGE, true).await.unwrap();
        return;
    }

    //Vérification de l'éxistance de la route
    let collection: Collection<Road> = client.database(RPBOT_BDD).collection::<Document>(ROADS_COLLECTION).clone_with_type();
    let filter = doc! {
        "$or": [
            {
                ROAD_PLACE1: place1_id as i64,
                ROAD_PLACE2: place2_id as i64
            },
            {
                ROAD_PLACE1: place2_id as i64,
                ROAD_PLACE2: place1_id as i64
            }
        ]
    };
    let existing_roads = collection.find(filter, None).await.unwrap().count().await;
    if existing_roads != 0 {
        send_report_localized(&ctx, aci, ERROR, CREATE_ROAD_ALREADY_EXIST_TITLE, CREATE_ROAD_ALREADY_EXIST_MASSAGE, true).await.unwrap();
        return;
    }

    //Récupération des routes existantes et verification de la limite à 25 (maximum de choix possible dans un select menu)
    let collection = client.database(RPBOT_BDD).collection::<Document>(ROADS_COLLECTION);
    let filter = doc! {
        "$or": [
            {"$or": [
                {ROAD_PLACE1: place1_id as i64},
                {ROAD_PLACE2: place1_id as i64}
            ]},
            {"$or": [
                {ROAD_PLACE1: place2_id as i64},
                {ROAD_PLACE2: place2_id as i64}
            ]}
        ]
    };
    let max_road = collection.find(filter, None).await.unwrap().count().await;
    if max_road >= 25 {
        send_report_localized(&ctx, aci, ERROR, CREATE_ROAD_MAXIMUM_HIT_TITLE, (CREATE_ROAD_MAXIMUM_HIT_MESSAGE1).as_ref(), true).await.unwrap();
        return;
    }

    //Recupération des guild des lieux d'arrivé pour les noms et vérification que les ID sont bien des lieux
    let collection : Collection<Place> = get_collection(PLACES_COLLECTION).clone_with_type();
    let filter = doc! {
        "$or": [
            {
                PLACE_ID: place1_id as i64
            },
            {
                PLACE_ID: place2_id as i64
            }
        ]
    };
    let places = collection.find(filter, None)
        .await
        .unwrap()
        .map(|result| result.expect("Failed to get result"))
        .collect::<Vec<Place>>()
        .await;

    let (place1, place2) = if places.len() == 2{
        (ctx.http.get_guild(places[0].server_id).await.unwrap().channels(&ctx.http).await.unwrap().get(&ChannelId(places[0].place_id)).unwrap().name.clone(),
         ctx.http.get_guild(places[1].server_id).await.unwrap().channels(&ctx.http).await.unwrap().get(&ChannelId(places[1].place_id)).unwrap().name.clone())
    }
        else {
            (ctx.http.get_guild(places[0].server_id).await.unwrap().channels(&ctx.http).await.unwrap().get(&ChannelId(places[0].place_id)).unwrap().name.clone(),
             ctx.http.get_guild(places[0].server_id).await.unwrap().channels(&ctx.http).await.unwrap().get(&ChannelId(places[0].place_id)).unwrap().name.clone())
        };

    if (places.len() == 2) && (places[0].universe_id != places[1].universe_id){
        //TODO erreur
        send_report_localized(
            &ctx, aci, ERROR,
            PLACE_IN_DIFFERENT_UNIVERSE_TITLE, PLACE_IN_DIFFERENT_UNIVERSE_MESSAGE, false)
            .await.unwrap()
        ;
        log(ERROR, "Les lieux n'appartiennent pas au même univers");
        println!("Erreur");
        return;
    }

    let name = place1.to_owned() + "-" + place2.as_ref();

    let collection = client.database(RPBOT_BDD).collection::<Document>(SERVER_COLLECTION);
    let server_info: Vec<Document> = collection.aggregate(
        vec![
            doc!{
                "$match": doc!{
                    SERVER_ID : *&aci.guild_id.unwrap().0 as i64
                }
            },
            doc!{
                "$project": doc!{
                    UNIVERSE_ID : 1,
                    ROAD_CATEGORY_ID : 1
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

    let universe_id = server_info.get(0).unwrap().get(UNIVERSE_ID).unwrap().as_object_id().unwrap().clone();
    let road_category_id = server_info.get(0).unwrap().get(ROAD_CATEGORY_ID).unwrap().as_i64().unwrap() as u64;
    let road_category = ctx.http.get_channel(road_category_id).await.unwrap().category().unwrap();

    let guild = ctx.http.get_guild(aci.guild_id.as_ref().unwrap().0).await.unwrap();

    let role = guild.create_role(&ctx.http, |role|{
        role.name(&name)
            .permissions(Permissions::empty())
    })
        .await
        .unwrap();

    let everyone_role = guild.role_by_name("@everyone").unwrap();

    let perms = vec![
        PermissionOverwrite{
            allow: Permissions::empty(),
            deny: Permissions::all(),
            kind: PermissionOverwriteType::Role(everyone_role.id),
        },
        PermissionOverwrite{
            allow: Permissions::VIEW_CHANNEL |
                Permissions::SEND_MESSAGES |
                Permissions::READ_MESSAGE_HISTORY |
                Permissions::USE_SLASH_COMMANDS,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Role(role.id),
        },
    ];

    let road_channel = guild.create_channel(&ctx.http, |channel|{
        channel.kind(ChannelType::Text)
            .name(&name)
            .permissions(perms)
            .category(road_category.id)
    })
        .await.unwrap();

    let mut road = Road::default();
    road.universe_id = universe_id;
    road.server_id = guild.id.0;
    road.road_id = road_channel.id.0;
    road.place_1 = place1_id;
    road.place_2 = place2_id;
    road.distance = distance;
    road.is_secret = secret;
    road.role_id = role.id.0;

    let collection = client.database(RPBOT_BDD).collection::<Document>(ROADS_COLLECTION);
    match collection.insert_one( to_document(&road).unwrap(), None).await {
        Ok(_) => { send_report_localized(&ctx, aci, SUCCESS, CREATE_ROAD_SUCCESS_TITLE, CREATE_ROAD_SUCCESS_MESSAGE, false).await.unwrap() }
        Err(_) => { send_report_localized(&ctx, aci, ERROR, CREATE_ROAD_ERROR_TITLE, CREATE_ROAD_ERROR_MESSAGE, false).await.unwrap() }
    };
    log(SUCCESS, format!("Nouvelle route {} créé avec succès pour le serveur {}", road_channel.name, guild.name).as_str())
}