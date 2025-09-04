use leptos_sync_core::crdt::advanced::Rga;
use leptos_sync_core::crdt::{PositionId, ReplicaId, Mergeable};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A collaborative text editor using RGA (Replicated Growable Array)
/// 
/// This implementation provides real-time collaborative text editing capabilities
/// with conflict-free replication using the RGA CRDT.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditor {
    /// The underlying RGA storing the text content
    content: Rga<char>,
    /// Current cursor position
    cursor_position: Option<PositionId>,
    /// User ID for this editor instance
    user_id: Uuid,
    /// Selection range (start and end positions)
    selection: Option<(PositionId, PositionId)>,
    /// History for undo/redo functionality
    history: Vec<EditorState>,
    /// Current history index
    history_index: usize,
    /// Maximum history size
    max_history_size: usize,
}

/// Represents a snapshot of the editor state for undo/redo
#[derive(Debug, Clone, Serialize, Deserialize)]
struct EditorState {
    content: Rga<char>,
    cursor_position: Option<PositionId>,
    selection: Option<(PositionId, PositionId)>,
}

/// Represents a text operation (insert, delete, or replace)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TextOperation {
    Insert { character: char, position: Option<PositionId> },
    Delete { position: PositionId },
    Replace { position: PositionId, new_character: char },
}

/// Represents a cursor update from another user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorUpdate {
    pub user_id: Uuid,
    pub position: Option<PositionId>,
    pub selection: Option<(PositionId, PositionId)>,
}

/// Configuration for the text editor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEditorConfig {
    pub max_history_size: usize,
    pub enable_undo_redo: bool,
    pub enable_selection: bool,
    pub enable_cursor_tracking: bool,
}

impl Default for TextEditorConfig {
    fn default() -> Self {
        Self {
            max_history_size: 100,
            enable_undo_redo: true,
            enable_selection: true,
            enable_cursor_tracking: true,
        }
    }
}

impl TextEditor {
    /// Create a new text editor with default configuration
    pub fn new(user_id: Uuid) -> Self {
        Self::with_config(user_id, TextEditorConfig::default())
    }

    /// Create a new text editor with custom configuration
    pub fn with_config(user_id: Uuid, config: TextEditorConfig) -> Self {
        let replica_id = ReplicaId::default();
        Self {
            content: Rga::new(replica_id),
            cursor_position: None,
            user_id,
            selection: None,
            history: vec![EditorState {
                content: Rga::new(replica_id),
                cursor_position: None,
                selection: None,
            }],
            history_index: 0,
            max_history_size: config.max_history_size,
        }
    }

    /// Get the current text content as a string
    pub fn get_text(&self) -> String {
        self.content.to_vec().into_iter().collect()
    }

    /// Get the length of the text content
    pub fn len(&self) -> usize {
        self.content.len()
    }

    /// Check if the editor is empty
    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    /// Insert a character at the current cursor position
    pub fn insert_char(&mut self, character: char) -> Result<PositionId, String> {
        let position = self.content.insert_after(character, self.cursor_position.clone())
            .map_err(|e| e.to_string())?;
        self.cursor_position = Some(position.clone());
        self.save_state();
        Ok(position)
    }

    /// Insert a character at a specific position
    pub fn insert_char_at(&mut self, character: char, position: Option<PositionId>) -> Result<PositionId, String> {
        let new_position = self.content.insert_after(character, position)
            .map_err(|e| e.to_string())?;
        self.cursor_position = Some(new_position.clone());
        self.save_state();
        Ok(new_position)
    }

    /// Delete the character at the current cursor position
    pub fn delete_char(&mut self) -> Result<(), String> {
        if let Some(pos) = &self.cursor_position {
            self.content.delete(pos).map_err(|e| e.to_string())?;
            // Move cursor to previous position if possible
            self.cursor_position = self.get_previous_position(pos);
            self.save_state();
        }
        Ok(())
    }

