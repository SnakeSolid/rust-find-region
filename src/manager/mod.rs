mod error;

pub use self::error::DynamicConnectionsError;
pub use self::error::DynamicConnectionsResult;

use crate::config::Config;
use crate::config::ConnectionSettings;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

#[derive(Debug, Clone)]
pub struct DynamicConnectionsRef {
    inner: Arc<RwLock<DynamicConnections>>,
}

impl DynamicConnectionsRef {
    fn with_read<F, T>(&self, callback: F) -> DynamicConnectionsResult<T>
    where
        F: FnOnce(&DynamicConnections) -> DynamicConnectionsResult<T>,
    {
        match self.inner.write() {
            Ok(ref manager) => callback(manager),
            Err(err) => {
                warn!("Failed to acquire write lock - {}", err);

                Err(DynamicConnectionsError::new("Failed to acquire write lock"))
            }
        }
    }

    fn with_write<F, T>(&self, callback: F) -> DynamicConnectionsResult<T>
    where
        F: FnOnce(&mut DynamicConnections) -> DynamicConnectionsResult<T>,
    {
        match self.inner.write() {
            Ok(ref mut manager) => callback(manager),
            Err(err) => {
                warn!("Failed to acquire write lock - {}", err);

                Err(DynamicConnectionsError::new("Failed to acquire write lock"))
            }
        }
    }

    pub fn get(&self, index: usize) -> DynamicConnectionsResult<Option<ConnectionSettings>> {
        self.with_read(move |manager| Ok(manager.get(index).cloned()))
    }

    pub fn for_each<F>(&self, callback: F) -> DynamicConnectionsResult<()>
    where
        F: FnMut(usize, &ConnectionSettings),
    {
        self.with_read(move |manager| {
            manager.for_each(callback);
            Ok(())
        })
    }

    pub fn update(
        &self,
        index: usize,
        connection: ConnectionSettings,
    ) -> DynamicConnectionsResult<()> {
        self.with_write(move |manager| {
            manager.update(index, connection);
            Ok(())
        })
    }

    pub fn insert(&self, connection: ConnectionSettings) -> DynamicConnectionsResult<()> {
        self.with_write(move |manager| {
            manager.insert(connection);
            Ok(())
        })
    }

    pub fn remove(&self, index: usize) -> DynamicConnectionsResult<()> {
        self.with_write(move |manager| {
            manager.remove(index);
            Ok(())
        })
    }
}

#[derive(Debug)]
struct DynamicConnections {
    valid_index: usize,
    connections: HashMap<usize, ConnectionSettings>,
}

impl DynamicConnections {
    fn new(valid_index: usize) -> DynamicConnections {
        DynamicConnections {
            valid_index,
            connections: HashMap::new(),
        }
    }

    fn get(&self, index: usize) -> Option<&ConnectionSettings> {
        debug!("Get dynamic connection: index = {}", index);

        self.connections.get(&index)
    }

    fn for_each<F>(&self, mut callback: F)
    where
        F: FnMut(usize, &ConnectionSettings),
    {
        self.connections
            .iter()
            .for_each(|(&index, connection)| callback(index, connection))
    }

    fn update(&mut self, index: usize, connection: ConnectionSettings) {
        debug!(
            "Update dynamic connection: index = {}, connection = {:?}",
            index, connection
        );

        self.connections.insert(index, connection);
    }

    fn insert(&mut self, connection: ConnectionSettings) {
        debug!(
            "Insert dynamic connection: valid_index = {}, connection = {:?}",
            self.valid_index, connection
        );

        self.connections.insert(self.valid_index, connection);

        self.valid_index += 1;
    }

    fn remove(&mut self, index: usize) {
        debug!("Remove dynamic connection: index = {}", index,);

        self.connections.remove(&index);
    }
}

pub fn dynamic_connections(config: &Config) -> DynamicConnectionsRef {
    let valid_index = config.connections().static_connections().len();

    DynamicConnectionsRef {
        inner: Arc::new(RwLock::new(DynamicConnections::new(valid_index))),
    }
}
