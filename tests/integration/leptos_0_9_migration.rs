//! Integration tests for Leptos 0.9 migration
//!
//! Tests that validate functionality after migrating from Leptos 0.8 to 0.9.

use leptos::*;
use leptos_sync_core::crdt::{CrdtType, ReplicaId};
use leptos_sync_core::transport::{
    message_protocol::{PresenceAction, ServerInfo, SyncMessage, UserInfo},
    SyncTransport, WebSocketClient, WebSocketClientConfig,
};
use leptos_sync_core::*;
use std::time::{Duration, SystemTime};
use tokio::time::timeout;
use uuid::Uuid;

/// Test basic Leptos 0.9 functionality
mod basic_tests {
    use super::*;

    #[component]
    fn TestComponent(prop: String) -> impl IntoView {
        view! {
            <div>{prop}</div>
        }
    }

    #[component]
    fn TestComponentWithProps(
        #[prop(optional)] optional_prop: Option<String>,
        #[prop(default = "default".to_string())] default_prop: String,
    ) -> impl IntoView {
        view! {
            <div>
                {optional_prop.unwrap_or("none".to_string())}
                {default_prop}
            </div>
        }
    }

    #[component]
    fn TestComponentWithChildren(children: Children) -> impl IntoView {
        view! {
            <div>
                {children()}
            </div>
        }
    }

    #[test]
    fn test_basic_component_rendering() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let component = TestComponent {
                prop: "test".to_string(),
            };
            // Test component rendering
            assert!(component.render().is_ok());
        });
    }

    #[test]
    fn test_component_with_props() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let component = TestComponentWithProps {
                optional_prop: Some("optional".to_string()),
                default_prop: "default".to_string(),
            };
            // Test component with props
            assert!(component.render().is_ok());
        });
    }

    #[test]
    fn test_component_with_children() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let component = TestComponentWithChildren {
                children: Children::new(|_| view! { <span>"child"</span> }),
            };
            // Test component with children
            assert!(component.render().is_ok());
        });
    }
}

/// Test signal functionality with Leptos 0.9
mod signal_tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let (count, set_count) = create_signal(0);
            assert_eq!(count(), 0);
            set_count(1);
            assert_eq!(count(), 1);
        });
    }

    #[test]
    fn test_signal_memo() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let (count, set_count) = create_signal(0);
            let doubled = create_memo(move || count() * 2);

            assert_eq!(doubled(), 0);
            set_count(1);
            assert_eq!(doubled(), 2);
        });
    }

    #[test]
    fn test_signal_effect() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let (count, set_count) = create_signal(0);
            let effect_count = create_signal(0);

            create_effect(move || {
                let _ = count();
                effect_count.set(effect_count() + 1);
            });

            assert_eq!(effect_count(), 1);
            set_count(1);
            assert_eq!(effect_count(), 2);
        });
    }

    #[test]
    fn test_signal_resource() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let data = create_resource(|| (), |_| async move { "Hello from resource".to_string() });

            // Test resource creation
            assert!(data.get().is_some());
        });
    }
}

/// Test server function functionality with Leptos 0.9
mod server_function_tests {
    use super::*;
    use leptos::server_fn::*;

    #[server(GetData)]
    pub async fn get_data() -> Result<String, ServerFnError> {
        Ok("Hello from server".to_string())
    }

    #[server(GetDataWithParams)]
    pub async fn get_data_with_params(param: String) -> Result<String, ServerFnError> {
        Ok(format!("Hello {}", param))
    }

    #[server(GetDataError)]
    pub async fn get_data_error() -> Result<String, ServerFnError> {
        Err(ServerFnError::ServerError("Test error".to_string()))
    }

    #[tokio::test]
    async fn test_basic_server_function() {
        let result = get_data().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello from server");
    }

    #[tokio::test]
    async fn test_server_function_with_params() {
        let result = get_data_with_params("test".to_string()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello test");
    }

    #[tokio::test]
    async fn test_server_function_error_handling() {
        let result = get_data_error().await;
        assert!(result.is_err());
    }
}

/// Test WebSocket functionality with Leptos 0.9
mod websocket_tests {
    use super::*;

