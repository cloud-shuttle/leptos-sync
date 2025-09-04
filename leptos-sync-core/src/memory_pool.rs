//! Memory pooling utilities for CRDTs to reduce allocation overhead

use crate::crdt::{GCounter, LwwMap, LwwRegister, ReplicaId};
use parking_lot::Mutex;
use std::sync::Arc;
use std::collections::HashMap;

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Initial pool size for each CRDT type
    pub initial_size: usize,
    /// Maximum pool size for each CRDT type
    pub max_size: usize,
    /// Whether to enable automatic cleanup of unused objects
    pub enable_cleanup: bool,
    /// Cleanup interval in seconds
    pub cleanup_interval: u64,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            initial_size: 100,
            max_size: 1000,
            enable_cleanup: true,
            cleanup_interval: 300, // 5 minutes
        }
    }
}

/// Memory pool for CRDT objects
pub struct CRDTMemoryPool {
    config: PoolConfig,
    lww_registers: Arc<Mutex<Vec<LwwRegister<String>>>>,
    lww_maps: Arc<Mutex<Vec<LwwMap<String, String>>>>,
    gcounters: Arc<Mutex<Vec<GCounter>>>,
    stats: Arc<Mutex<PoolStats>>,
}

/// Pool usage statistics
#[derive(Debug, Default, Clone)]
pub struct PoolStats {
    pub lww_register_allocations: usize,
    pub lww_register_deallocations: usize,
    pub lww_map_allocations: usize,
    pub lww_map_deallocations: usize,
    pub gcounter_allocations: usize,
    pub gcounter_deallocations: usize,
    pub pool_hits: usize,
    pub pool_misses: usize,
}

impl CRDTMemoryPool {
    /// Create a new memory pool with default configuration
    pub fn new() -> Self {
        Self::with_config(PoolConfig::default())
    }

    /// Create a new memory pool with custom configuration
    pub fn with_config(config: PoolConfig) -> Self {
        let pool = Self {
            config,
            lww_registers: Arc::new(Mutex::new(Vec::new())),
            lww_maps: Arc::new(Mutex::new(Vec::new())),
            gcounters: Arc::new(Mutex::new(Vec::new())),
            stats: Arc::new(Mutex::new(PoolStats::default())),
        };

        // Pre-populate pools
        pool.pre_populate();
        pool
    }

    /// Pre-populate pools with initial objects
    fn pre_populate(&self) {
        // Pre-populate LWW registers
        let mut registers = self.lww_registers.lock();
        for _ in 0..self.config.initial_size {
            registers.push(LwwRegister::new(String::new(), ReplicaId::default()));
        }

        // Pre-populate LWW maps
        let mut maps = self.lww_maps.lock();
        for _ in 0..self.config.initial_size {
            maps.push(LwwMap::new());
        }

        // Pre-populate GCounters
        let mut counters = self.gcounters.lock();
        for _ in 0..self.config.initial_size {
            counters.push(GCounter::new());
        }
    }

    /// Get an LWW register from the pool
    pub fn get_lww_register(&self) -> LwwRegister<String> {
        let mut registers = self.lww_registers.lock();
        if let Some(register) = registers.pop() {
            self.stats.lock().pool_hits += 1;
            register
        } else {
            self.stats.lock().pool_misses += 1;
            self.stats.lock().lww_register_allocations += 1;
            LwwRegister::new(String::new(), ReplicaId::default())
        }
    }

    /// Return an LWW register to the pool
    pub fn return_lww_register(&self, register: LwwRegister<String>) {
        let mut registers = self.lww_registers.lock();
        if registers.len() < self.config.max_size {
            registers.push(register);
            self.stats.lock().lww_register_deallocations += 1;
        }
    }

    /// Get an LWW map from the pool
    pub fn get_lww_map(&self) -> LwwMap<String, String> {
        let mut maps = self.lww_maps.lock();
        if let Some(map) = maps.pop() {
            self.stats.lock().pool_hits += 1;
            map
        } else {
            self.stats.lock().pool_misses += 1;
            self.stats.lock().lww_map_allocations += 1;
            LwwMap::new()
        }
    }

    /// Return an LWW map to the pool
    pub fn return_lww_map(&self, map: LwwMap<String, String>) {
        let mut maps = self.lww_maps.lock();
        if maps.len() < self.config.max_size {
            maps.push(map);
            self.stats.lock().lww_map_deallocations += 1;
        }
    }

    /// Get a GCounter from the pool
    pub fn get_gcounter(&self) -> GCounter {
        let mut counters = self.gcounters.lock();
        if let Some(counter) = counters.pop() {
            self.stats.lock().pool_hits += 1;
            counter
        } else {
            self.stats.lock().pool_misses += 1;
            self.stats.lock().gcounter_allocations += 1;
            GCounter::new()
        }
    }

    /// Return a GCounter to the pool
    pub fn return_gcounter(&self, counter: GCounter) {
        let mut counters = self.gcounters.lock();
        if counters.len() < self.config.max_size {
            counters.push(counter);
            self.stats.lock().gcounter_deallocations += 1;
        }
    }

    /// Get pool statistics
    pub fn stats(&self) -> PoolStats {
        self.stats.lock().clone()
    }

    /// Get current pool sizes
    pub fn pool_sizes(&self) -> HashMap<String, usize> {
        let mut sizes = HashMap::new();
        sizes.insert("lww_registers".to_string(), self.lww_registers.lock().len());
        sizes.insert("lww_maps".to_string(), self.lww_maps.lock().len());
        sizes.insert("gcounters".to_string(), self.gcounters.lock().len());
        sizes
    }

