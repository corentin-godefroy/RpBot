use std::io::{BufRead, Write};
use mongodb::bson::{doc, Document};
use serenity::builder::{CreateSelectMenu, CreateSelectMenuOption};
use serenity::client::Context;
use serenity::json::ToNumber;
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::prelude::application_command::{ApplicationCommandInteraction, CommandDataOption};
use serenity::model::prelude::interaction_trait::InteractionResponse;
use serenity::model::prelude::InteractionResponseType::ChannelMessageWithSource;
use crate::common_functions::ReportType::{ERROR, SUCCESS, WARNING};
use crate::common_functions::{get_guild_from_id, get_parent_category, get_parent_category_resolved, get_timestamp, get_universe_id, log, send_report, send_report_localized};
use crate::constants::{ROAD_ID, DEFAULT_CHANNEL_PLACE, DEFAULT_LANG, DISTANCE, MOUNT_SPEED_STAT, MOVE_COMMAND_LOCALE_DESCRIPTION, MOVE_COMMAND_LOCALE_NAME, MOVE_MODAL_CUSTOM_ID, MOVE_VALUE_CUSTOM_ID, PLACE_ID, PLACES_COLLECTION, PLAYER_COLLECTION, PLAYER_DESTINATION_ID, PLAYER_DESTINATION_SERVER_ID, PLAYER_END_TIMESTAMP, PLAYER_ID, PLAYER_CURRENT_POSITION_ID, PLAYER_START_TIMESTAMP, PLAYER_STATS, ROAD_PLACE1, ROAD_PLACE2, ROADS_COLLECTION, ROLE_ID, RPBOT_BDD, SECRET_ROAD, SERVER_ID, TIME_MODIFIER, SPEED_STAT, UNIVERSE_COLLECTION, UNIVERSE_ID, PLAYER_IS_IN_MOVE, PLAYER_POSITION_TIMESTAMP, PLAYER_NAME, LIGHT_BLUE_COLOR, MOVE_COMMAND_OPTION_LOCALE_DESCRIPTION, MOVE_COMMAND_OPTION_LOCALE_NAME, PLAYER_CURRENT_POSITION_SERVER_ID, PLAYER_PRIVILEGES, PRIVILEGES, OPTION_ON_ROAD_UNABLE_TITLE, OPTION_ON_ROAD_UNABLE_MESSAGE, MOVE_NOT_ALLOWED_HERE_TITLE, MOVE_NOT_ALLOWED_MESSAGE, PLACE_NOT_FOUND_TITLE, PLACE_NOT_FOUND_MESSAGE, ROADS_NOT_FOUND_TITLE, ROADS_NOT_FOUND_MESSAGE, CANT_RECOVER_ROADS_TITLE, CANT_RECOVER_ROADS_MESSAGE, GUILD_NOT_FOUND_TITLE, GUILD_NOT_FOUND_MESSAGE, ROAD_NOT_FOUND_TITLE, ROAD_NOT_FOUND_MESSAGE, UNIVERSE_NOT_FOUND_TITLE, UNIVERSE_NOT_FOUND_MESSAGE, MISSING_PRIVILEGE_TITLE, MISSING_PRIVILEGE_MESSAGE, MOVE_STARTED_TITLE, MOVE_STARTED_MESSAGE, MOVE_STOPPED_TITLE, MOVE_STOPPED_MESSAGE, SERVER_COLLECTION, PLACE_1, PLACE_2, STOP_MOVE, PLAYER_ALREADY_IN_MOVE_TITLE, PLAYER_ALREADY_IN_MOVE_MESSAGE};
use crate::lang::lang_loader::get_key;
use crate::{LANGS, MONGOCLIENT};
use crate::bdd::global::get_collection;
use crate::bdd::place::channel_is_place;
use crate::bdd::player::get_player_doc;
use crate::bdd::road::{channel_is_road, get_road_by_id, get_road_destinations, get_road_from_places, get_roads_from_place, Road};
use crate::players::player_moove::{add_player_move, set_move};

pub async fn setup_move_command(ctx : &Context){
    Command::create_global_application_command(&ctx, |command|{
        command.name(get_key(DEFAULT_LANG, MOVE_COMMAND_LOCALE_NAME))
            .dm_permission(false)
            .description(get_key(DEFAULT_LANG, MOVE_COMMAND_LOCALE_DESCRIPTION))
            .create_option(|option| {
                option.name(MOVE_COMMAND_OPTION_LOCALE_NAME)
                    .description(MOVE_COMMAND_OPTION_LOCALE_DESCRIPTION)
                    .kind(CommandOptionType::String)
                    .required(false);

                for lang in LANGS.get().unwrap(){
                    option.name_localized(
                        lang.0,
                        lang.1[MOVE_COMMAND_OPTION_LOCALE_NAME].to_string()
                    )
                        .description_localized(
                            lang.0,
                            get_key(lang.0, MOVE_COMMAND_OPTION_LOCALE_DESCRIPTION)
                        );
                }
                option
            });


        for lang in LANGS.get().unwrap(){
            command.name_localized(
                lang.0,
                lang.1[MOVE_COMMAND_LOCALE_NAME].to_string()
            )
                .description_localized(
                    lang.0,
                    get_key(lang.0, MOVE_COMMAND_LOCALE_DESCRIPTION)
                );
        }
        command
    })
        .await
        .expect("Error on creation create_new_player command.");
}

