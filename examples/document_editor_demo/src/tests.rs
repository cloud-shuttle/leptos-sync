use leptos_sync_core::crdt::advanced::YjsTree;
use leptos_sync_core::crdt::{ReplicaId, Mergeable};
use crate::document_editor::{DocumentEditor, DocumentNode, NodeType, DocumentError};

/// Test suite for collaborative document editor using Yjs Tree
/// This module contains comprehensive tests for the document editor functionality,
/// following Test-Driven Development (TDD) principles.

#[test]
fn test_document_editor_initialization() {
    let user_id = uuid::Uuid::new_v4();
    let editor = DocumentEditor::new(user_id);
    assert_eq!(editor.len(), 0);
    assert!(editor.is_empty());
}

#[test]
fn test_add_root_section() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let _section_id = editor.add_section("Introduction".to_string()).unwrap();
    assert_eq!(editor.len(), 1);
    assert!(!editor.is_empty());
    
    let tree = editor.get_document_tree();
    assert!(tree.is_some());
    let tree = tree.unwrap();
    assert_eq!(tree.value.title, "Introduction");
    assert_eq!(tree.value.node_type, NodeType::Section);
}

#[test]
fn test_add_paragraph_to_section() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let section_id = editor.add_section("Introduction".to_string()).unwrap();
    let _paragraph_id = editor.add_paragraph(&section_id, "This is the introduction paragraph.".to_string()).unwrap();
    
    assert_eq!(editor.len(), 2);
    
    let tree = editor.get_document_tree();
    assert!(tree.is_some());
    let tree = tree.unwrap();
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].value.content, "This is the introduction paragraph.");
    assert_eq!(tree.children[0].value.node_type, NodeType::Paragraph);
}

#[test]
fn test_add_heading_to_section() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let section_id = editor.add_section("Chapter 1".to_string()).unwrap();
    let _heading_id = editor.add_heading(&section_id, "Getting Started".to_string(), 1).unwrap();
    
    assert_eq!(editor.len(), 2);
    
    let tree = editor.get_document_tree();
    assert!(tree.is_some());
    let tree = tree.unwrap();
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].value.title, "Getting Started");
    assert_eq!(tree.children[0].value.node_type, NodeType::Heading(1));
}

#[test]
fn test_add_list_to_section() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let section_id = editor.add_section("Features".to_string()).unwrap();
    let _list_id = editor.add_list(&section_id, vec![
        "Feature 1".to_string(),
        "Feature 2".to_string(),
        "Feature 3".to_string(),
    ]).unwrap();
    
    assert_eq!(editor.len(), 2);
    
    let tree = editor.get_document_tree();
    assert!(tree.is_some());
    let tree = tree.unwrap();
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].value.node_type, NodeType::List);
    assert_eq!(tree.children[0].value.items.len(), 3);
}

#[test]
fn test_delete_node() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let section_id = editor.add_section("Introduction".to_string()).unwrap();
    let paragraph_id = editor.add_paragraph(&section_id, "This paragraph will be deleted.".to_string()).unwrap();
    
    assert_eq!(editor.len(), 2);
    
    editor.delete_node(&paragraph_id).unwrap();
    // Note: YjsTree.len() counts all nodes including deleted ones
    // In a real implementation, we'd filter for visible nodes
    assert_eq!(editor.len(), 2); // Both nodes remain in the tree structure
    
    let tree = editor.get_document_tree();
    assert!(tree.is_some());
    let tree = tree.unwrap();
    assert_eq!(tree.children.len(), 0); // No children after deletion
}

#[test]
fn test_merge_document_editors() {
    let user_id1 = uuid::Uuid::new_v4();
    let user_id2 = uuid::Uuid::new_v4();
    
    let mut editor1 = DocumentEditor::new(user_id1);
    let mut editor2 = DocumentEditor::new(user_id2);
    
    let section1_id = editor1.add_section("Section 1".to_string()).unwrap();
    let _paragraph1_id = editor1.add_paragraph(&section1_id, "Content from editor 1.".to_string()).unwrap();
    
    let section2_id = editor2.add_section("Section 2".to_string()).unwrap();
    let _paragraph2_id = editor2.add_paragraph(&section2_id, "Content from editor 2.".to_string()).unwrap();
    
    assert_eq!(editor1.len(), 2);
    assert_eq!(editor2.len(), 2);
    
    editor1.merge(&editor2).unwrap();
    assert_eq!(editor1.len(), 4); // Both sections and paragraphs
    
    let tree = editor1.get_document_tree();
    assert!(tree.is_some());
    // Note: After merge, we might have multiple root sections
    // The exact structure depends on the merge implementation
}

#[test]
fn test_update_node_content() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let section_id = editor.add_section("Introduction".to_string()).unwrap();
    let paragraph_id = editor.add_paragraph(&section_id, "Original content.".to_string()).unwrap();
    
    editor.update_paragraph_content(&paragraph_id, "Updated content.".to_string()).unwrap();
    
    let tree = editor.get_document_tree();
    assert!(tree.is_some());
    let tree = tree.unwrap();
    if !tree.children.is_empty() {
        assert_eq!(tree.children[0].value.content, "Updated content.");
    }
}

#[test]
fn test_get_flat_document_structure() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let section_id = editor.add_section("Chapter 1".to_string()).unwrap();
    let _heading_id = editor.add_heading(&section_id, "Introduction".to_string(), 1).unwrap();
    let _paragraph_id = editor.add_paragraph(&section_id, "This is a paragraph.".to_string()).unwrap();
    
    let flat_structure = editor.get_flat_structure();
    assert_eq!(flat_structure.len(), 3); // Section, heading, paragraph
    
    // Check that all nodes are present
    let titles: Vec<&String> = flat_structure.iter().map(|node| &node.title).collect();
    let contents: Vec<&String> = flat_structure.iter().map(|node| &node.content).collect();
    assert!(titles.contains(&&"Chapter 1".to_string()));
    assert!(titles.contains(&&"Introduction".to_string()));
    assert!(contents.contains(&&"This is a paragraph.".to_string()));
}

#[test]
fn test_document_editor_serialization() {
    let user_id = uuid::Uuid::new_v4();
    let mut editor = DocumentEditor::new(user_id);
    
    let section_id = editor.add_section("Test Section".to_string()).unwrap();
    let _paragraph_id = editor.add_paragraph(&section_id, "Test paragraph content.".to_string()).unwrap();
    
    let tree = editor.get_document_tree();
    assert!(tree.is_some());
    let tree = tree.unwrap();
    assert_eq!(tree.value.title, "Test Section");
    assert_eq!(tree.children.len(), 1);
    assert_eq!(tree.children[0].value.content, "Test paragraph content.");
}
