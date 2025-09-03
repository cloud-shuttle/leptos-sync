//! Storage abstraction layer

pub trait LocalStorage: Clone + Send + Sync + 'static {
    type Error: std::error::Error + Send + Sync;
    
    async fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Self::Error>;
    async fn set<T: serde::Serialize>(&self, key: &str, value: &T) -> Result<(), Self::Error>;
    async fn delete(&self, key: &str) -> Result<(), Self::Error>;
    async fn keys(&self) -> Result<Vec<String>, Self::Error>;
    async fn clear(&self) -> Result<(), Self::Error>;
}