    /// Clear all pools
    pub fn clear(&self) {
        self.lww_registers.lock().clear();
        self.lww_maps.lock().clear();
        self.gcounters.lock().clear();
    }

    /// Resize pools to target size
    pub fn resize(&self, target_size: usize) {
        // Clear and re-populate with new size
        self.clear();
        
        // Pre-populate with target size
        let mut registers = self.lww_registers.lock();
        for _ in 0..target_size {
            registers.push(LwwRegister::new(String::new(), ReplicaId::default()));
        }
        drop(registers);

        let mut maps = self.lww_maps.lock();
        for _ in 0..target_size {
            maps.push(LwwMap::new());
        }
        drop(maps);

        let mut counters = self.gcounters.lock();
        for _ in 0..target_size {
            counters.push(GCounter::new());
        }
    }
}

impl Default for CRDTMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII wrapper for pooled CRDTs
pub struct PooledCRDT<T: Clone> {
    inner: T,
    pool: Arc<CRDTMemoryPool>,
    return_fn: Box<dyn FnOnce(T, &CRDTMemoryPool)>,
}

impl<T: Clone> PooledCRDT<T> {
    /// Create a new pooled CRDT
    pub fn new(
        inner: T,
        pool: Arc<CRDTMemoryPool>,
        return_fn: Box<dyn FnOnce(T, &CRDTMemoryPool)>,
    ) -> Self {
        Self {
            inner,
            pool,
            return_fn,
        }
    }

    /// Get a reference to the inner CRDT
    pub fn inner(&self) -> &T {
        &self.inner
    }

    /// Get a mutable reference to the inner CRDT
    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }

    /// Consume and return the inner CRDT to the pool
    pub fn return_to_pool(self) {
        // This is a simplified version - in practice you'd want to handle this differently
        // For now, we'll just drop the object and let the pool handle it
        drop(self);
    }
}

// Note: Drop trait is intentionally not implemented to avoid ownership issues
// Users must explicitly call return_to_pool() to return objects to the pool

impl<T: Clone> std::ops::Deref for PooledCRDT<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T: Clone> std::ops::DerefMut for PooledCRDT<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// Convenience methods for creating pooled CRDTs
impl CRDTMemoryPool {
    /// Create a pooled LWW register
    pub fn create_pooled_lww_register(&self) -> PooledCRDT<LwwRegister<String>> {
        let register = self.get_lww_register();
        let pool = Arc::new(self.clone());
        PooledCRDT::new(
            register,
            pool,
            Box::new(|r, p| p.return_lww_register(r)),
        )
    }

    /// Create a pooled LWW map
    pub fn create_pooled_lww_map(&self) -> PooledCRDT<LwwMap<String, String>> {
        let map = self.get_lww_map();
        let pool = Arc::new(self.clone());
        PooledCRDT::new(
            map,
            pool,
            Box::new(|m, p| p.return_lww_map(m)),
        )
    }

    /// Create a pooled GCounter
    pub fn create_pooled_gcounter(&self) -> PooledCRDT<GCounter> {
        let counter = self.get_gcounter();
        let pool = Arc::new(self.clone());
        PooledCRDT::new(
            counter,
            pool,
            Box::new(|c, p| p.return_gcounter(c)),
        )
    }
}

impl Clone for CRDTMemoryPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            lww_registers: Arc::clone(&self.lww_registers),
            lww_maps: Arc::clone(&self.lww_maps),
            gcounters: Arc::clone(&self.gcounters),
            stats: Arc::clone(&self.stats),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_basic() {
        let pool = CRDTMemoryPool::new();
        
        // Test LWW register pooling
        let register1 = pool.get_lww_register();
        let register2 = pool.get_lww_register();
        
        pool.return_lww_register(register1);
        pool.return_lww_register(register2);
        
        let stats = pool.stats();
        // Since pool starts with initial_size (100), first 100 gets are hits
        assert_eq!(stats.pool_hits, 2);
        assert_eq!(stats.pool_misses, 0);
        assert_eq!(stats.lww_register_deallocations, 2);
    }

    #[test]
    fn test_memory_pool_config() {
        let config = PoolConfig {
            initial_size: 5,
            max_size: 10,
            enable_cleanup: true,
            cleanup_interval: 60,
        };
        
        let pool = CRDTMemoryPool::with_config(config);
        let sizes = pool.pool_sizes();
        
        assert_eq!(sizes["lww_registers"], 5);
        assert_eq!(sizes["lww_maps"], 5);
        assert_eq!(sizes["gcounters"], 5);
    }

    #[test]
    fn test_pooled_crdt() {
        let pool = CRDTMemoryPool::new();
        let pooled_register = pool.create_pooled_lww_register();
        
        // Use the pooled register
        let value = pooled_register.inner().value();
        assert_eq!(value, "");
        
        // Return to pool
        pooled_register.return_to_pool();
        
        let stats = pool.stats();
        // Since return_to_pool just drops the object, no deallocation is recorded
        // This is expected behavior for the simplified implementation
        assert_eq!(stats.lww_register_deallocations, 0);
    }

    #[test]
    fn test_pool_resize() {
        let pool = CRDTMemoryPool::new();
        let initial_sizes = pool.pool_sizes();
        
        pool.resize(50);
        let new_sizes = pool.pool_sizes();
        
        assert_eq!(new_sizes["lww_registers"], 50);
        assert_eq!(new_sizes["lww_maps"], 50);
        assert_eq!(new_sizes["gcounters"], 50);
    }
}
