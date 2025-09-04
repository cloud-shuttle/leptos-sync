//! Integration tests for Document Editor Demo using Yjs Tree CRDT

use leptos_sync_core::crdt::tree::YjsTree;
use leptos_sync_core::crdt::{Mergeable, ReplicaId};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
struct DocumentNode {
    id: Uuid,
    title: String,
    content: String,
    node_type: NodeType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NodeType {
    Section,
    Paragraph,
    Heading,
    List,
    CodeBlock,
}

impl DocumentNode {
    fn new(title: String, content: String, node_type: NodeType) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            node_type,
        }
    }
}

#[test]
fn test_document_editor_yjs_tree_integration() {
    // Test basic Yjs Tree functionality for document editing
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut tree1 = YjsTree::new(replica1);
    let mut tree2 = YjsTree::new(replica2);
    
    // Create document structure from replica1
    let section1 = DocumentNode::new(
        "Introduction".to_string(),
        "".to_string(),
        NodeType::Section,
    );
    let paragraph1 = DocumentNode::new(
        "".to_string(),
        "This is the introduction paragraph.".to_string(),
        NodeType::Paragraph,
    );
    
    let pos1 = tree1.insert_after(None, section1.clone()).unwrap();
    let pos2 = tree1.insert_after(Some(pos1.clone()), paragraph1.clone()).unwrap();
    
    // Create document structure from replica2
    let section2 = DocumentNode::new(
        "Conclusion".to_string(),
        "".to_string(),
        NodeType::Section,
    );
    let paragraph2 = DocumentNode::new(
        "".to_string(),
        "This is the conclusion paragraph.".to_string(),
        NodeType::Paragraph,
    );
    
    let pos3 = tree2.insert_after(None, section2.clone()).unwrap();
    let pos4 = tree2.insert_after(Some(pos3.clone()), paragraph2.clone()).unwrap();
    
    // Merge both ways
    tree1.merge(&tree2).unwrap();
    tree2.merge(&tree1).unwrap();
    
    // Both should have the same content
    let nodes1: Vec<DocumentNode> = tree1.to_vec().into_iter().collect();
    let nodes2: Vec<DocumentNode> = tree2.to_vec().into_iter().collect();
    
    assert_eq!(nodes1.len(), 4);
    assert_eq!(nodes2.len(), 4);
    assert_eq!(nodes1, nodes2);
}

#[test]
fn test_document_editor_hierarchical_structure() {
    // Test hierarchical document structure
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut tree = YjsTree::new(replica);
    
    // Create a hierarchical document
    let section = DocumentNode::new(
        "Chapter 1".to_string(),
        "".to_string(),
        NodeType::Section,
    );
    let heading = DocumentNode::new(
        "Introduction".to_string(),
        "".to_string(),
        NodeType::Heading,
    );
    let paragraph = DocumentNode::new(
        "".to_string(),
        "This is the first paragraph.".to_string(),
        NodeType::Paragraph,
    );
    let code_block = DocumentNode::new(
        "".to_string(),
        "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
        NodeType::CodeBlock,
    );
    
    let pos1 = tree.insert_after(None, section.clone()).unwrap();
    let pos2 = tree.insert_after(Some(pos1.clone()), heading.clone()).unwrap();
    let pos3 = tree.insert_after(Some(pos2.clone()), paragraph.clone()).unwrap();
    let pos4 = tree.insert_after(Some(pos3.clone()), code_block.clone()).unwrap();
    
    // Verify the structure
    let nodes: Vec<DocumentNode> = tree.to_vec().into_iter().collect();
    assert_eq!(nodes.len(), 4);
    
    let titles: Vec<String> = nodes.iter().map(|n| n.title.clone()).collect();
    assert!(titles.contains(&"Chapter 1".to_string()));
    assert!(titles.contains(&"Introduction".to_string()));
}

#[test]
fn test_document_editor_concurrent_edits() {
    // Test concurrent editing of the same document
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut tree1 = YjsTree::new(replica1);
    let mut tree2 = YjsTree::new(replica2);
    
    // Both start with the same document
    let section = DocumentNode::new(
        "Shared Section".to_string(),
        "".to_string(),
        NodeType::Section,
    );
    let pos1 = tree1.insert_after(None, section.clone()).unwrap();
    tree2.merge(&tree1).unwrap();
    
    // Concurrent edits: both add content to the same section
    let paragraph1 = DocumentNode::new(
        "".to_string(),
        "Content from replica1".to_string(),
        NodeType::Paragraph,
    );
    let paragraph2 = DocumentNode::new(
        "".to_string(),
        "Content from replica2".to_string(),
        NodeType::Paragraph,
    );
    
    let pos2 = tree1.insert_after(Some(pos1.clone()), paragraph1.clone()).unwrap();
    let pos3 = tree2.insert_after(Some(pos1.clone()), paragraph2.clone()).unwrap();
    
    // Merge both ways
    tree1.merge(&tree2).unwrap();
    tree2.merge(&tree1).unwrap();
    
    // Both should have the same content
    let nodes1: Vec<DocumentNode> = tree1.to_vec().into_iter().collect();
    let nodes2: Vec<DocumentNode> = tree2.to_vec().into_iter().collect();
    
    assert_eq!(nodes1.len(), 3); // Section + 2 paragraphs
    assert_eq!(nodes2.len(), 3);
    assert_eq!(nodes1, nodes2);
}

