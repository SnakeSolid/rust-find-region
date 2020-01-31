use crate::config::ConfigRef;
use crate::error::ApplicationError;
use crate::error::ApplicationResult;
use crate::handler::ConnectionsHandler;
use crate::handler::FindRegionHandler;
use crate::manager::DynamicConnectionsRef;
use crate::options::Options;
use iron::Iron;
use mount::Mount;
use staticfile::Static;

pub fn start(
    options: &Options,
    config: ConfigRef,
    dynamic_connections: DynamicConnectionsRef,
) -> ApplicationResult {
    let mut mount = Mount::new();
    mount.mount(
        "/api/v1/connections",
        ConnectionsHandler::new(config.clone(), dynamic_connections.clone()),
    );
    mount.mount(
        "/api/v1/find_region",
        FindRegionHandler::new(config.clone(), dynamic_connections.clone()),
    );
    mount.mount("/static", Static::new("public/static"));
    mount.mount("/", Static::new("public"));

    let address = options.address();
    let port = options.port();

    println!("Listening on {}:{}...", address, port);

    Iron::new(mount)
        .http((address, port))
        .map(|_| ())
        .map_err(ApplicationError::server_error)
}
