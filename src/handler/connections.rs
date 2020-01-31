use super::util::handle_empty;
use crate::config::ConfigRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IromRequest;
use iron::Response as IromResponse;

#[derive(Debug)]
pub struct ConnectionsHandler {
    config: ConfigRef,
}

impl ConnectionsHandler {
    pub fn new(config: ConfigRef) -> ConnectionsHandler {
        ConnectionsHandler { config }
    }
}

impl Handler for ConnectionsHandler {
    fn handle(&self, _req: &mut IromRequest) -> IronResult<IromResponse> {
        handle_empty(move || {
            let mut connections = Vec::new();

            for (index, connection) in self
                .config
                .connections()
                .static_connections()
                .iter()
                .enumerate()
            {
                let description = format!(
                    "{} ({}@{}/{})",
                    connection.description(),
                    connection.role(),
                    connection.host(),
                    connection.database()
                );
                let connection = Connection::new(index, &description);

                connections.push(connection);
            }

            Ok(connections)
        })
    }
}

#[derive(Debug, Clone, Serialize)]
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