    /// Delete a character at a specific position
    pub fn delete_char_at(&mut self, position: PositionId) -> Result<(), String> {
        self.content.delete(&position).map_err(|e| e.to_string())?;
        // Update cursor position if it was affected
        if Some(&position) == self.cursor_position.as_ref() {
            self.cursor_position = self.get_previous_position(&position);
        }
        self.save_state();
        Ok(())
    }

    /// Set the cursor position
    pub fn set_cursor_position(&mut self, position: Option<PositionId>) {
        self.cursor_position = position;
    }

    /// Get the current cursor position
    pub fn get_cursor_position(&self) -> Option<PositionId> {
        self.cursor_position.clone()
    }

    /// Set the selection range
    pub fn set_selection(&mut self, start: PositionId, end: PositionId) {
        self.selection = Some((start, end));
    }

    /// Clear the selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Get the current selection
    pub fn get_selection(&self) -> Option<(PositionId, PositionId)> {
        self.selection.clone()
    }

    /// Move the cursor to the beginning of the document
    pub fn move_cursor_to_beginning(&mut self) {
        // For simplicity, we'll just clear the cursor position
        // In a real implementation, you'd find the first position
        self.cursor_position = None;
    }

    /// Move the cursor to the end of the document
    pub fn move_cursor_to_end(&mut self) {
        // For simplicity, we'll just clear the cursor position
        // In a real implementation, you'd find the last position
        self.cursor_position = None;
    }

    /// Undo the last operation
    pub fn undo(&mut self) -> Result<(), String> {
        if self.history_index > 0 {
            self.history_index -= 1;
            let state = &self.history[self.history_index];
            self.content = state.content.clone();
            self.cursor_position = state.cursor_position.clone();
            self.selection = state.selection.clone();
        }
        Ok(())
    }

    /// Redo the last undone operation
    pub fn redo(&mut self) -> Result<(), String> {
        if self.history_index < self.history.len() - 1 {
            self.history_index += 1;
            let state = &self.history[self.history_index];
            self.content = state.content.clone();
            self.cursor_position = state.cursor_position.clone();
            self.selection = state.selection.clone();
        }
        Ok(())
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        self.history_index > 0
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        self.history_index < self.history.len() - 1
    }

    /// Merge changes from another editor
    pub fn merge(&mut self, other: &TextEditor) -> Result<(), String> {
        self.content.merge(&other.content).map_err(|e| e.to_string())?;
        self.save_state();
        Ok(())
    }

    /// Apply a text operation
    pub fn apply_operation(&mut self, operation: TextOperation) -> Result<(), String> {
        match operation {
            TextOperation::Insert { character, position } => {
                self.insert_char_at(character, position)?;
            }
            TextOperation::Delete { position } => {
                self.delete_char_at(position)?;
            }
            TextOperation::Replace { position, new_character } => {
                self.delete_char_at(position.clone())?;
                self.insert_char_at(new_character, Some(position))?;
            }
        }
        Ok(())
    }

    /// Get all cursor positions for collaborative editing
    pub fn get_cursor_positions(&self) -> Vec<CursorUpdate> {
        vec![CursorUpdate {
            user_id: self.user_id,
            position: self.cursor_position.clone(),
            selection: self.selection.clone(),
        }]
    }

    /// Update cursor position from another user
    pub fn update_cursor(&mut self, update: CursorUpdate) {
        if update.user_id != self.user_id {
            // Store cursor updates from other users
            // This would typically be handled by the UI layer
        }
    }

    /// Find and replace text (simplified implementation)
    pub fn find_and_replace(&mut self, find: &str, replace: &str) -> Result<usize, String> {
        let text = self.get_text();
        if let Some(start) = text.find(find) {
            // Simple implementation: clear and rebuild
            let replica_id = ReplicaId::default();
            self.content = Rga::new(replica_id);
            self.cursor_position = None;
            
            // Insert the replacement text
            let mut last_pos = None;
            for ch in replace.chars() {
                last_pos = Some(self.content.insert_after(ch, last_pos)
                    .map_err(|e| e.to_string())?);
            }
            
            self.save_state();
            Ok(1)
        } else {
            Ok(0)
        }
    }

