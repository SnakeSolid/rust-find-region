mod error;
mod validate;

pub use self::error::ConfigError;
pub use self::error::ConfigResult;
pub use self::validate::validate;
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::Arc;

pub type ConfigRef = Arc<Config>;

#[derive(Debug, Deserialize)]
pub struct Config {
    connections: ConnectionsSettings,
    query_schemas: HashMap<String, QuerySchemaSettings>,
}

impl Config {
    pub fn connections(&self) -> &ConnectionsSettings {
        &self.connections
    }

    pub fn query_schemas(&self) -> &HashMap<String, QuerySchemaSettings> {
        &self.query_schemas
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionsSettings {
    update_command: Option<String>,
    static_connections: Vec<ConnectionSettings>,
}

impl ConnectionsSettings {
    pub fn update_command(&self) -> Option<&String> {
        self.update_command.as_ref()
    }

    pub fn static_connections(&self) -> &[ConnectionSettings] {
        &self.static_connections
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionSettings {
    description: String,
    query_schema: String,
    host: String,
    database: String,
    port: u16,
    role: String,
    password: Option<String>,
}

impl ConnectionSettings {
    pub fn description(&self) -> &str {
        &self.host
    }

    pub fn query_schema(&self) -> &str {
        &self.query_schema
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn database(&self) -> &str {
        &self.database
    }

    pub fn role(&self) -> &str {
        &self.role
    }

    pub fn password(&self) -> Option<&String> {
        self.password.as_ref()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuerySchemaSettings {
    region_by_id: String,
    regions_by_name: String,
    hierarchy_by_id: String,
}

impl QuerySchemaSettings {
    pub fn region_by_id(&self) -> &str {
        &self.region_by_id
    }

    pub fn regions_by_name(&self) -> &str {
        &self.regions_by_name
    }

    pub fn hierarchy_by_id(&self) -> &str {
        &self.hierarchy_by_id
    }
}

pub fn load<P>(path: P) -> ConfigResult<ConfigRef>
where
    P: AsRef<Path>,
{
    let reader = File::open(path).map_err(ConfigError::io_error)?;
    let config = serde_yaml::from_reader(reader).map_err(ConfigError::yaml_error)?;

    Ok(Arc::new(config))
}
