pub const DEFAULT_LANG: &str = "en-US";
pub const FR_LANG: &str = "fr";

pub const RED_COLOR: u64 = 0xFF0000;
pub const GREEN_COLOR: u64 = 0x00FF00;
pub const BLUE_COLOR: u64 = 0x0000FF;
pub const LIGHT_BLUE_COLOR: u64 = 0x04EEE6;

//role colors
pub const ADMIN_ROLE_COLOR: u64 = 0x77FFFF;
pub const MODO_ROLE_COLOR: u64 = 0xFFA836;
pub const PLAYER_ROLE_COLOR: u64 = 0x37FF66;
pub const SPECTATOR_ROLE_COLOR: u64 = 0xFFF666;

//role names
pub const ADMIN_ROLE: &str = "Admin";
pub const MODO_ROLE: &str = "Modo";
pub const SPECTATOR_ROLE: &str = "Spectator";
pub const PLAYER_ROLE: &str = "Player";

//channel names
pub const ADMIN_CATEGORY_NAME: &str = "Admin";
pub const RP_CATEGORY_NAME: &str = "RP";
pub const NRP_CATEGORY_NAME: &str = "NO RP";
pub const ROAD_CATEGORY_NAME: &str = "Roads";

pub const ADMIN_MODERATION_CHANNEL_NAME: &str = "moderation";
pub const ADMIN_COMMANDS_CHANNEL_NAME: &str = "commands";

pub const NRP_GENERAL_CHANNEL_NAME: &str = "general";
pub const NRP_GENERAL_VOICE_CHANNEL_NAME: &str = "general";
pub const NRP_GENERAL_RULES_CHANNEL_NAME: &str = "rules";
pub const NRP_RP_EXCHANGES_CHANNEL_NAME: &str = "rp-exchanges";

pub const RP_STORY_CHANNEL_NAME: &str = "story";
pub const RP_PLAYER_CHARACTERS_CHANNEL_NAME: &str = "player-characters";
pub const RP_INDEX_CHANNEL_NAME: &str = "index";
pub const RP_RULES_CHANNEL_NAME: &str = "rules";
pub const RP_QA_CHANNEL_NAME: &str = "questions-answers";

//command default names
pub const CREATE_UNIVERSE_COMMAND_NAME: &str = "create_new_universe";


//commands consts
pub const CREATE_UNIVERSE_COMMAND_LOCALE_NAME: &str = "create_universe_command_name";
pub const CREATE_UNIVERSE_CAMMAND_LOCALE_DESCRIPTION: &str = "create_universe_command_description";
pub const CREATE_UNIVERSE_NAME_LOCALE_OPTION: &str = "create_universe_name_option";
pub const CREATE_UNIVERSE_NAME_OPTION_LOCALE_DESCRIPTION: &str = "create_universe_name_option";
pub const CREATE_UNIVERSE_PARTIAL_SETUP_OPTION: &str = "create_universe_partial_setup_option";
pub const CREATE_UNIVERSE_PARTIAL_SETUP_OPTION_DESCRIPTION: &str = "create_universe_partial_setup_option_description";
pub const CREATE_UNIVERSE_ERROR_ALREADY_EXIST_TITLE: &str = "universe_already_exist_title";
pub const CREATE_UNIVERSE_ERROR_UNIVERSE_ALREADY_EXIST: &str = "universe_already_exist";
pub const CREATE_UNIVERSE_SUCCESS_TITLE: &str = "universe_success_title";
pub const CREATE_UNIVERSE_SUCCESS: &str = "universe_success_setup";

pub const SPEED_MODIFIER_OPTION: &str = "speed_modifier_option";
pub const SPEED_MODIFIER_OPTION_DESCRIPTION: &str = "speed_modifier_option_description";
pub const SPEED_MODIFIER_VALUE_ERROR_TITLE: &str = "speed_modifier_value_error_title";
pub const SPEED_MODIFIER_VALUE_ERROR: &str = "speed_modifier_value_error";

//bdd collection names
pub const RPBOT_BDD: &str = "RpBot";
pub const UNIVERSE_COLLECTION: &str = "universe";
pub const SERVER_COLLECTION: &str = "servers";
pub const PLAYER_COLLECTION: &str = "players";
pub const MOUNTS_COLLECTION: &str = "mounts";
pub const PRIVATE_PROPERTIES_COLLECTION: &str = "private_properties";

//universe doc fields
pub const UNIVERSE_NAME_FIELD: &str = "name";
pub const SERVERS_UNIVERSE_FIELD: &str = "servers";
pub const UNIVERSE_ADMIN_ID: &str = "creator";

//Server doc fields
pub const SERVER_ID: &str = "server_id";
pub const ADMIN_ROLE_ID: &str = "admin_role_id";
pub const MODERATOR_ROLE_ID: &str = "moderator_role_id";
pub const SPECTATOR_ROLE_ID: &str = "spectator_role_id";
pub const PLAYER_ROLE_ID: &str = "player_role_id";
pub const NRP_CATEGORY_ID: &str = "nrp_category_id";
pub const RP_CATEGORY_ID: &str = "rp_category_id";
pub const ADMIN_CATEGORY_ID: &str = "admin_category_id";
pub const ROAD_CATEGORY_ID: &str = "road_category_id";

pub const RP_STORY_CHANNEL_ID: &str = "story_id";
pub const RP_PLAYER_CHARACTERS_CHANNEL_ID: &str = "player_characters_id";
pub const RP_INDEX_CHANNEL_ID: &str = "index_id";
pub const RP_RULES_CHANNEL_ID: &str = "rules_id";
pub const RP_QA_CHANNEL_ID: &str = "questions_answers_id";

pub const SPEED_MODIFIER: &str = "speed_modifier";
