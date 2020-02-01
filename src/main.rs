#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

mod config;
mod database;
mod error;
mod handler;
mod manager;
mod options;
mod server;
mod worker;

use crate::error::ApplicationError;
use crate::error::ApplicationResult;
use crate::options::Options;
use structopt::StructOpt;

fn main() -> ApplicationResult {
    env_logger::init();

    let options = Options::from_args();
    let config =
        config::load(options.config_path()).map_err(ApplicationError::read_config_error)?;

    config::validate(&config).map_err(ApplicationError::config_error)?;

    let dynamic_connections = manager::dynamic_connections(&config);

    if let Some(settings) = config.connections().dynamic_connections() {
        worker::start(settings, dynamic_connections.clone())
            .map_err(ApplicationError::update_connections_error)?;
    }

    server::start(&options, config, dynamic_connections)
}
