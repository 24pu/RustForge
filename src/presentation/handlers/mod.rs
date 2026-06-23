pub mod auth;
mod content;
mod category;
mod media;
mod user;
mod role;
mod config;
mod health;
mod utils;
mod home;
mod theme_admin;
pub mod plugin_admin;
mod plugin_gateway;
pub mod sitemap;
pub mod plugin_settings;
pub mod install;
pub mod server_status;
pub mod lang_settings;
pub mod search;
pub mod product_admin;
pub mod amatemplate;

pub mod product_public;
pub mod product_export;
pub mod attribute;

pub use install::install_handler;

pub use auth::*;
pub use content::*;
pub use category::*;
pub use media::*;
pub use user::*;
pub use role::*;
pub use config::*;
pub use health::*;
pub use utils::{check_permission, get_config_value, get_nav_categories, generate_thumbnail,get_site_config_map};
pub use home::{home_handler, products_page_handler, product_detail_page_handler_by_slug};

pub use theme_admin::*;
pub use plugin_admin::*;
pub use plugin_gateway::plugin_gateway_handler;
pub use plugin_gateway::plugin_page_handler;
pub use plugin_gateway::plugin_static_handler;
pub use plugin_settings::get_public_plugin_settings;
pub use product_public::*;
pub use product_export::*;

pub use amatemplate::{
    list_amatemplates_handler,
    create_amatemplate_handler,
    get_amatemplate_handler,
    update_amatemplate_handler,
    delete_amatemplate_handler,
};