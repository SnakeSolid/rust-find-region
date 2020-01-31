use super::ConfigError;
use super::ConfigRef;
use super::ConfigResult;
use super::ConnectionSettings;

use std::path::Path;

#[allow(clippy::needless_pass_by_value)]
pub fn validate(config: ConfigRef) -> ConfigResult<()> {
    let query_schemas = config.query_schemas();

    if let Some(schema_name) = config
        .connections()
        .static_connections()
        .iter()
        .map(ConnectionSettings::query_schema)
        .filter(|&schema_name| !query_schemas.contains_key(schema_name))
        .next()
    {
        return Err(ConfigError::format(format_args!(
            "Query schema {} is not defined in query_schemas",
            schema_name,
        )));
    }

    if let Some(command) = config.connections().update_command() {
        validate_file(command, "connections.update_command")?;
    }

    Ok(())
}

fn validate_file<P>(path: P, name: &str) -> ConfigResult<()>
where
    P: AsRef<Path>,
{
    let path = path.as_ref();

    if !path.exists() {
        Err(ConfigError::format(format_args!(
            "{} directory ({}) is not exists",
            name,
            path.display(),
        )))
    } else if !path.is_file() {
        Err(ConfigError::format(format_args!(
            "{} directory ({}) is not a file",
            name,
            path.display()
        )))
    } else {
        Ok(())
    }
}
