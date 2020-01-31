use crate::config::ConfigRef;
use crate::database::Database;
use crate::database::DatabaseClient;
use crate::database::Hierarchy as DbHierarchy;
use crate::database::Region as DbRegion;
use crate::handler::error::HandlerError;
use crate::handler::error::HandlerResult;
use crate::handler::util::handle_request;
use crate::manager::DynamicConnectionsRef;
use iron::middleware::Handler;
use iron::IronResult;
use iron::Request as IromRequest;
use iron::Response as IromResponse;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Debug)]
pub struct FindRegionHandler {
    config: ConfigRef,
    dynamic_connections: DynamicConnectionsRef,
}

const QUERY_SEPARATOR: char = '>';

impl FindRegionHandler {
    pub fn new(config: ConfigRef, dynamic_connections: DynamicConnectionsRef) -> FindRegionHandler {
        FindRegionHandler {
            config,
            dynamic_connections,
        }
    }

    fn prepare_query(&self, query: String) -> HandlerResult<(String, Vec<String>)> {
        if query.is_empty() {
            return Err(HandlerError::new("Region name query must not be empty"));
        }

        let query_parts: Vec<_> = query
            .split(QUERY_SEPARATOR)
            .map(|name| name.trim().to_lowercase())
            .collect();

        if query_parts.iter().any(|name| name.is_empty()) {
            return Err(HandlerError::new("Region name must not be empty"));
        }

        match query_parts.last() {
            Some(last) => Ok((last.into(), query_parts)),
            None => Err(HandlerError::new("Query must contain at least one part")),
        }
    }

    fn prepare_connection(&self, index: usize) -> HandlerResult<DatabaseClient> {
        let dynamic_connection = self.dynamic_connections.get(index).unwrap_or(None);
        let static_connection = self.config.connections().static_connections().get(index);
        let connection = match static_connection.cloned().or(dynamic_connection) {
            Some(connection) => connection,
            None => {
                return Err(HandlerError::new(&format!(
                    "Invalid connection index `{}`",
                    index
                )))
            }
        };
        let query_schema = match self.config.query_schemas().get(connection.query_schema()) {
            Some(query_schema) => query_schema,
            None => {
                return Err(HandlerError::new(&format!(
                    "Invalid query schema `{}` in connection `{}`",
                    connection.query_schema(),
                    connection.description(),
                )))
            }
        };

        Database::new(connection, query_schema)
            .connect()
            .map_err(|_| HandlerError::new("Failed to connect to database"))
    }

    fn collect_hierarchy<I>(
        &self,
        client: &mut DatabaseClient,
        it: I,
    ) -> HandlerResult<Vec<DbHierarchy>>
    where
        I: IntoIterator<Item = i64>,
    {
        client
            .hierarchy_by_id(it)
            .map_err(|_| HandlerError::new("Failed to query hierarchy"))
    }

    fn collect_all_regions(
        &self,
        client: &mut DatabaseClient,
        regions: &HashMap<i64, DbRegion>,
        hierarchies: &[DbHierarchy],
    ) -> HandlerResult<HashMap<i64, DbRegion>> {
        let mut region_ids = HashSet::new();

        for hierarchy in hierarchies {
            region_ids.insert(hierarchy.id());
            region_ids.extend(hierarchy.parts());
        }

        let extended_regions = client
            .regions_by_id(
                region_ids
                    .into_iter()
                    .filter(|region_id| !regions.contains_key(region_id)),
            )
            .map_err(|_| HandlerError::new("Failed to query region name"))?;
        let mut result = regions.clone();
        result.extend(extended_regions);

        Ok(result)
    }

    fn collect_query_hierarchies<'a>(
        &self,
        query_parts: &[String],
        regions: &HashMap<i64, DbRegion>,
        hierarchies: &'a [DbHierarchy],
    ) -> Vec<&'a DbHierarchy> {
        hierarchies
            .iter()
            .filter(|hierarchy| self.is_hierarchy_matches(hierarchy, query_parts, regions))
            .collect()
    }

    fn is_hierarchy_matches(
        &self,
        _hierarchy: &DbHierarchy,
        _query_parts: &[String],
        _regions: &HashMap<i64, DbRegion>,
    ) -> bool {
        true
    }
}

impl Handler for FindRegionHandler {
    fn handle(&self, request: &mut IromRequest) -> IronResult<IromResponse> {
        handle_request(request, move |request: Request| {
            let (name, query_parts) = self.prepare_query(request.query)?;
            let mut client = self.prepare_connection(request.connection)?;
            let query_regions = client
                .regions_by_name(&name)
                .map_err(|_| HandlerError::new("Failed to query region by name"))?;
            let extended_hierarchies =
                self.collect_hierarchy(&mut client, query_regions.keys().cloned())?;
            let all_regions =
                self.collect_all_regions(&mut client, &query_regions, &extended_hierarchies)?;
            let query_hierarchies =
                self.collect_query_hierarchies(&query_parts, &all_regions, &extended_hierarchies);
            let regions = all_regions
                .into_iter()
                .map(|(id, region)| (id, region.into()))
                .collect();
            let hierarchies = query_hierarchies.into_iter().map(Hierarchy::from).collect();

            Ok(Response {
                regions,
                hierarchies,
            })
        })
    }
}

#[derive(Debug, Deserialize)]
struct Request {
    connection: usize,
    query: String,
}

#[derive(Debug, Serialize)]
struct Response {
    regions: HashMap<i64, Region>,
    hierarchies: Vec<Hierarchy>,
}

#[derive(Debug, Serialize)]
struct Region {
    default_name: String,
    names: HashMap<String, String>,
}

impl From<DbRegion> for Region {
    fn from(region: DbRegion) -> Region {
        Region {
            default_name: region.default_name().into(),
            names: region
                .names()
                .iter()
                .map(|name| (name.language().into(), name.name().into()))
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
struct Hierarchy {
    id: i64,
    parts: Vec<i64>,
    bigger: bool,
}

impl From<&DbHierarchy> for Hierarchy {
    fn from(hierarchy: &DbHierarchy) -> Hierarchy {
        let bigger = match hierarchy.parts().last() {
            Some(&id) => id != hierarchy.id(),
            None => true,
        };

        Hierarchy {
            id: hierarchy.id(),
            parts: hierarchy.parts().into(),
            bigger,
        }
    }
}
