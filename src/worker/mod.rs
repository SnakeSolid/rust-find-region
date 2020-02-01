mod error;

pub use self::error::UpdateConnectionsError;
pub use self::error::UpdateConnectionsResult;

use crate::config;
use crate::config::ConnectionSettings;
use crate::config::DynamicConnectionsSettings;
use crate::manager::DynamicConnectionsRef;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::thread::Builder;
use std::time::Duration;

#[derive(Debug)]
pub struct UpdateConnectionsWorker {
    interval: Duration,
    command: String,
    dynamic_connections: DynamicConnectionsRef,
}

impl UpdateConnectionsWorker {
    fn new(
        config: &DynamicConnectionsSettings,
        dynamic_connections: DynamicConnectionsRef,
    ) -> UpdateConnectionsWorker {
        UpdateConnectionsWorker {
            interval: Duration::from_secs(config.interval()),
            command: config.command().into(),
            dynamic_connections,
        }
    }

    fn run(self) {
        info!("Update connections thread started.");

        loop {
            match self.update_connections() {
                Ok(()) => {}
                Err(error) => warn!("Failed to update dynamic connections - {}", error),
            }

            thread::sleep(self.interval);
        }
    }

    fn update_connections(&self) -> UpdateConnectionsResult<()> {
        let mut child = Command::new(&self.command)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(UpdateConnectionsError::spawn_command_error)?;
        let connections = child
            .stdout
            .take()
            .map(|stdout| self.read_connections(stdout));

        child
            .wait()
            .map_err(UpdateConnectionsError::wait_command_error)?;

        if let Some(connections) = connections {
            self.write_connections(connections?)?;
        }

        Ok(())
    }

    fn write_connections(
        &self,
        connections: Vec<ConnectionSettings>,
    ) -> UpdateConnectionsResult<()> {
        let mut current_connections = HashMap::new();

        self.dynamic_connections
            .for_each(|index, connection| {
                current_connections.insert(self.connection_string(connection), index);
            })
            .map_err(UpdateConnectionsError::update_connections_error)?;

        debug!("current dynamic connections: {:?}", current_connections);

        for connection in connections {
            let string = self.connection_string(&connection);

            if let Some(index) = current_connections.remove(&string) {
                self.dynamic_connections
                    .update(index, connection)
                    .map_err(UpdateConnectionsError::update_connections_error)?;
            } else {
                self.dynamic_connections
                    .insert(connection)
                    .map_err(UpdateConnectionsError::update_connections_error)?;
            }
        }

        for &index in current_connections.values() {
            self.dynamic_connections
                .remove(index)
                .map_err(UpdateConnectionsError::update_connections_error)?;
        }

        Ok(())
    }

    fn connection_string(&self, connection: &ConnectionSettings) -> String {
        match (
            connection.role(),
            connection.host(),
            connection.port(),
            connection.database(),
        ) {
            (role, host, Some(port), database) => {
                format!("{}@{}:{}/{}", role, host, port, database)
            }
            (role, host, None, database) => format!("{}@{}/{}", role, host, database),
        }
    }

    fn read_connections<R>(&self, read: R) -> UpdateConnectionsResult<Vec<ConnectionSettings>>
    where
        R: Read,
    {
        let reader = BufReader::new(read);
        let mut connections = Vec::new();
        let mut builder = ConnectionBuilder::new();

        for line in reader.lines() {
            let line = line.map_err(UpdateConnectionsError::read_output_error)?;
            let mut parts = line.splitn(2, ':').map(|value| value.trim());

            match (parts.next(), parts.next()) {
                (Some(key), Some(value)) if key == "description" => builder.set_description(value),
                (Some(key), Some(value)) if key == "query schema" => {
                    builder.set_query_schema(value)
                }
                (Some(key), Some(value)) if key == "host" => builder.set_host(value),
                (Some(key), Some(value)) if key == "port" => builder.set_port(value),
                (Some(key), Some(value)) if key == "database" => builder.set_database(value),
                (Some(key), Some(value)) if key == "role" => builder.set_role(value),
                (Some(key), Some(value)) if key == "password" => builder.set_password(value),
                (Some(key), _) if key == "~~~" => {
                    if let Some(connection) = builder.build() {
                        connections.push(connection);
                    }

                    builder = ConnectionBuilder::new();
                }
                (Some(key), _) => warn!("Update command output contains invalid key `{}`", key),
                (None, _) => warn!("Update command output contains empty line"),
            }
        }

        Ok(connections)
    }
}

#[derive(Debug)]
struct ConnectionBuilder {
    description: Option<String>,
    query_schema: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    database: Option<String>,
    role: Option<String>,
    password: Option<String>,
}

impl ConnectionBuilder {
    fn new() -> ConnectionBuilder {
        ConnectionBuilder {
            description: None,
            query_schema: None,
            host: None,
            port: None,
            database: None,
            role: None,
            password: None,
        }
    }

    fn set_description(&mut self, value: &str) {
        self.description = Some(value.into());
    }

    fn set_query_schema(&mut self, value: &str) {
        self.query_schema = Some(value.into());
    }

    fn set_host(&mut self, value: &str) {
        self.host = Some(value.into());
    }

    fn set_port(&mut self, value: &str) {
        match value.parse() {
            Ok(value) => self.port = Some(value),
            Err(err) => warn!("Failed to parse {} as port number - {}", value, err),
        }
    }

    fn set_database(&mut self, value: &str) {
        self.database = Some(value.into());
    }

    fn set_role(&mut self, value: &str) {
        self.role = Some(value.into());
    }

    fn set_password(&mut self, value: &str) {
        self.password = Some(value.into());
    }

    fn build(self) -> Option<ConnectionSettings> {
        match (
            self.description,
            self.query_schema,
            self.host,
            self.port,
            self.database,
            self.role,
            self.password,
        ) {
            (
                Some(description),
                Some(query_schema),
                Some(host),
                port,
                Some(database),
                Some(role),
                password,
            ) => Some(config::connection_settings(
                &description,
                &query_schema,
                &host,
                port,
                &database,
                &role,
                password.as_ref(),
            )),
            (None, _, _, _, _, _, _) => {
                warn!("Failed to reader connection - missing description");
                None
            }
            (_, None, _, _, _, _, _) => {
                warn!("Failed to reader connection - missing query schema");
                None
            }
            (_, _, None, _, _, _, _) => {
                warn!("Failed to reader connection - missing host");
                None
            }
            (_, _, _, _, None, _, _) => {
                warn!("Failed to reader connection - missing database");
                None
            }
            (_, _, _, _, _, None, _) => {
                warn!("Failed to reader connection - missing role");
                None
            }
        }
    }
}

pub fn start(
    config: &DynamicConnectionsSettings,
    dynamic_connections: DynamicConnectionsRef,
) -> UpdateConnectionsResult<()> {
    let worker = UpdateConnectionsWorker::new(config, dynamic_connections);

    Builder::new()
        .name("dynamic connections updater".into())
        .spawn(move || worker.run())
        .map_err(UpdateConnectionsError::start_thread_error)?;

    Ok(())
}
