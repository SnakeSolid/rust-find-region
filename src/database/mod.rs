mod error;

pub use self::error::DatabaseError;
pub use self::error::DatabaseResult;

use crate::config::ConnectionSettings;
use crate::config::QuerySchemaSettings;
use postgres::config::Config;
use postgres::row::Row;
use postgres::Client;
use postgres::NoTls;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;

#[derive(Debug)]
pub struct Database<'a> {
    settings: ConnectionSettings,
    query_schema: &'a QuerySchemaSettings,
}

impl<'a> Database<'a> {
    pub fn new(settings: ConnectionSettings, query_schema: &QuerySchemaSettings) -> Database {
        Database {
            settings,
            query_schema,
        }
    }

    pub fn connect(self) -> DatabaseResult<DatabaseClient<'a>> {
        let mut config = Config::new();
        config.host(&self.settings.host());
        config.port(self.settings.port());
        config.dbname(&self.settings.database());
        config.user(&self.settings.role());

        if let Some(ref password) = self.settings.password() {
            config.password(password);
        }

        config.connect_timeout(Duration::from_secs(10));

        let client = config
            .connect(NoTls)
            .map_err(DatabaseError::connection_error)?;

        Ok(DatabaseClient::new(self.query_schema, client))
    }
}

pub struct DatabaseClient<'a> {
    query_schema: &'a QuerySchemaSettings,
    client: Client,
}

impl<'a> DatabaseClient<'a> {
    fn new(query_schema: &QuerySchemaSettings, client: Client) -> DatabaseClient {
        DatabaseClient {
            query_schema,
            client,
        }
    }

    pub fn regions_by_id<I>(&mut self, it: I) -> DatabaseResult<HashMap<i64, Region>>
    where
        I: IntoIterator<Item = i64>,
    {
        let ids: Vec<i64> = it.into_iter().collect();

        debug!("Get region names by id: ids = {:?}", ids);

        let rows = self
            .client
            .query(self.query_schema.regions_by_id(), &[&ids])
            .map_err(DatabaseError::query_execution_error)?;

        self.collect_regions(rows)
    }

    pub fn regions_by_name(&mut self, name: &str) -> DatabaseResult<HashMap<i64, Region>> {
        debug!("Get regions by name: name = {}", name);

        let rows = self
            .client
            .query(self.query_schema.regions_by_name(), &[&name])
            .map_err(DatabaseError::query_execution_error)?;

        self.collect_regions(rows)
    }

    #[inline]
    fn collect_regions(&self, result: Vec<Row>) -> DatabaseResult<HashMap<i64, Region>> {
        let mut builders = HashMap::new();

        for row in result {
            let id: i64 = row.try_get(0).map_err(DatabaseError::value_error)?;
            let language: String = row.try_get(1).map_err(DatabaseError::value_error)?;
            let name: String = row.try_get(2).map_err(DatabaseError::value_error)?;
            let is_default: bool = row.try_get(3).map_err(DatabaseError::value_error)?;
            let builder = builders.entry(id).or_insert_with(RegionBuilder::new);

            builder.insert_name(language, name, is_default);
        }

        Ok(builders
            .into_iter()
            .map(|(id, builder)| (id, builder.build()))
            .collect())
    }

    pub fn hierarchy_by_id<I>(&mut self, it: I) -> DatabaseResult<Vec<Hierarchy>>
    where
        I: IntoIterator<Item = i64>,
    {
        let ids: Vec<i64> = it.into_iter().collect();

        debug!("Get hierarchy by id: ids = {:?}", ids);

        let mut result = Vec::new();

        for row in self
            .client
            .query(self.query_schema.hierarchy_by_id(), &[&ids])
            .map_err(DatabaseError::query_execution_error)?
        {
            let id: i64 = row.try_get(0).map_err(DatabaseError::value_error)?;
            let level_1: Option<i64> = row.try_get(1).map_err(DatabaseError::value_error)?;
            let level_2: Option<i64> = row.try_get(2).map_err(DatabaseError::value_error)?;
            let level_3: Option<i64> = row.try_get(3).map_err(DatabaseError::value_error)?;
            let level_4: Option<i64> = row.try_get(4).map_err(DatabaseError::value_error)?;
            let level_5: Option<i64> = row.try_get(5).map_err(DatabaseError::value_error)?;

            result.push(Hierarchy::new(
                id, level_1, level_2, level_3, level_4, level_5,
            ));
        }

        Ok(result)
    }
}

#[derive(Debug)]
struct RegionBuilder {
    default_name: Option<String>,
    names: Vec<RegionName>,
    lower_name_set: HashSet<String>,
}

impl RegionBuilder {
    fn new() -> RegionBuilder {
        RegionBuilder {
            default_name: None,
            names: Vec::new(),
            lower_name_set: HashSet::new(),
        }
    }

    fn insert_name(&mut self, language: String, name: String, is_default: bool) {
        if is_default {
            self.default_name = Some(name.clone());
        }

        self.names.push(RegionName::new(language, name.clone()));
        self.lower_name_set.insert(name.to_lowercase());
    }

    fn build(self) -> Region {
        let default_name = self
            .default_name
            .unwrap_or_else(|| "<no default name>".into());

        Region::new(default_name, self.names, self.lower_name_set)
    }
}

#[derive(Debug, Clone)]
pub struct Region {
    default_name: String,
    names: Vec<RegionName>,
    lower_name_set: HashSet<String>,
}

impl Region {
    fn new(
        default_name: String,
        names: Vec<RegionName>,
        lower_name_set: HashSet<String>,
    ) -> Region {
        Region {
            default_name,
            names,
            lower_name_set,
        }
    }

    pub fn default_name(&self) -> &str {
        &self.default_name
    }

    pub fn names(&self) -> &[RegionName] {
        &self.names
    }

    pub fn contains_name(&self, pattern: &str) -> bool {
        let pattern = pattern.to_lowercase();

        self.lower_name_set
            .iter()
            .any(|name| name.contains(&pattern))
    }
}

#[derive(Debug, Clone)]
pub struct RegionName {
    language: String,
    name: String,
}

impl RegionName {
    fn new(language: String, name: String) -> RegionName {
        RegionName { language, name }
    }

    pub fn language(&self) -> &str {
        &self.language
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone)]
pub struct Hierarchy {
    id: i64,
    parts: Vec<i64>,
}

impl Hierarchy {
    fn new(
        id: i64,
        level_1: Option<i64>,
        level_2: Option<i64>,
        level_3: Option<i64>,
        level_4: Option<i64>,
        level_5: Option<i64>,
    ) -> Hierarchy {
        let parts = [level_1, level_2, level_3, level_4, level_5]
            .iter()
            .filter_map(|&part| part)
            .collect();

        Hierarchy { id, parts }
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn parts(&self) -> &[i64] {
        &self.parts
    }
}