    fn create_test_config() -> WebSocketClientConfig {
        WebSocketClientConfig {
            url: "ws://localhost:3001/test".to_string(),
            heartbeat_interval: Duration::from_secs(30),
            message_timeout: Duration::from_secs(5),
            reconnect_interval: Duration::from_secs(1),
            max_reconnect_attempts: 3,
            user_info: Some(UserInfo {
                user_id: "test_user".to_string(),
                username: Some("testuser".to_string()),
                display_name: Some("Test User".to_string()),
                avatar_url: None,
            }),
        }
    }

    fn create_test_replica_id() -> ReplicaId {
        ReplicaId::from(Uuid::new_v4())
    }

    fn create_test_delta_message() -> SyncMessage {
        SyncMessage::Delta {
            collection_id: "test-collection".to_string(),
            crdt_type: CrdtType::LwwRegister,
            delta: vec![1, 2, 3, 4],
            timestamp: SystemTime::now(),
            replica_id: create_test_replica_id(),
        }
    }

    #[tokio::test]
    async fn test_websocket_client_creation() {
        let config = create_test_config();
        let replica_id = create_test_replica_id();
        let client = WebSocketClient::new(config, replica_id);

        // Test client creation
        assert_eq!(client.replica_id(), replica_id);
    }

    #[tokio::test]
    async fn test_websocket_message_serialization() {
        let message = create_test_delta_message();

        // Test message serialization
        let serialized = serde_json::to_string(&message).unwrap();
        assert!(!serialized.is_empty());

        // Test deserialization
        let deserialized: SyncMessage = serde_json::from_str(&serialized).unwrap();
        match (&message, &deserialized) {
            (
                SyncMessage::Delta {
                    collection_id: id1,
                    crdt_type: type1,
                    delta: delta1,
                    replica_id: rid1,
                    ..
                },
                SyncMessage::Delta {
                    collection_id: id2,
                    crdt_type: type2,
                    delta: delta2,
                    replica_id: rid2,
                    ..
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(type1, type2);
                assert_eq!(delta1, delta2);
                assert_eq!(rid1, rid2);
            }
            _ => panic!("Message types don't match"),
        }
    }
}

/// Test CRDT functionality with Leptos 0.9
mod crdt_tests {
    use super::*;
    use leptos_sync_core::crdt::basic::*;

    #[test]
    fn test_lww_register() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let mut register = LwwRegister::new(ReplicaId::from(Uuid::new_v4()));

            // Test basic operations
            register.set("Hello".to_string());
            assert_eq!(register.get(), "Hello");

            register.set("World".to_string());
            assert_eq!(register.get(), "World");
        });
    }

    #[test]
    fn test_gcounter() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let mut counter = GCounter::new(ReplicaId::from(Uuid::new_v4()));

            // Test basic operations
            counter.increment();
            assert_eq!(counter.value(), 1);

            counter.increment();
            assert_eq!(counter.value(), 2);
        });
    }

    #[test]
    fn test_lww_map() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let mut map = LwwMap::new(ReplicaId::from(Uuid::new_v4()));

            // Test basic operations
            map.set("key1".to_string(), "value1".to_string());
            assert_eq!(map.get("key1"), Some("value1".to_string()));

            map.set("key2".to_string(), "value2".to_string());
            assert_eq!(map.get("key2"), Some("value2".to_string()));
        });
    }
}

/// Test storage functionality with Leptos 0.9
mod storage_tests {
    use super::*;
    use leptos_sync_core::storage::*;

    #[tokio::test]
    async fn test_memory_storage() {
        let storage = Storage::Memory(MemoryStorage::new());

        // Test basic operations
        storage
            .set("key1".to_string(), "value1".to_string())
            .await
            .unwrap();
        assert_eq!(
            storage.get("key1").await.unwrap(),
            Some("value1".to_string())
        );

        storage
            .set("key2".to_string(), "value2".to_string())
            .await
            .unwrap();
        assert_eq!(
            storage.get("key2").await.unwrap(),
            Some("value2".to_string())
        );

        storage.remove("key1").await.unwrap();
        assert_eq!(storage.get("key1").await.unwrap(), None);
    }

