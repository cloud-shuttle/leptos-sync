use super::{SyncTransport, TransportError};
use super::leptos_ws_pro_transport::LeptosWsProTransport;
use super::compatibility_layer::CompatibilityTransport;
use super::memory::InMemoryTransport;
use super::websocket::WebSocketTransport;

/// Hybrid transport that can use multiple backends
#[derive(Clone)]
pub enum HybridTransport {
    WebSocket(WebSocketTransport),
    LeptosWsPro(LeptosWsProTransport),
    Compatibility(CompatibilityTransport),
    InMemory(InMemoryTransport),
    Fallback {
        primary: Box<HybridTransport>,
        fallback: Box<HybridTransport>,
    },
}

impl HybridTransport {
    pub fn with_websocket(url: String) -> Self {
        Self::WebSocket(WebSocketTransport::new(url))
    }

    pub fn with_leptos_ws_pro(config: super::leptos_ws_pro_transport::LeptosWsProConfig) -> Self {
        Self::LeptosWsPro(LeptosWsProTransport::new(config))
    }

    pub fn with_compatibility(config: super::leptos_ws_pro_transport::LeptosWsProConfig) -> Self {
        Self::Compatibility(CompatibilityTransport::new(config))
    }

    pub fn with_in_memory() -> Self {
        Self::InMemory(InMemoryTransport::new())
    }

    pub fn with_fallback(primary: HybridTransport, fallback: HybridTransport) -> Self {
        Self::Fallback { 
            primary: Box::new(primary), 
            fallback: Box::new(fallback) 
        }
    }

    pub fn add_transport(&mut self, transport: HybridTransport) {
        // For now, just replace the current transport
        // In a more sophisticated implementation, you'd maintain a list
        *self = transport;
    }

    pub async fn connect(&self) -> Result<(), TransportError> {
        match self {
            HybridTransport::WebSocket(ws) => ws.connect().await,
            HybridTransport::LeptosWsPro(leptos_ws) => leptos_ws.connect().await.map_err(|e| e.into()),
            HybridTransport::Compatibility(compat) => compat.connect().await.map_err(|e| e.into()),
            HybridTransport::InMemory(_) => Ok(()), // In-memory is always "connected"
            HybridTransport::Fallback { primary, fallback } => {
                // Try primary first, fall back to fallback
                match Box::pin(primary.connect()).await {
                    Ok(()) => Ok(()),
                    Err(_) => Box::pin(fallback.connect()).await,
                }
            }
        }
    }

    pub async fn disconnect(&self) -> Result<(), TransportError> {
        match self {
            HybridTransport::WebSocket(ws) => ws.disconnect().await,
            HybridTransport::LeptosWsPro(leptos_ws) => leptos_ws.disconnect().await.map_err(|e| e.into()),
            HybridTransport::Compatibility(compat) => compat.disconnect().await.map_err(|e| e.into()),
            HybridTransport::InMemory(_) => Ok(()), // In-memory disconnect is always successful
            HybridTransport::Fallback { primary, fallback } => {
                // Disconnect both transports
                let _ = Box::pin(primary.disconnect()).await;
                let _ = Box::pin(fallback.disconnect()).await;
                Ok(())
            }
        }
    }
}

impl SyncTransport for HybridTransport {
    type Error = TransportError;

    fn send<'a>(&'a self, data: &'a [u8]) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), Self::Error>> + Send + 'a>> {
        Box::pin(async move {
            match self {
                HybridTransport::WebSocket(ws) => ws.send(data).await,
                HybridTransport::LeptosWsPro(leptos_ws) => leptos_ws.send(data).await.map_err(|e| e.into()),
                HybridTransport::Compatibility(compat) => compat.send(data).await.map_err(|e| e.into()),
                HybridTransport::InMemory(mem) => mem.send(data).await,
                HybridTransport::Fallback { primary, fallback } => {
                    // Try primary first, fall back to fallback
                    match Box::pin(primary.send(data)).await {
                        Ok(()) => Ok(()),
                        Err(_) => Box::pin(fallback.send(data)).await,
                    }
                }
            }
        })
    }

    fn receive(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<Vec<u8>>, Self::Error>> + Send + '_>> {
        Box::pin(async move {
            match self {
                HybridTransport::WebSocket(ws) => ws.receive().await,
                HybridTransport::LeptosWsPro(leptos_ws) => leptos_ws.receive().await.map_err(|e| e.into()),
                HybridTransport::Compatibility(compat) => compat.receive().await.map_err(|e| e.into()),
                HybridTransport::InMemory(mem) => mem.receive().await,
                HybridTransport::Fallback { primary, fallback } => {
                    // Try primary first, fall back to fallback
                    match Box::pin(primary.receive()).await {
                        Ok(messages) => Ok(messages),
                        Err(_) => Box::pin(fallback.receive()).await,
                    }
                }
            }
        })
    }

    fn is_connected(&self) -> bool {
        match self {
            HybridTransport::WebSocket(ws) => ws.is_connected(),
            HybridTransport::LeptosWsPro(leptos_ws) => leptos_ws.is_connected(),
            HybridTransport::Compatibility(compat) => compat.is_connected(),
            HybridTransport::InMemory(mem) => mem.is_connected(),
            HybridTransport::Fallback { primary, fallback } => {
                primary.is_connected() || fallback.is_connected()
            }
        }
    }
}
