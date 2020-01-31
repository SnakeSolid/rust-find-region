mod error;

pub use self::error::DynamicConnectionsError;
pub use self::error::DynamicConnectionsResult;

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
}

#[derive(Debug)]
struct DynamicConnections {
    connections: HashMap<usize, ConnectionSettings>,
}

impl DynamicConnections {
    fn new() -> DynamicConnections {
        DynamicConnections {
            connections: HashMap::new(),
        }
    }

    fn get(&self, index: usize) -> Option<&ConnectionSettings> {
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
}

pub fn dynamic_connections() -> DynamicConnectionsRef {
    DynamicConnectionsRef {
        inner: Arc::new(RwLock::new(DynamicConnections::new())),
    }
}
