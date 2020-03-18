use super::util::handle_empty;
use crate::config::ConfigRef;
use crate::config::ConnectionSettings;
use crate::manager::DynamicConnectionsRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IromRequest;
use iron::Response as IromResponse;

#[derive(Debug)]
pub struct ConnectionsHandler {
    config: ConfigRef,
    dynamic_connections: DynamicConnectionsRef,
}

impl ConnectionsHandler {
    pub fn new(
        config: ConfigRef,
        dynamic_connections: DynamicConnectionsRef,
    ) -> ConnectionsHandler {
        ConnectionsHandler {
            config,
            dynamic_connections,
        }
    }
}

impl Handler for ConnectionsHandler {
    fn handle(&self, _req: &mut IromRequest) -> IronResult<IromResponse> {
        handle_empty(move || {
            let mut connections: Vec<Connection> = self
                .config
                .connections()
                .static_connections()
                .iter()
                .enumerate()
                .map(|connection| connection.into())
                .collect();
            let _ = self
                .dynamic_connections
                .for_each(|index, connection| connections.push((index, connection).into()));

            connections.sort_by(|a, b| a.description.cmp(&b.description));

            Ok(connections)
        })
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct Connection {
    index: usize,
    description: String,
}

impl Connection {
    fn new(index: usize, description: &str) -> Connection {
        Connection {
            index,
            description: description.into(),
        }
    }
}

impl From<(usize, &ConnectionSettings)> for Connection {
    fn from(value: (usize, &ConnectionSettings)) -> Connection {
        let index = value.0;
        let connection = value.1;
        let description = format!(
            "{} ({}@{}/{})",
            connection.description(),
            connection.role(),
            connection.host(),
            connection.database()
        );

        Connection::new(index, &description)
    }
}
