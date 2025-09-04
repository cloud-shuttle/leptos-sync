use leptos_sync_core::crdt::advanced::Rga;
use leptos_sync_core::crdt::{ReplicaId, Mergeable};
use uuid::Uuid;

/// Test suite for collaborative text editor using RGA
/// 
/// This module contains comprehensive tests for the text editor functionality,
/// following Test-Driven Development (TDD) principles.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_editor_initialization() {
        // Test: Text editor should initialize with empty content
        let replica_id = ReplicaId::default();
        let mut editor: Rga<char> = Rga::new(replica_id);
        assert_eq!(editor.len(), 0);
        assert!(editor.is_empty());
    }

    #[test]
    fn test_text_editor_single_character_insertion() {
        // Test: Inserting a single character should work correctly
        let replica_id = ReplicaId::default();
        let mut editor: Rga<char> = Rga::new(replica_id);
        let _pos = editor.insert_after('H', None).unwrap();
        
        assert_eq!(editor.len(), 1);
        let elements = editor.to_vec();
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0], 'H');
    }

    #[test]
    fn test_text_editor_multiple_character_insertion() {
        // Test: Inserting multiple characters should maintain order
        let replica_id = ReplicaId::default();
        let mut editor: Rga<char> = Rga::new(replica_id);
        let pos1 = editor.insert_after('H', None).unwrap();
        let pos2 = editor.insert_after('e', Some(pos1)).unwrap();
        let pos3 = editor.insert_after('l', Some(pos2)).unwrap();
        let pos4 = editor.insert_after('l', Some(pos3)).unwrap();
        let _pos5 = editor.insert_after('o', Some(pos4)).unwrap();
        
        assert_eq!(editor.len(), 5);
        
        // Verify the text content
        let content: String = editor.to_vec().into_iter().collect();
        assert_eq!(content, "Hello");
    }

    #[test]
    fn test_text_editor_character_deletion() {
        // Test: Deleting characters should work correctly
        let replica_id = ReplicaId::default();
        let mut editor: Rga<char> = Rga::new(replica_id);
        let pos1 = editor.insert_after('H', None).unwrap();
        let pos2 = editor.insert_after('e', Some(pos1)).unwrap();
        let pos3 = editor.insert_after('l', Some(pos2)).unwrap();
        let pos4 = editor.insert_after('l', Some(pos3)).unwrap();
        let _pos5 = editor.insert_after('o', Some(pos4.clone())).unwrap();
        
        // Delete the second 'l'
        editor.delete(&pos4).unwrap();
        
        assert_eq!(editor.len(), 5); // Elements still exist but one is invisible
        let content: String = editor.to_vec().into_iter().collect();
        assert_eq!(content, "Helo");
    }

    #[test]
    fn test_text_editor_collaborative_editing() {
        // Test: Two users editing simultaneously should merge correctly
        let replica_id1 = ReplicaId::default();
        let replica_id2 = ReplicaId::default();
        let mut editor1: Rga<char> = Rga::new(replica_id1);
        let mut editor2: Rga<char> = Rga::new(replica_id2);
        
        // User 1 types "Hello"
        let pos1 = editor1.insert_after('H', None).unwrap();
        let pos2 = editor1.insert_after('e', Some(pos1)).unwrap();
        let pos3 = editor1.insert_after('l', Some(pos2)).unwrap();
        let pos4 = editor1.insert_after('l', Some(pos3)).unwrap();
        let _pos5 = editor1.insert_after('o', Some(pos4)).unwrap();
        
        // User 2 types "World" (starting from empty)
        let pos6 = editor2.insert_after('W', None).unwrap();
        let pos7 = editor2.insert_after('o', Some(pos6)).unwrap();
        let pos8 = editor2.insert_after('r', Some(pos7)).unwrap();
        let pos9 = editor2.insert_after('l', Some(pos8)).unwrap();
        let _pos10 = editor2.insert_after('d', Some(pos9)).unwrap();
        
        // Merge editor2 into editor1
        editor1.merge(&editor2).unwrap();
        
        // The result should contain both "Hello" and "World"
        assert_eq!(editor1.len(), 10);
        let content: String = editor1.to_vec().into_iter().collect();
        assert!(content.contains("Hello"));
        assert!(content.contains("World"));
    }

    #[test]
    fn test_text_editor_concurrent_insertions() {
        // Test: Concurrent insertions at the same position should be handled correctly
        let replica_id1 = ReplicaId::default();
        let replica_id2 = ReplicaId::default();
        let mut editor1: Rga<char> = Rga::new(replica_id1);
        let mut editor2: Rga<char> = Rga::new(replica_id2);
        
        // Both users start with "Hi"
        let pos1 = editor1.insert_after('H', None).unwrap();
        let pos2 = editor1.insert_after('i', Some(pos1)).unwrap();
        
        let pos3 = editor2.insert_after('H', None).unwrap();
        let pos4 = editor2.insert_after('i', Some(pos3)).unwrap();
        
        // User 1 inserts '!' after 'i'
        let _pos5 = editor1.insert_after('!', Some(pos2)).unwrap();
        
        // User 2 inserts '?' after 'i' (concurrent)
        let _pos6 = editor2.insert_after('?', Some(pos4)).unwrap();
        
        // Merge both changes
        editor1.merge(&editor2).unwrap();
        
        // Both characters should be present
        assert_eq!(editor1.len(), 6);
        let content: String = editor1.to_vec().into_iter().collect();
        assert!(content.contains("!"));
        assert!(content.contains("?"));
    }

    #[test]
    fn test_text_editor_large_document() {
        // Test: Performance with large documents
        let replica_id = ReplicaId::default();
        let mut editor: Rga<char> = Rga::new(replica_id);
        
        // Insert a large amount of text
        let text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(10);
        let mut last_pos = None;
        
        for ch in text.chars() {
            last_pos = Some(editor.insert_after(ch, last_pos).unwrap());
        }
        
        assert_eq!(editor.len(), text.len());
        
        // Verify content integrity
        let content: String = editor.to_vec().into_iter().collect();
        assert_eq!(content, text);
    }

    #[test]
    fn test_text_editor_unicode_support() {
        // Test: Unicode character support
        let replica_id = ReplicaId::default();
        let mut editor: Rga<char> = Rga::new(replica_id);
        
        // Insert various Unicode characters
        let unicode_chars = ['🚀', '🌟', '💻', '🎉', '🔥'];
        let mut last_pos = None;
        
        for &ch in &unicode_chars {
            last_pos = Some(editor.insert_after(ch, last_pos).unwrap());
        }
        
        assert_eq!(editor.len(), unicode_chars.len());
        
        // Verify Unicode content
        let content: Vec<char> = editor.to_vec();
        assert_eq!(content, unicode_chars);
    }

    #[test]
    fn test_text_editor_serialization() {
        // Test: Serialization and deserialization
        let replica_id = ReplicaId::default();
        let mut editor: Rga<char> = Rga::new(replica_id);
        
        // Insert some text
        let pos1 = editor.insert_after('H', None).unwrap();
        let _pos2 = editor.insert_after('i', Some(pos1)).unwrap();
        
        // For now, just test that we can get the content
        // Serialization will be tested separately when RGA implements proper serialization
        let content: String = editor.to_vec().into_iter().collect();
        assert_eq!(content, "Hi");
        assert_eq!(editor.len(), 2);
    }
}