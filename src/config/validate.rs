use super::Config;
use super::ConfigError;
use super::ConfigResult;
use super::ConnectionSettings;
use std::path::Path;

#[allow(clippy::needless_pass_by_value)]
pub fn validate(config: &Config) -> ConfigResult<()> {
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

    if let Some(dynamic_connections) = config.connections().dynamic_connections() {
        let interval = dynamic_connections.interval();
        let command = dynamic_connections.command();

        validate_number(interval, "dynamic_connections.interval")?;
        validate_file(command, "dynamic_connections.command")?;
    }

    Ok(())
}

fn validate_number(value: u64, name: &str) -> ConfigResult<()> {
    if value > 0 {
        Ok(())
    } else {
        Err(ConfigError::format(format_args!(
            "Value `{}` must be greater than zero, but {} given",
            name, value
        )))
    }
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