    /// Get the user ID
    pub fn get_user_id(&self) -> Uuid {
        self.user_id
    }

    /// Get the underlying RGA for advanced operations
    pub fn get_rga(&self) -> &Rga<char> {
        &self.content
    }

    /// Get a mutable reference to the underlying RGA
    pub fn get_rga_mut(&mut self) -> &mut Rga<char> {
        &mut self.content
    }

    // Private helper methods

    fn save_state(&mut self) {
        if self.history_index < self.history.len() - 1 {
            // Remove any states after current index
            self.history.truncate(self.history_index + 1);
        }
        
        let new_state = EditorState {
            content: self.content.clone(),
            cursor_position: self.cursor_position.clone(),
            selection: self.selection.clone(),
        };
        
        self.history.push(new_state);
        
        // Limit history size
        if self.history.len() > self.max_history_size {
            self.history.remove(0);
        } else {
            self.history_index += 1;
        }
    }

    fn get_previous_position(&self, _current_pos: &PositionId) -> Option<PositionId> {
        // Simplified implementation - in reality you'd traverse the RGA structure
        None
    }
}

impl Default for TextEditor {
    fn default() -> Self {
        Self::new(Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_editor_creation() {
        let user_id = Uuid::new_v4();
        let editor = TextEditor::new(user_id);
        assert_eq!(editor.get_user_id(), user_id);
        assert!(editor.is_empty());
        assert_eq!(editor.len(), 0);
    }

    #[test]
    fn test_text_editor_insert_char() {
        let mut editor = TextEditor::new(Uuid::new_v4());
        let pos = editor.insert_char('H').unwrap();
        assert_eq!(editor.get_text(), "H");
        assert_eq!(editor.get_cursor_position(), Some(pos));
    }

    #[test]
    fn test_text_editor_multiple_insertions() {
        let mut editor = TextEditor::new(Uuid::new_v4());
        editor.insert_char('H').unwrap();
        editor.insert_char('e').unwrap();
        editor.insert_char('l').unwrap();
        editor.insert_char('l').unwrap();
        editor.insert_char('o').unwrap();
        assert_eq!(editor.get_text(), "Hello");
    }

    #[test]
    fn test_text_editor_delete_char() {
        let mut editor = TextEditor::new(Uuid::new_v4());
        editor.insert_char('H').unwrap();
        editor.insert_char('e').unwrap();
        editor.insert_char('l').unwrap();
        editor.delete_char().unwrap();
        assert_eq!(editor.get_text(), "He");
    }

    #[test]
    fn test_text_editor_undo_redo() {
        let mut editor = TextEditor::new(Uuid::new_v4());
        editor.insert_char('H').unwrap();
        editor.insert_char('e').unwrap();
        assert_eq!(editor.get_text(), "He");
        
        editor.undo().unwrap();
        assert_eq!(editor.get_text(), "H");
        
        editor.redo().unwrap();
        assert_eq!(editor.get_text(), "He");
    }

    #[test]
    fn test_text_editor_merge() {
        let mut editor1 = TextEditor::new(Uuid::new_v4());
        let mut editor2 = TextEditor::new(Uuid::new_v4());
        
        editor1.insert_char('H').unwrap();
        editor1.insert_char('e').unwrap();
        editor1.insert_char('l').unwrap();
        editor1.insert_char('l').unwrap();
        editor1.insert_char('o').unwrap();
        
        editor2.insert_char('W').unwrap();
        editor2.insert_char('o').unwrap();
        editor2.insert_char('r').unwrap();
        editor2.insert_char('l').unwrap();
        editor2.insert_char('d').unwrap();
        
        editor1.merge(&editor2).unwrap();
        
        let text = editor1.get_text();
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
    }
}