    #[tokio::test]
    async fn test_storage_keys() {
        let storage = Storage::Memory(MemoryStorage::new());

        // Test key enumeration
        storage
            .set("key1".to_string(), "value1".to_string())
            .await
            .unwrap();
        storage
            .set("key2".to_string(), "value2".to_string())
            .await
            .unwrap();

        let keys: Vec<String> = storage.keys().await.unwrap().collect();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
    }
}

/// Test performance with Leptos 0.9
mod performance_tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_component_rendering_performance() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let start = Instant::now();

            // Render 1000 components
            for i in 0..1000 {
                let component = view! {
                    <div>{format!("test_{}", i)}</div>
                };
                let _ = component.render();
            }

            let duration = start.elapsed();
            assert!(duration < Duration::from_secs(1)); // Should complete in under 1 second
        });
    }

    #[test]
    fn test_signal_performance() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let (count, set_count) = create_signal(0);
            let start = Instant::now();

            // Update signal 10000 times
            for i in 0..10000 {
                set_count(i);
            }

            let duration = start.elapsed();
            assert!(duration < Duration::from_secs(1)); // Should complete in under 1 second
        });
    }

    #[test]
    fn test_memo_performance() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let (count, set_count) = create_signal(0);
            let doubled = create_memo(move || count() * 2);
            let start = Instant::now();

            // Update signal 10000 times
            for i in 0..10000 {
                set_count(i);
                let _ = doubled();
            }

            let duration = start.elapsed();
            assert!(duration < Duration::from_secs(1)); // Should complete in under 1 second
        });
    }
}

/// Test integration between Leptos 0.9 and leptos-sync
mod integration_tests {
    use super::*;

    #[component]
    fn SyncComponent() -> impl IntoView {
        let (count, set_count) = create_signal(0);
        let data = create_resource(|| (), |_| async move { "Hello from resource".to_string() });

        view! {
            <div>
                <button on:click=move |_| set_count(count() + 1)>
                    "Count: " {count}
                </button>
                <div>
                    {move || data.get().unwrap_or("Loading...".to_string())}
                </div>
            </div>
        }
    }

    #[test]
    fn test_sync_component_integration() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let component = SyncComponent {};
            // Test component integration
            assert!(component.render().is_ok());
        });
    }

    #[tokio::test]
    async fn test_websocket_integration() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let config = WebSocketClientConfig {
                url: "ws://localhost:3001/test".to_string(),
                heartbeat_interval: Duration::from_secs(30),
                message_timeout: Duration::from_secs(5),
                reconnect_interval: Duration::from_secs(1),
                max_reconnect_attempts: 3,
                user_info: None,
            };

            let replica_id = ReplicaId::from(Uuid::new_v4());
            let client = WebSocketClient::new(config, replica_id);

            // Test WebSocket integration
            assert_eq!(client.replica_id(), replica_id);
        });
    }
}

/// Test error handling with Leptos 0.9
mod error_handling_tests {
    use super::*;

    #[test]
    fn test_component_error_handling() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            // Test error handling in components
            let (count, set_count) = create_signal(0);

            // Test that errors are handled gracefully
            set_count(1);
            assert_eq!(count(), 1);
        });
    }

    #[tokio::test]
    async fn test_server_function_error_handling() {
        let result = server_function_tests::get_data_error().await;
        assert!(result.is_err());

        match result {
            Err(ServerFnError::ServerError(msg)) => {
                assert_eq!(msg, "Test error");
            }
            _ => panic!("Unexpected error type"),
        }
    }
}

/// Test browser/WASM functionality with Leptos 0.9
#[cfg(target_arch = "wasm32")]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    #[wasm_bindgen_test]
    fn test_wasm_component_rendering() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let component = view! {
                <div>"Hello WASM"</div>
            };
            // Test WASM component rendering
            assert!(component.render().is_ok());
        });
    }

    #[wasm_bindgen_test]
    fn test_wasm_signal_functionality() {
        let runtime = create_runtime();
        let _ = runtime.run_scope(|| {
            let (count, set_count) = create_signal(0);
            assert_eq!(count(), 0);
            set_count(1);
            assert_eq!(count(), 1);
        });
    }
}
