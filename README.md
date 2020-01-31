# Find Region

Web interface to subset search query set. This utility allows to find all region within given hierarchy.

## Usage

Start find_region with default configuration:

```bash
./find_region
```

Optional arguments:

* `-a` (`--address`) ADDR: Address to listen on, default value - localhost;
* `-p` (`--port`) PORT: Port to listen on, default value - 8080;
* `-c` (`--config`) PATH: Path to configuration file, default value - config.yaml;
* `-h` (`--help`): Show help and exit.

## Configuration Example

Simple configuration example:

```yaml
---
connections:
  update_command: ~ # Command to dynamically generate connection list
  static_connections: # Connections which always present in connections
    - description: "regions" # connection description
      query_schema: "SCHEMA" # query schema name, link to `query_schemas`
      host: "localhost" # host name or ip address
      port: 5432 # port
      database: "n11" # database name
      role: "postgres" # user name
      password: "postgres" # optionsl password

query_schemas: # contains map query schema name to schema
  "SCHEMA": # name of this schema
    regions_by_name: | # query to find all regions with given name (name provided as is)
      select
        region_id::bigint as id,
        language_code as language_code,
        name as name,
        is_defaul as is_default
      from region_names
      where snn.name ilike $1
      order by feature_id, language_code, name
    region_by_id: | # query to select all region names using region identifier
      select
        language_code as language_code,
        name as name,
        is_default
      from region_names
      where region_id::bigint = $1
      order by language_code, name
    hierarchy_by_id: | # query to select region administrative hierarchy
      select
        id::bigint as id,
        level_1::bigint as level_1,
        level_2::bigint as level_2,
        level_3::bigint as level_3,
        level_4::bigint as level_4,
        level_5::bigint as level_5
      from region_hierarchy
      where id::bigint = $1
      order by id
```