#[test]
fn test_document_editor_node_updates() {
    // Test updating existing document nodes
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut tree = YjsTree::new(replica);
    
    // Add a paragraph
    let mut paragraph = DocumentNode::new(
        "".to_string(),
        "Original content".to_string(),
        NodeType::Paragraph,
    );
    let pos = tree.insert_after(None, paragraph.clone()).unwrap();
    
    // Update the paragraph content
    paragraph.content = "Updated content".to_string();
    tree.update(pos.clone(), paragraph.clone()).unwrap();
    
    // Verify the update
    let nodes: Vec<DocumentNode> = tree.to_vec().into_iter().collect();
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0].content, "Updated content");
}

#[test]
fn test_document_editor_node_deletion() {
    // Test deleting document nodes
    let replica = ReplicaId::from(Uuid::new_v4());
    let mut tree = YjsTree::new(replica);
    
    // Add multiple nodes
    let section = DocumentNode::new(
        "Section".to_string(),
        "".to_string(),
        NodeType::Section,
    );
    let paragraph1 = DocumentNode::new(
        "".to_string(),
        "Paragraph 1".to_string(),
        NodeType::Paragraph,
    );
    let paragraph2 = DocumentNode::new(
        "".to_string(),
        "Paragraph 2".to_string(),
        NodeType::Paragraph,
    );
    
    let pos1 = tree.insert_after(None, section.clone()).unwrap();
    let pos2 = tree.insert_after(Some(pos1.clone()), paragraph1.clone()).unwrap();
    let pos3 = tree.insert_after(Some(pos2.clone()), paragraph2.clone()).unwrap();
    
    // Delete the first paragraph
    tree.delete(pos2).unwrap();
    
    // Verify deletion
    let nodes: Vec<DocumentNode> = tree.to_vec().into_iter().collect();
    assert_eq!(nodes.len(), 2);
    
    let contents: Vec<String> = nodes.iter().map(|n| n.content.clone()).collect();
    assert!(!contents.contains(&"Paragraph 1".to_string()));
    assert!(contents.contains(&"Paragraph 2".to_string()));
}

#[test]
fn test_document_editor_merge_with_deletions() {
    // Test merging when one replica has deletions
    let replica1 = ReplicaId::from(Uuid::new_v4());
    let replica2 = ReplicaId::from(Uuid::new_v4());
    
    let mut tree1 = YjsTree::new(replica1);
    let mut tree2 = YjsTree::new(replica2);
    
    // Both start with the same document
    let section = DocumentNode::new(
        "Section".to_string(),
        "".to_string(),
        NodeType::Section,
    );
    let paragraph1 = DocumentNode::new(
        "".to_string(),
        "Paragraph 1".to_string(),
        NodeType::Paragraph,
    );
    let paragraph2 = DocumentNode::new(
        "".to_string(),
        "Paragraph 2".to_string(),
        NodeType::Paragraph,
    );
    
    let pos1 = tree1.insert_after(None, section.clone()).unwrap();
    let pos2 = tree1.insert_after(Some(pos1.clone()), paragraph1.clone()).unwrap();
    let pos3 = tree1.insert_after(Some(pos2.clone()), paragraph2.clone()).unwrap();
    
    tree2.merge(&tree1).unwrap();
    
    // Replica1 deletes a paragraph, replica2 adds a new one
    tree1.delete(pos2).unwrap();
    let paragraph3 = DocumentNode::new(
        "".to_string(),
        "Paragraph 3".to_string(),
        NodeType::Paragraph,
    );
    let pos4 = tree2.insert_after(Some(pos3.clone()), paragraph3.clone()).unwrap();
    
    // Merge both ways
    tree1.merge(&tree2).unwrap();
    tree2.merge(&tree1).unwrap();
    
    // Both should have the same final state
    let nodes1: Vec<DocumentNode> = tree1.to_vec().into_iter().collect();
    let nodes2: Vec<DocumentNode> = tree2.to_vec().into_iter().collect();
    
    assert_eq!(nodes1.len(), 3); // Section + Paragraph 2 + Paragraph 3
    assert_eq!(nodes2.len(), 3);
    assert_eq!(nodes1, nodes2);
}
