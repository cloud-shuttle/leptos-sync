use leptos_sync_core::crdt::advanced::{YjsTree, YjsTreeNode};
use leptos_sync_core::crdt::{PositionId, ReplicaId, Mergeable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Document node types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Section,
    Paragraph,
    Heading(u8), // Heading level (1-6)
    List,
    CodeBlock,
}

impl std::fmt::Display for NodeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeType::Section => write!(f, "Section"),
            NodeType::Paragraph => write!(f, "Paragraph"),
            NodeType::Heading(level) => write!(f, "Heading {}", level),
            NodeType::List => write!(f, "List"),
            NodeType::CodeBlock => write!(f, "Code Block"),
        }
    }
}

/// Document node representing a piece of content
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentNode {
    /// Unique node identifier
    pub id: Uuid,
    /// Node title (for sections, headings) or content (for paragraphs)
    pub title: String,
    /// Node content (for paragraphs, code blocks)
    pub content: String,
    /// Node type
    pub node_type: NodeType,
    /// List items (for list nodes)
    pub items: Vec<String>,
    /// Code language (for code blocks)
    pub language: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl DocumentNode {
    /// Create a new document node
    pub fn new(title: String, content: String, node_type: NodeType) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            title,
            content,
            node_type,
            items: Vec::new(),
            language: None,
            created_at: now,
            updated_at: now,
        }
    }
    
    /// Create a section node
    pub fn section(title: String) -> Self {
        Self::new(title, String::new(), NodeType::Section)
    }
    
    /// Create a paragraph node
    pub fn paragraph(content: String) -> Self {
        Self::new(String::new(), content, NodeType::Paragraph)
    }
    
    /// Create a heading node
    pub fn heading(title: String, level: u8) -> Self {
        Self::new(title, String::new(), NodeType::Heading(level))
    }
    
    /// Create a list node
    pub fn list(items: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            content: String::new(),
            node_type: NodeType::List,
            items,
            language: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Create a code block node
    pub fn code_block(content: String, language: Option<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            content,
            node_type: NodeType::CodeBlock,
            items: Vec::new(),
            language,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    /// Update the node content
    pub fn update_content(&mut self, content: String) {
        self.content = content;
        self.updated_at = Utc::now();
    }
    
    /// Update the node title
    pub fn update_title(&mut self, title: String) {
        self.title = title;
        self.updated_at = Utc::now();
    }
    
    /// Update list items
    pub fn update_items(&mut self, items: Vec<String>) {
        self.items = items;
        self.updated_at = Utc::now();
    }
}

/// Document update event for real-time collaboration
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentUpdate {
    pub user_id: Uuid,
    pub node_id: Uuid,
    pub position: Option<PositionId>,
    pub update_type: DocumentUpdateType,
}

/// Types of document updates
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentUpdateType {
    NodeCreated,
    NodeUpdated,
    NodeDeleted,
    ContentChanged(String),
    TitleChanged(String),
}

/// Collaborative Document Editor using Yjs Tree CRDT
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentEditor {
    /// Yjs Tree for storing document structure
    tree: YjsTree<DocumentNode>,
    /// User ID for this editor instance
    user_id: Uuid,
    /// Position to node ID mapping for quick lookups
    position_to_node_id: HashMap<PositionId, Uuid>,
    /// Node ID to position mapping for quick lookups
    node_id_to_position: HashMap<Uuid, PositionId>,
}

impl DocumentEditor {
    /// Create a new document editor
    pub fn new(user_id: Uuid) -> Self {
        let replica_id = ReplicaId::default();
        Self {
            tree: YjsTree::new(replica_id),
            user_id,
            position_to_node_id: HashMap::new(),
            node_id_to_position: HashMap::new(),
        }
    }
    
    /// Add a root section
    pub fn add_section(&mut self, title: String) -> Result<PositionId, DocumentError> {
        let node = DocumentNode::section(title);
        let node_id = node.id;
        let position = self.tree.add_root(node).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
        
        self.position_to_node_id.insert(position.clone(), node_id);
        self.node_id_to_position.insert(node_id, position.clone());
        
        Ok(position)
    }
    
    /// Add a paragraph to a parent node
    pub fn add_paragraph(&mut self, parent_id: &PositionId, content: String) -> Result<PositionId, DocumentError> {
        let node = DocumentNode::paragraph(content);
        let node_id = node.id;
        let position = self.tree.add_child(parent_id, node).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
        
        self.position_to_node_id.insert(position.clone(), node_id);
        self.node_id_to_position.insert(node_id, position.clone());
        
        Ok(position)
    }
    
    /// Add a heading to a parent node
    pub fn add_heading(&mut self, parent_id: &PositionId, title: String, level: u8) -> Result<PositionId, DocumentError> {
        let node = DocumentNode::heading(title, level);
        let node_id = node.id;
        let position = self.tree.add_child(parent_id, node).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
        
        self.position_to_node_id.insert(position.clone(), node_id);
        self.node_id_to_position.insert(node_id, position.clone());
        
        Ok(position)
    }
    
    /// Add a list to a parent node
    pub fn add_list(&mut self, parent_id: &PositionId, items: Vec<String>) -> Result<PositionId, DocumentError> {
        let node = DocumentNode::list(items);
        let node_id = node.id;
        let position = self.tree.add_child(parent_id, node).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
        
        self.position_to_node_id.insert(position.clone(), node_id);
        self.node_id_to_position.insert(node_id, position.clone());
        
        Ok(position)
    }
    
    /// Add a code block to a parent node
    pub fn add_code_block(&mut self, parent_id: &PositionId, content: String, language: Option<String>) -> Result<PositionId, DocumentError> {
        let node = DocumentNode::code_block(content, language);
        let node_id = node.id;
        let position = self.tree.add_child(parent_id, node).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
        
        self.position_to_node_id.insert(position.clone(), node_id);
        self.node_id_to_position.insert(node_id, position.clone());
        
        Ok(position)
    }
    
    /// Update paragraph content
    pub fn update_paragraph_content(&mut self, position: &PositionId, content: String) -> Result<(), DocumentError> {
        // Find the node and update it
        if let Some(node_id) = self.position_to_node_id.get(position).copied() {
            // For now, we'll need to delete and recreate the node
            // In a more sophisticated implementation, we'd update the node in place
            self.tree.delete(position).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
            
            // Find the parent and recreate the node
            if let Some(parent_position) = self.find_parent_position(position) {
                let new_position = self.add_paragraph(&parent_position, content)?;
                // Update mappings
                self.position_to_node_id.remove(position);
                self.node_id_to_position.remove(&node_id);
                self.position_to_node_id.insert(new_position.clone(), node_id);
                self.node_id_to_position.insert(node_id, new_position);
            }
        }
        Ok(())
    }
    
    /// Delete a node
    pub fn delete_node(&mut self, position: &PositionId) -> Result<(), DocumentError> {
        if let Some(node_id) = self.position_to_node_id.get(position).copied() {
            self.tree.delete(position).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
            self.position_to_node_id.remove(position);
            self.node_id_to_position.remove(&node_id);
        }
        Ok(())
    }
    
    /// Get the document tree structure
    pub fn get_document_tree(&self) -> Option<YjsTreeNode<DocumentNode>> {
        self.tree.to_tree()
    }
    
    /// Get a flat list of all visible nodes
    pub fn get_flat_structure(&self) -> Vec<DocumentNode> {
        let mut result = Vec::new();
        if let Some(tree) = self.get_document_tree() {
            self.collect_nodes_recursive(&tree, &mut result);
        }
        result
    }
    
    /// Recursively collect all nodes from the tree
    fn collect_nodes_recursive(&self, node: &YjsTreeNode<DocumentNode>, result: &mut Vec<DocumentNode>) {
        result.push(node.value.clone());
        for child in &node.children {
            self.collect_nodes_recursive(child, result);
        }
    }
    
    /// Find the parent position of a given position
    fn find_parent_position(&self, position: &PositionId) -> Option<PositionId> {
        // This is a simplified implementation
        // In a real implementation, we'd traverse the tree to find the parent
        None
    }
    
    /// Get node count (only visible nodes)
    pub fn len(&self) -> usize {
        // For now, use the tree's len method
        // In a more sophisticated implementation, we'd filter for visible nodes
        self.tree.len()
    }
    
    /// Check if document editor is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    
    /// Get user ID
    pub fn get_user_id(&self) -> Uuid {
        self.user_id
    }
    
    /// Get document updates for real-time collaboration
    pub fn get_document_updates(&self) -> Vec<DocumentUpdate> {
        // Simplified implementation - in a real app, this would track actual updates
        vec![]
    }
    
    /// Get the first available position (for demo purposes)
    pub fn get_first_position(&self) -> Option<PositionId> {
        self.position_to_node_id.keys().next().cloned()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    #[error("Yjs Tree error: {0}")]
    YjsTreeError(String),
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

impl Mergeable for DocumentEditor {
    type Error = DocumentError;
    
    fn merge(&mut self, other: &Self) -> Result<(), Self::Error> {
        // Merge the underlying Yjs Tree
        self.tree.merge(&other.tree).map_err(|e| DocumentError::YjsTreeError(e.to_string()))?;
        
        // Rebuild position mappings
        self.position_to_node_id.clear();
        self.node_id_to_position.clear();
        
        // Rebuild mappings from the merged tree
        if let Some(tree) = self.get_document_tree() {
            self.rebuild_mappings_recursive(&tree);
        }
        
        Ok(())
    }
    
    fn has_conflict(&self, other: &Self) -> bool {
        self.tree.has_conflict(&other.tree)
    }
}

impl DocumentEditor {
    /// Rebuild position mappings recursively
    fn rebuild_mappings_recursive(&mut self, node: &YjsTreeNode<DocumentNode>) {
        self.position_to_node_id.insert(node.id.clone(), node.value.id);
        self.node_id_to_position.insert(node.value.id, node.id.clone());
        
        for child in &node.children {
            self.rebuild_mappings_recursive(child);
        }
    }
}