///Envoie un menu corrrespondant à sa situation pour faire le choix de la route à prendre ou de s'arreter.
pub async fn move_command_reactor(ctx : &Context, aci : &ApplicationCommandInteraction){
    //TODO
    //  Avec argument
    //      Si en route : erreur
    //      si en lieu :
    //          si route existante : mouvement
    //  Sans argument
    //      Si en route :
    //          Si à l'arret :
    //              choix 1 : demis tour
    //              choix 2 : continue
    //          si en mouvement :
    //              choix 1 : demis tour
    //              choix 2 : continue
    //              choix 3 : arret
    //      Si dans un lieu :
    //          choix multiples < 25 : routes possibles

    let args = aci.data.options.clone();
    let universe_id = get_universe_id(aci.guild_id.unwrap().0).await;
    let player = match get_player_doc(aci.user.id.0, &universe_id).await {
        Ok(doc) => {doc}
        Err(_) => {
            //TODO better error message
            send_report(&ctx, aci, ERROR, "Joueur non trouvé", "Le joueur n'a pas été trouvé dans la base de données.", true).await.expect("TODO: panic message");
            log(ERROR, "erreur, fiche personnage non trouvée"); return;
        }
    };

    if args.is_empty(){
        if player.is_in_move{
            //TODO
            move_command_without_option_in_move(&ctx, &aci).await;
        }
        else {
            //TODO (done ?)
            move_command_without_option_stopped(&ctx, &aci).await;
        }
    }
    else {
        if player.is_in_move{
            send_report(&ctx, aci, ERROR, PLAYER_ALREADY_IN_MOVE_TITLE, PLAYER_ALREADY_IN_MOVE_MESSAGE, true).await;
            log(WARNING, format!("Player {} try to move while already in move.", aci.user.id.0).as_str());
            return;
        }
        else {
            //TODO
            move_command_with_option(&ctx, &aci).await;
        }
    }
    log(SUCCESS, "");
}

//======================================================================================================================

async fn move_command_without_option_stopped(ctx: &Context, aci: &ApplicationCommandInteraction){
    //TODO :
    // 2 possibilitées :
    //  1 : Dans un lieu =>
    //    récupérer les routes disponibles
    //  2 : Sur une route
    //    récupérer les destinations disponibles
    if channel_is_place(ctx, aci.channel_id.0.clone()).await.is_some() {
        move_from_place(ctx, aci).await;
        //DONE
    }

    else if channel_is_road(aci.channel_id.0.clone()).await.is_some() {
        move_from_road(ctx, aci).await;
        //Done
    }

    else {
        send_report( &ctx, aci, ERROR, "", "", true);
        log(ERROR, "Try to move from a bad channel.");
        //TODO Make better error
    }
}

async fn move_from_place(ctx : &Context, aci: &ApplicationCommandInteraction) {
    //TODO : récupérer les routes disponibles GOOD
    //     : envoyer un menu avec les routes disponibles

    let catergory = get_parent_category_resolved(&ctx, aci.clone(), aci.guild_id.unwrap().0, aci.channel_id.0).await;
    let roads = get_roads_from_place(catergory.0).await;
    if roads.is_empty() {
        println!("No road found");
        //erreur
        //2 cas : pas de route OU route secrète
        //TODO envoyer un message d'erreur
    } else {
        //DONE
        let menu = create_move_menu(&ctx, aci, roads, None).await;
        send_menu(aci, ctx, "", menu).await;
    }
}

async fn move_from_road(ctx : &Context, aci : &ApplicationCommandInteraction){
    //Done
    println!("Move from road detected");
    let road_id = aci.channel_id.0;
    let road = get_road_by_id(road_id).await;
    match road {
        None => {//TODO message d'erreur
            println!("erreur, pas de route trouvée");
            return;
        }
        Some(doc) => {
            let destinations = get_road_destinations(road_id).await;
            let menu = create_menu_from_places(ctx, destinations, None).await;
            send_menu(aci, ctx, "", menu.to_owned()).await;
        }
    }
}

//======================================================================================================================

