use std::collections::HashMap;
use mongodb::bson::{doc, Document, to_document};
use serenity::client::Context;
use serenity::futures::{StreamExt};
use serenity::model::application::command::{Command, CommandOptionType, CommandType};
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::channel::{ChannelType, PermissionOverwrite, PermissionOverwriteType};

use serenity::model::Permissions;
use crate::constants::{COLLECTABLES, COLLECTABLES_DELAY, CREATE_PLACE_COMMAND_LOCALE_DESCRIPTION, CREATE_PLACE_COMMAND_NAME, CREATE_PLACE_NAME_LOCALE_DESCRIPTION, CREATE_PLACE_NAME_LOCALE_OPTION, DEFAULT_CHANNEL_PLACE, DEFAULT_LANG, DELAY_BEFORE_RECOLLECT, NEW_PLACE_CREATED_MESSAGE, NEW_PLACE_CREATED_TITLE, PLACE_ID, PLACES_COLLECTION, ROLE_ID, RPBOT_BDD, SERVER_COLLECTION, SERVER_ID, UNIVERSE_ID};
use crate::lang::lang_loader::get_key;
use crate::{LANGS, MONGOCLIENT};
use crate::bdd::place::Place;
use crate::common_functions::{send_report_localized};
use crate::common_functions::ReportType::{SUCCESS};
use crate::items::collectables::Collectables;

pub async fn create_place(ctx : &Context) {
    Command::create_global_application_command(&ctx, |command|{
        command.name(get_key(DEFAULT_LANG, CREATE_PLACE_COMMAND_NAME))
            .dm_permission(false)
            .default_member_permissions(Permissions::ADMINISTRATOR)
            .description(get_key(DEFAULT_LANG, CREATE_PLACE_COMMAND_LOCALE_DESCRIPTION));

        for lang in LANGS.get().unwrap(){
            command.name_localized(lang.0, lang.1[CREATE_PLACE_COMMAND_NAME].to_string())
                .description_localized(lang.0, get_key(lang.0, CREATE_PLACE_COMMAND_LOCALE_DESCRIPTION));
        }

        command.kind(CommandType::ChatInput)
            .create_option(|option|{
                option.name(get_key(DEFAULT_LANG, CREATE_PLACE_NAME_LOCALE_OPTION))
                    .description(get_key(DEFAULT_LANG, CREATE_PLACE_NAME_LOCALE_DESCRIPTION))
                    .kind(CommandOptionType::String)
                    .required(true);

                for lang in LANGS.get().unwrap() {
                    option.name_localized(lang.0, lang.1[CREATE_PLACE_NAME_LOCALE_OPTION].to_string())
                        .set_autocomplete(true)
                        .description_localized(lang.0, get_key(lang.0, CREATE_PLACE_NAME_LOCALE_DESCRIPTION));
                }
                option
            })
    })
        .await
        .expect("Error on creation create_universe command.");
}

pub async fn create_place_reactor(ctx: &Context, aci: &ApplicationCommandInteraction){
    let guild = ctx.http.get_guild(aci.guild_id.as_ref().unwrap().0).await.unwrap();
    let name = &aci.data.options.get(0).clone().unwrap().value.clone().unwrap();

    //Vérification qu'un univers existe pour ce serveur et récupération de l'id de l'univers
    let client = MONGOCLIENT.get().expect("MONGOCLIENT not initialized").clone();

    let collection = client.database(RPBOT_BDD).collection::<Document>(SERVER_COLLECTION);
    let filter = doc! {
        SERVER_ID: guild.id.0 as i64
    };

    let server = collection.find(filter, None)
        .await
        .expect("error")
        .collect::<Vec<_>>()
        .await;

    if server.is_empty(){
        //TODO report
        return;
    }

    let universe_id = server.get(0).unwrap().clone().unwrap().get(UNIVERSE_ID).unwrap().as_object_id().unwrap();


    let role = guild.create_role(&ctx.http, |role|{
        role.name(name.as_str().unwrap())
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

    let category = guild.create_channel(&ctx.http, |category|{
        category.kind(ChannelType::Category)
            .name(name.as_str().unwrap())
            .permissions(perms)
    })
        .await.unwrap();

    let channel = guild.create_channel(&ctx.http, |channel|{
        channel.kind(ChannelType::Text)
            .name("rename-me-but-do-not-delete")
            .category(category.id)
    }).await.unwrap();

    let collection = client.database(RPBOT_BDD).collection::<Document>(PLACES_COLLECTION);

    let mut place = Place::default();
    place.server_id = guild.id.0;
    place.universe_id = universe_id.clone();
    place.role_id = role.id.0;
    place.place_id = category.id.0;
    place.default_channel_id = channel.id.0;

    collection.insert_one(to_document(&place).unwrap(), None).await.unwrap();

    send_report_localized(&ctx, aci, SUCCESS, NEW_PLACE_CREATED_TITLE, NEW_PLACE_CREATED_MESSAGE, false).await.unwrap();
}