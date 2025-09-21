//! End-to-end synchronization demo

use leptos_sync_core::{
    crdt::ReplicaId,
    storage::memory::MemoryStorage,
    transport::memory::InMemoryTransport,
    sync::{EndToEndSyncManager, CollectionMetadata},
};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("🚀 Starting End-to-End Synchronization Demo");
    println!("=============================================");

    // Create two replica managers to simulate peer-to-peer sync
    let replica1_id = ReplicaId::new("replica1".to_string());
    let replica2_id = ReplicaId::new("replica2".to_string());

    // Create storage and transport for replica 1
    let storage1 = Arc::new(MemoryStorage::new());
    let transport1 = Arc::new(InMemoryTransport::new());

    // Create storage and transport for replica 2
    let storage2 = Arc::new(MemoryStorage::new());
    let transport2 = Arc::new(InMemoryTransport::new());

    // Create sync managers
    let manager1 = EndToEndSyncManager::new(
        replica1_id.clone(),
        storage1.clone(),
        transport1.clone(),
        Duration::from_secs(2),
        Duration::from_secs(10),
    );

    let manager2 = EndToEndSyncManager::new(
        replica2_id.clone(),
        storage2.clone(),
        transport2.clone(),
        Duration::from_secs(2),
        Duration::from_secs(10),
    );

    // Add a collection to both managers
    let collection_metadata = CollectionMetadata {
        id: "demo_collection".to_string(),
        name: "Demo Collection".to_string(),
        crdt_type: "LwwMap".to_string(),
        version: 1,
        last_sync: 0,
        replica_count: 2,
    };

    manager1.add_collection(collection_metadata.clone()).await?;
    manager2.add_collection(collection_metadata).await?;

    println!("✅ Created sync managers and collections");

    // Start both managers
    manager1.start().await?;
    manager2.start().await?;

    println!("✅ Started sync managers");

    // Add some initial data to replica 1
    let initial_data = b"Hello from replica 1!".to_vec();
    storage1.set("demo_collection", &initial_data).await?;
    println!("📝 Added initial data to replica 1: {:?}", String::from_utf8_lossy(&initial_data));

    // Wait for sync
    sleep(Duration::from_secs(3)).await;

    // Check if data was synced to replica 2
    if let Some(synced_data) = storage2.get::<Vec<u8>>("demo_collection").await? {
        println!("🔄 Data synced to replica 2: {:?}", String::from_utf8_lossy(&synced_data));
    } else {
        println!("❌ Data not synced to replica 2");
    }

    // Add some data to replica 2
    let replica2_data = b"Hello from replica 2!".to_vec();
    storage2.set("demo_collection", &replica2_data).await?;
    println!("📝 Added data to replica 2: {:?}", String::from_utf8_lossy(&replica2_data));

    // Wait for sync
    sleep(Duration::from_secs(3)).await;

    // Check if data was synced to replica 1
    if let Some(synced_data) = storage1.get::<Vec<u8>>("demo_collection").await? {
        println!("🔄 Data synced to replica 1: {:?}", String::from_utf8_lossy(&synced_data));
    } else {
        println!("❌ Data not synced to replica 1");
    }

    // Show peer information
    let peers1 = manager1.list_peers().await?;
    let peers2 = manager2.list_peers().await?;

    println!("👥 Peers for replica 1: {}", peers1.len());
    for peer in peers1 {
        println!("   - {}: {:?}", peer.id.id, peer.status);
    }

    println!("👥 Peers for replica 2: {}", peers2.len());
    for peer in peers2 {
        println!("   - {}: {:?}", peer.id.id, peer.status);
    }

    // Show collections
    let collections1 = manager1.list_collections().await?;
    let collections2 = manager2.list_collections().await?;

    println!("📚 Collections for replica 1: {}", collections1.len());
    for collection in collections1 {
        println!("   - {}: {} ({})", collection.id, collection.name, collection.crdt_type);
    }

    println!("📚 Collections for replica 2: {}", collections2.len());
    for collection in collections2 {
        println!("   - {}: {} ({})", collection.id, collection.name, collection.crdt_type);
    }

    // Show sync states
    let state1 = manager1.get_sync_state().await;
    let state2 = manager2.get_sync_state().await;

    println!("🔄 Sync state for replica 1: {:?}", state1);
    println!("🔄 Sync state for replica 2: {:?}", state2);

    // Stop managers
    manager1.stop().await?;
    manager2.stop().await?;

    println!("✅ Stopped sync managers");
    println!("🎉 End-to-end synchronization demo completed successfully!");

    Ok(())
}
