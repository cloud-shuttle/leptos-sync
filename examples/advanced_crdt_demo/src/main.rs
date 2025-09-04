use leptos_sync_core::{
    crdt::{
        Rga, Lseq, YjsTree, Dag, ReplicaId, Mergeable,
    },
    DevTools, DevToolsConfig, CrdtInspector,
};
use std::sync::Arc;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Advanced CRDT Types Demo");
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

    // Demo 1: RGA (Replicated Growable Array) for Collaborative Text Editing
    println!("📝 Demo 1: RGA - Collaborative Text Editing");
    println!("---------------------------------------------");
    
    let replica_id1 = ReplicaId::from(Uuid::new_v4());
    let replica_id2 = ReplicaId::from(Uuid::new_v4());
    
    let mut rga1 = Rga::<String>::new(replica_id1.clone());
    let mut rga2 = Rga::<String>::new(replica_id2.clone());
    
    // User 1 types "Hello"
    let pos1 = rga1.insert_after("Hello".to_string(), None)?;
    let pos2 = rga1.insert_after(" ".to_string(), Some(pos1.clone()))?;
    let pos3 = rga1.insert_after("World".to_string(), Some(pos2.clone()))?;
    
    // User 2 types "Beautiful" at the same time
    let pos4 = rga2.insert_after("Beautiful".to_string(), None)?;
    let pos5 = rga2.insert_after(" ".to_string(), Some(pos4.clone()))?;
    let pos6 = rga2.insert_after("Day".to_string(), Some(pos5.clone()))?;
    
    println!("User 1's text: {:?}", rga1.to_vec());
    println!("User 2's text: {:?}", rga2.to_vec());
    
    // Merge the changes
    rga1.merge(&rga2)?;
    println!("Merged text: {:?}", rga1.to_vec());
    
    // Record operations
    devtools.record_crdt_operation("rga-demo".to_string(), "insert".to_string(), replica_id1).await;
    devtools.record_crdt_operation("rga-demo".to_string(), "merge".to_string(), replica_id2).await;
    println!();

    // Demo 2: LSEQ (Logoot Sequence) for Ordered Lists
    println!("📋 Demo 2: LSEQ - Ordered Lists");
    println!("-------------------------------");
    
    let replica_id3 = ReplicaId::from(Uuid::new_v4());
    let replica_id4 = ReplicaId::from(Uuid::new_v4());
    
    let mut lseq1 = Lseq::<String>::new(replica_id3.clone());
    let mut lseq2 = Lseq::<String>::new(replica_id4.clone());
    
    // User 1 creates a todo list
    lseq1.insert("Buy groceries".to_string(), None)?;
    lseq1.insert("Walk the dog".to_string(), None)?;
    lseq1.insert("Finish project".to_string(), None)?;
    
    // User 2 adds items to the same list
    lseq2.insert("Call mom".to_string(), None)?;
    lseq2.insert("Exercise".to_string(), None)?;
    
    println!("User 1's todos: {:?}", lseq1.to_vec());
    println!("User 2's todos: {:?}", lseq2.to_vec());
    
    // Merge the lists
    lseq1.merge(&lseq2)?;
    println!("Merged todos: {:?}", lseq1.to_vec());
    
    // Mark some items as done (delete them)
    let items = lseq1.to_vec();
    if let Some(first_item) = items.first() {
        // Find and delete the first item
        let mut pos_to_delete = None;
        for (pos, element) in lseq1.get_elements() {
            if element.value == *first_item {
                pos_to_delete = Some(pos.clone());
                break;
            }
        }
        if let Some(pos) = pos_to_delete {
            lseq1.delete(&pos)?;
        }
    }
    
    println!("After completing first item: {:?}", lseq1.to_vec());
    
    // Record operations
    devtools.record_crdt_operation("lseq-demo".to_string(), "insert".to_string(), replica_id3).await;
    devtools.record_crdt_operation("lseq-demo".to_string(), "delete".to_string(), replica_id4).await;
    println!();

    // Demo 3: Yjs-style Tree for Hierarchical Data
    println!("🌳 Demo 3: Yjs Tree - Hierarchical Data");
    println!("---------------------------------------");
    
    let replica_id5 = ReplicaId::from(Uuid::new_v4());
    let replica_id6 = ReplicaId::from(Uuid::new_v4());
    
    let mut tree1 = YjsTree::<String>::new(replica_id5.clone());
    let mut tree2 = YjsTree::<String>::new(replica_id6.clone());
    
    // User 1 creates a document structure
    let root_id1 = tree1.add_root("Document".to_string())?;
    let chapter1_id = tree1.add_child(&root_id1, "Chapter 1: Introduction".to_string())?;
    let section1_id = tree1.add_child(&chapter1_id, "Section 1.1: Overview".to_string())?;
    let section2_id = tree1.add_child(&chapter1_id, "Section 1.2: Goals".to_string())?;
    
    // User 2 creates a different structure
    let root_id2 = tree2.add_root("Document".to_string())?;
    let chapter2_id = tree2.add_child(&root_id2, "Chapter 2: Implementation".to_string())?;
    let section3_id = tree2.add_child(&chapter2_id, "Section 2.1: Architecture".to_string())?;
    
    println!("User 1's document structure:");
    if let Some(tree_structure) = tree1.to_tree() {
        print_tree(&tree_structure, 0);
    }
    
    println!("\nUser 2's document structure:");
    if let Some(tree_structure) = tree2.to_tree() {
        print_tree(&tree_structure, 0);
    }
    
    // Merge the document structures
    tree1.merge(&tree2)?;
    
    println!("\nMerged document structure:");
    if let Some(tree_structure) = tree1.to_tree() {
        print_tree(&tree_structure, 0);
    }
    
    // Record operations
    devtools.record_crdt_operation("yjs-tree-demo".to_string(), "add_root".to_string(), replica_id5).await;
    devtools.record_crdt_operation("yjs-tree-demo".to_string(), "add_child".to_string(), replica_id6).await;
    println!();

    // Demo 4: DAG (Directed Acyclic Graph) for Complex Relationships
    println!("🕸️  Demo 4: DAG - Complex Relationships");
    println!("---------------------------------------");
    
    let replica_id7 = ReplicaId::from(Uuid::new_v4());
    let replica_id8 = ReplicaId::from(Uuid::new_v4());
    
    let mut dag1 = Dag::<String>::new(replica_id7.clone());
    let mut dag2 = Dag::<String>::new(replica_id8.clone());
    
    // User 1 creates a project dependency graph
    let task1_id = dag1.add_node("Design UI".to_string())?;
    let task2_id = dag1.add_node("Implement Backend".to_string())?;
    let task3_id = dag1.add_node("Write Tests".to_string())?;
    let task4_id = dag1.add_node("Deploy".to_string())?;
    
    // Add dependencies
    dag1.add_edge(&task1_id, &task2_id)?; // Design UI before Implement Backend
    dag1.add_edge(&task2_id, &task3_id)?; // Implement Backend before Write Tests
    dag1.add_edge(&task3_id, &task4_id)?; // Write Tests before Deploy
    
    // User 2 adds more tasks
    let task5_id = dag2.add_node("Setup Database".to_string())?;
    let task6_id = dag2.add_node("Create API".to_string())?;
    
    // Add dependencies
    dag2.add_edge(&task5_id, &task6_id)?; // Setup Database before Create API
    
    println!("User 1's project graph:");
    let sorted1 = dag1.topological_sort();
    for (i, pos) in sorted1.iter().enumerate() {
        if let Some(node) = dag1.get_nodes().get(pos) {
            println!("  {}. {}", i + 1, node.value);
        }
    }
    
    println!("\nUser 2's project graph:");
    let sorted2 = dag2.topological_sort();
    for (i, pos) in sorted2.iter().enumerate() {
        if let Some(node) = dag2.get_nodes().get(pos) {
            println!("  {}. {}", i + 1, node.value);
        }
    }
    
    // Merge the project graphs
    dag1.merge(&dag2)?;
    
    println!("\nMerged project graph (topological order):");
    let sorted_merged = dag1.topological_sort();
    for (i, pos) in sorted_merged.iter().enumerate() {
        if let Some(node) = dag1.get_nodes().get(pos) {
            println!("  {}. {}", i + 1, node.value);
        }
    }
    
    // Try to add a cycle (should fail)
    println!("\nTrying to add a cycle (should fail):");
    let result = dag1.add_edge(&task4_id, &task1_id);
    match result {
        Ok(_) => println!("  Cycle added successfully (unexpected!)"),
        Err(e) => println!("  Cycle detection worked: {}", e),
    }
    
    // Record operations
    devtools.record_crdt_operation("dag-demo".to_string(), "add_node".to_string(), replica_id7).await;
    devtools.record_crdt_operation("dag-demo".to_string(), "add_edge".to_string(), replica_id8).await;
    println!();

    // Demo 5: CRDT Inspection and Performance Analysis
    println!("🔍 Demo 5: CRDT Inspection & Performance");
    println!("----------------------------------------");
    
    // Inspect all CRDTs
    let rga_inspection = inspector.inspect_crdt(&rga1, "rga-demo".to_string()).await;
    let lseq_inspection = inspector.inspect_crdt(&lseq1, "lseq-demo".to_string()).await;
    let tree_inspection = inspector.inspect_crdt(&tree1, "yjs-tree-demo".to_string()).await;
    let dag_inspection = inspector.inspect_crdt(&dag1, "dag-demo".to_string()).await;
    
    println!("RGA Inspection:");
    println!("  Type: {}", rga_inspection.type_name);
    println!("  Operations: {}", rga_inspection.operation_count);
    println!("  Memory: {} bytes", rga_inspection.memory_usage_bytes);
    
    println!("LSEQ Inspection:");
    println!("  Type: {}", lseq_inspection.type_name);
    println!("  Operations: {}", lseq_inspection.operation_count);
    println!("  Memory: {} bytes", lseq_inspection.memory_usage_bytes);
    
    println!("Yjs Tree Inspection:");
    println!("  Type: {}", tree_inspection.type_name);
    println!("  Operations: {}", tree_inspection.operation_count);
    println!("  Memory: {} bytes", tree_inspection.memory_usage_bytes);
    
    println!("DAG Inspection:");
    println!("  Type: {}", dag_inspection.type_name);
    println!("  Operations: {}", dag_inspection.operation_count);
    println!("  Memory: {} bytes", dag_inspection.memory_usage_bytes);
    
    // Get overall statistics
    let event_counts = devtools.get_event_counts().await;
    let sync_stats = devtools.get_sync_stats().await;
    
    println!("\nOverall Statistics:");
    println!("  Event counts: {:?}", event_counts);
    println!("  Sync statistics: {:?}", sync_stats);
    
    // Export data
    let export_json = devtools.export_data().await?;
    println!("  Exported data size: {} characters", export_json.len());
    println!();

    println!("✅ Advanced CRDT Types Demo Complete!");
    println!("=====================================");
    println!("This demo showed how to:");
    println!("• Use RGA for collaborative text editing with automatic merging");
    println!("• Use LSEQ for ordered lists with conflict-free operations");
    println!("• Use Yjs-style trees for hierarchical document structures");
    println!("• Use DAGs for complex dependency relationships with cycle detection");
    println!("• Monitor CRDT operations with DevTools");
    println!("• Inspect CRDT state and performance metrics");
    println!("• Export debugging data for analysis");

    Ok(())
}

/// Helper function to print tree structure
fn print_tree<T>(node: &leptos_sync_core::crdt::YjsTreeNode<T>, depth: usize) 
where 
    T: std::fmt::Display 
{
    let indent = "  ".repeat(depth);
    println!("{}{}", indent, node.value);
    
    for child in &node.children {
        print_tree(child, depth + 1);
    }
}