async fn move_command_without_option_in_move(ctx: &Context, aci: &ApplicationCommandInteraction){
    //TODO
    //arreter le mouvement + demis-tour
    let road_id = aci.channel_id.0;
    let road = get_road_by_id(road_id).await;
    match road {
        None => {//TODO message d'erreur
            println!("erreur, pas de route trouvée, en déplacement dans une ville ?!");
            send_report_localized(ctx, aci, ERROR, ROAD_NOT_FOUND_TITLE, ROAD_NOT_FOUND_MESSAGE, true).await;
            log(ERROR, format!("Road not found road_id {}", road_id).as_str());
            return;
        }
        Some(doc) => {
            let destinations = get_road_destinations(road_id).await;
            let option = vec![CreateSelectMenuOption::default().label("Stop").value(STOP_MOVE).clone()];
            let menu = create_menu_from_places(ctx, destinations, Some(option)).await;
            send_menu(aci, ctx, "", menu.to_owned()).await;
        }
    }
}

async fn move_command_with_option(ctx: &Context, aci: &ApplicationCommandInteraction){
    //TODO
    //verifier que le joueur est dans un lieu
    //verifier que la route existe
    let option : u64 = aci.data.options.get(0).unwrap().value.clone().unwrap().as_str().unwrap().parse().unwrap();
    let parent = get_parent_category(ctx, aci.guild_id.unwrap().0, aci.channel_id.0).await.unwrap();
    match get_road_from_places(parent.0, option).await {
        None => {
            send_report(ctx, aci, WARNING, "Route inexistante", "La route vers ce lieu n'existe pas.", true).await;
        }
        Some(road) => {
            let dest_1 = road.place_1;
            let dest_2 = road.place_2;
            let dest = if dest_1 == parent.0 {dest_2} else {dest_1};

            set_move(ctx, aci.guild_id.unwrap().0, aci.user.id.0, aci.channel_id.0, dest).await;
            //TODO : envoyer un message rp au départ et dans la route
        }
    };
}

//======================================================================================================================

async fn create_move_menu(ctx: &Context, aci : &ApplicationCommandInteraction, roads : Vec<Road>, options: Option<Vec<CreateSelectMenuOption>>) -> CreateSelectMenu{
    let parent_category_id = ctx.http.get_channel(aci.channel_id.0).await.unwrap().guild().unwrap().parent_id.unwrap().0;
    let mut menu = CreateSelectMenu::default();
    menu.custom_id(MOVE_MODAL_CUSTOM_ID);
    menu.placeholder("Choisissez une destination");
    menu.min_values(1);
    menu.max_values(1);

    let mut options = match options.is_none() {
        true => {vec![]}
        false => {options.unwrap()}
    };

    for road in roads{
        if road.is_secret {
            continue;
        }

        let id = if road.place_1 == parent_category_id { road.place_2.clone() } else { road.place_1.clone() };
        let road_name = ctx.http.get_channel(id).await.unwrap().category().unwrap().name;

        let mut option = CreateSelectMenuOption::default();
        options.insert(0, option.label(road_name).value(id).to_owned());
    };
    menu.options(|opt| {
        opt.set_options(options)
    });
    menu
}

async fn create_menu_from_places(ctx : &Context, destinations : Vec<Document>, options: Option<Vec<CreateSelectMenuOption>>) -> CreateSelectMenu{
    let mut options = match options {
        None => {Vec::new()}
        Some(_) => {options.unwrap()}
    };
    for place in destinations{
        let place_id = place.get(PLACE_ID).unwrap().as_i64().unwrap() as u64;
        let place_name = ctx.http.get_channel(place_id).await.unwrap().category().unwrap().name;

        let mut option = CreateSelectMenuOption::default();
        options.insert(0, option.label(place_name).value(place_id).to_owned());
    };

    let mut menu = CreateSelectMenu::default();
    menu.custom_id(MOVE_MODAL_CUSTOM_ID);
    menu.placeholder("Choisissez une destination".to_string());
    menu.min_values(1);
    menu.max_values(1);
    menu.options(|opt| {
        opt.set_options(options)
    });
    menu
}

async fn send_menu(aci : &ApplicationCommandInteraction, ctx : &Context, content : &str, menu : CreateSelectMenu){
    let result = aci.create_interaction_response(&ctx.http, |response|{
        response.kind(ChannelMessageWithSource);
        response.interaction_response_data(|message|{
            message.content(content);
            message.ephemeral(true);
            message.components(|components|{
                components.create_action_row(|action_row|{
                    action_row.add_select_menu(menu);
                    action_row
                });
                components
            });
            message
        });
        response
    }).await;

    match result {
        Ok(_) => {
            log(SUCCESS, "Menu de déplacement envoyé avec succès.");
        }
        Err(e) => {
            log(ERROR, "Le menu de déplacement n'as pas pu être envoyé.");
            println!("{}", e);
        }
    }
}

//======================================================================================================================

