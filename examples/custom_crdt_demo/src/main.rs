use leptos_sync_core::{
    crdt::{
        CrdtBuilder, CrdtStrategy, CustomCrdt, ReplicaId, Mergeable,
    },
    DevTools, DevToolsConfig, CrdtInspector,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Custom CRDT Builder Demo");
    println!("===========================\n");

    // Create DevTools for monitoring
    let devtools_config = DevToolsConfig {
        enable_crdt_inspection: true,
        enable_sync_monitoring: true,
        enable_transport_monitoring: true,
        enable_performance_metrics: true,
        max_events: 100,
    };
    
    let devtools = Arc::new(DevTools::new(devtools_config));
    let inspector = CrdtInspector::new(devtools.clone());

    // Demo 1: User Profile CRDT
    println!("👤 Demo 1: User Profile CRDT");
    println!("-----------------------------");
    
    let replica_id = ReplicaId::from(Uuid::new_v4());
    
    // Create a custom CRDT for user profiles
    let user_profile_config = CrdtBuilder::new("UserProfile".to_string())
        .add_field("name".to_string(), CrdtStrategy::Lww)
        .add_field("email".to_string(), CrdtStrategy::Lww)
        .add_field("age".to_string(), CrdtStrategy::Lww)
        .add_field("friends".to_string(), CrdtStrategy::AddWins)
        .add_field("blocked_users".to_string(), CrdtStrategy::AddWins)
        .add_field("post_count".to_string(), CrdtStrategy::GCounter)
        .add_optional_field("bio".to_string(), CrdtStrategy::Lww, 
            serde_json::Value::String("No bio yet".to_string()))
        .add_optional_field("preferences".to_string(), CrdtStrategy::Lww,
            serde_json::Value::Object(serde_json::Map::new()))
        .build();
    
    let mut alice_profile = CustomCrdt::new(user_profile_config.clone(), replica_id.clone());
    
    // Set profile data
    alice_profile.set_field("name", serde_json::Value::String("Alice Johnson".to_string()))?;
    alice_profile.set_field("email", serde_json::Value::String("alice@example.com".to_string()))?;
    alice_profile.set_field("age", serde_json::Value::Number(serde_json::Number::from(28)))?;
    alice_profile.set_field("friends", serde_json::Value::Array(vec![
        serde_json::Value::String("Bob".to_string()),
        serde_json::Value::String("Charlie".to_string()),
    ]))?;
    alice_profile.set_field("post_count", serde_json::Value::Number(serde_json::Number::from(15)))?;
    alice_profile.set_field("bio", serde_json::Value::String("Software engineer and cat lover".to_string()))?;
    
    // Record CRDT operations
    devtools.record_crdt_operation("alice-profile".to_string(), "create".to_string(), replica_id.clone()).await;
    devtools.record_crdt_operation("alice-profile".to_string(), "update".to_string(), replica_id.clone()).await;
    
    println!("Alice's Profile:");
    println!("  Name: {}", alice_profile.get_field("name").unwrap());
    println!("  Email: {}", alice_profile.get_field("email").unwrap());
    println!("  Age: {}", alice_profile.get_field("age").unwrap());
    println!("  Bio: {}", alice_profile.get_field("bio").unwrap());
    println!("  Post Count: {}", alice_profile.get_field("post_count").unwrap());
    if let Some(friends) = alice_profile.get_field("friends") {
        if let Some(friends_array) = friends.as_array() {
            println!("  Friends: {}", friends_array.len());
        }
    }
    println!();

    // Demo 2: Collaborative Document CRDT
    println!("📝 Demo 2: Collaborative Document CRDT");
    println!("--------------------------------------");
    
    let doc_replica_id = ReplicaId::from(Uuid::new_v4());
    
    // Create a custom CRDT for collaborative documents
    let document_config = CrdtBuilder::new("CollaborativeDocument".to_string())
        .add_field("title".to_string(), CrdtStrategy::Lww)
        .add_field("content".to_string(), CrdtStrategy::Lww)
        .add_field("tags".to_string(), CrdtStrategy::AddWins)
        .add_field("collaborators".to_string(), CrdtStrategy::AddWins)
        .add_field("view_count".to_string(), CrdtStrategy::GCounter)
        .add_field("edit_count".to_string(), CrdtStrategy::GCounter)
        .add_optional_field("metadata".to_string(), CrdtStrategy::Lww,
            serde_json::Value::Object(serde_json::Map::new()))
        .build();
    
    let mut document = CustomCrdt::new(document_config.clone(), doc_replica_id.clone());
    
    // Set document data
    document.set_field("title", serde_json::Value::String("My Awesome Blog Post".to_string()))?;
    document.set_field("content", serde_json::Value::String("This is the content of my blog post...".to_string()))?;
    document.set_field("tags", serde_json::Value::Array(vec![
        serde_json::Value::String("rust".to_string()),
        serde_json::Value::String("programming".to_string()),
        serde_json::Value::String("crdt".to_string()),
    ]))?;
    document.set_field("collaborators", serde_json::Value::Array(vec![
        serde_json::Value::String("alice".to_string()),
        serde_json::Value::String("bob".to_string()),
    ]))?;
    document.set_field("view_count", serde_json::Value::Number(serde_json::Number::from(42)))?;
    document.set_field("edit_count", serde_json::Value::Number(serde_json::Number::from(5)))?;
    
    // Record operations
    devtools.record_crdt_operation("document-1".to_string(), "create".to_string(), doc_replica_id.clone()).await;
    devtools.record_crdt_operation("document-1".to_string(), "edit".to_string(), doc_replica_id.clone()).await;
    
    println!("Document:");
    println!("  Title: {}", document.get_field("title").unwrap());
    println!("  Content: {}", document.get_field("content").unwrap());
    println!("  View Count: {}", document.get_field("view_count").unwrap());
    println!("  Edit Count: {}", document.get_field("edit_count").unwrap());
    if let Some(tags) = document.get_field("tags") {
        if let Some(tags_array) = tags.as_array() {
            println!("  Tags: {}", tags_array.len());
        }
    }
    println!();

    // Demo 3: CRDT Merging
    println!("🔄 Demo 3: CRDT Merging");
    println!("-----------------------");
    
    // Create a second user profile (Bob)
    let bob_replica_id = ReplicaId::from(Uuid::new_v4());
    let mut bob_profile = CustomCrdt::new(user_profile_config, bob_replica_id.clone());
    
    // Set Bob's profile data
    bob_profile.set_field("name", serde_json::Value::String("Bob Smith".to_string()))?;
    bob_profile.set_field("email", serde_json::Value::String("bob@example.com".to_string()))?;
    bob_profile.set_field("age", serde_json::Value::Number(serde_json::Number::from(32)))?;
    bob_profile.set_field("friends", serde_json::Value::Array(vec![
        serde_json::Value::String("Alice".to_string()),
        serde_json::Value::String("David".to_string()),
    ]))?;
    bob_profile.set_field("post_count", serde_json::Value::Number(serde_json::Number::from(23)))?;
    
    // Record Bob's operations
    devtools.record_crdt_operation("bob-profile".to_string(), "create".to_string(), bob_replica_id.clone()).await;
    
    println!("Before merge:");
    println!("  Alice's friends: {:?}", alice_profile.get_field("friends").unwrap());
    println!("  Bob's friends: {:?}", bob_profile.get_field("friends").unwrap());
    
    // Merge Bob's profile into Alice's (simulating sync)
    alice_profile.merge(&bob_profile)?;
    
    println!("After merge:");
    println!("  Alice's friends: {:?}", alice_profile.get_field("friends").unwrap());
    println!("  Alice's post count: {}", alice_profile.get_field("post_count").unwrap());
    
    // Record merge operation
    devtools.record_sync_operation(
        "profile-sync".to_string(),
        "merge".to_string(),
        "success".to_string(),
        Some(150)
    ).await;
    println!();

    // Demo 4: CRDT Inspection
    println!("🔍 Demo 4: CRDT Inspection");
    println!("--------------------------");
    
    // Inspect the CRDTs
    let alice_inspection = inspector.inspect_crdt(&alice_profile, "alice-profile".to_string()).await;
    let document_inspection = inspector.inspect_crdt(&document, "document-1".to_string()).await;
    
    println!("Alice Profile Inspection:");
    println!("  Type: {}", alice_inspection.type_name);
    println!("  Operations: {}", alice_inspection.operation_count);
    println!("  Memory: {} bytes", alice_inspection.memory_usage_bytes);
    
    println!("Document Inspection:");
    println!("  Type: {}", document_inspection.type_name);
    println!("  Operations: {}", document_inspection.operation_count);
    println!("  Memory: {} bytes", document_inspection.memory_usage_bytes);
    println!();

    // Demo 5: Advanced CRDT Strategies
    println!("⚙️  Demo 5: Advanced CRDT Strategies");
    println!("------------------------------------");
    
    let advanced_replica_id = ReplicaId::from(Uuid::new_v4());
    
    // Create a CRDT with multiple strategies
    let advanced_config = CrdtBuilder::new("AdvancedCRDT".to_string())
        .add_field("lww_field".to_string(), CrdtStrategy::Lww)
        .add_field("add_wins_field".to_string(), CrdtStrategy::AddWins)
        .add_field("remove_wins_field".to_string(), CrdtStrategy::RemoveWins)
        .add_field("gcounter_field".to_string(), CrdtStrategy::GCounter)
        .add_field("mv_register_field".to_string(), CrdtStrategy::MvRegister)
        .build();
    
    let mut advanced_crdt = CustomCrdt::new(advanced_config, advanced_replica_id);
    
    // Test different strategies
    advanced_crdt.set_field("lww_field", serde_json::Value::String("initial".to_string()))?;
    advanced_crdt.set_field("add_wins_field", serde_json::Value::Array(vec![
        serde_json::Value::String("item1".to_string()),
    ]))?;
    advanced_crdt.set_field("gcounter_field", serde_json::Value::Number(serde_json::Number::from(10)))?;
    advanced_crdt.set_field("mv_register_field", serde_json::Value::Array(vec![
        serde_json::Value::String("value1".to_string()),
    ]))?;
    
    println!("Advanced CRDT Strategies:");
    println!("  LWW Field: {}", advanced_crdt.get_field("lww_field").unwrap());
    println!("  Add-Wins Field: {:?}", advanced_crdt.get_field("add_wins_field").unwrap());
    println!("  G-Counter Field: {}", advanced_crdt.get_field("gcounter_field").unwrap());
    println!("  MV-Register Field: {:?}", advanced_crdt.get_field("mv_register_field").unwrap());
    println!();

    // Demo 6: DevTools Export
    println!("📊 Demo 6: DevTools Export");
    println!("--------------------------");
    
    let export_json = devtools.export_data().await?;
    println!("Exported data size: {} characters", export_json.len());
    
    // Get event counts
    let event_counts = devtools.get_event_counts().await;
    println!("Event counts: {:?}", event_counts);
    
    // Get sync statistics
    let sync_stats = devtools.get_sync_stats().await;
    println!("Sync statistics: {:?}", sync_stats);
    println!();

    println!("✅ Custom CRDT Builder Demo Complete!");
    println!("=====================================");
    println!("This demo showed how to:");
    println!("• Create custom CRDT types using the builder pattern");
    println!("• Use different CRDT strategies (LWW, AddWins, GCounter, etc.)");
    println!("• Merge CRDTs with automatic conflict resolution");
    println!("• Monitor CRDT operations with DevTools");
    println!("• Inspect CRDT state and performance");
    println!("• Export debugging data for analysis");

    Ok(())
}
