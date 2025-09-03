//! Query API for local-first collections

use serde::{Deserialize, Serialize};

/// Query builder for filtering and sorting data
pub struct QueryBuilder<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> QueryBuilder<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn filter<F>(self, _predicate: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        self
    }

    pub fn sort_by<F>(self, _comparator: F) -> Self
    where
        F: Fn(&T, &T) -> std::cmp::Ordering + 'static,
    {
        self
    }

    pub fn limit(self, _limit: usize) -> Self {
        self
    }

    pub fn watch(self) -> Self {
        self
    }
}

impl<T> Default for QueryBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
