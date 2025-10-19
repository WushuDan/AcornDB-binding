use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct TestData {
    id: String,
    value: i32,
    name: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ComplexData {
    numbers: Vec<i32>,
    text: String,
    nested: Option<Box<ComplexData>>,
}

// These tests require the shim to be built and ACORN_SHIM_DIR to be set
// Run with: cargo test --features integration-tests

#[cfg(feature = "integration-tests")]
mod integration_tests {
    use super::*;

    fn get_test_tree() -> AcornTree {
        // Use a temporary directory for testing
        let temp_dir = std::env::temp_dir().join("acorn_test");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let uri = format!("file://{}", temp_dir.to_string_lossy());
        AcornTree::open(&uri).expect("Failed to open test tree")
    }

    #[test]
    fn test_basic_crud_operations() {
        let mut tree = get_test_tree();
        
        let test_data = TestData {
            id: "test-1".to_string(),
            value: 42,
            name: "Test Item".to_string(),
        };

        // Test stash
        tree.stash("test-1", &test_data).expect("Failed to stash data");

        // Test crack
        let retrieved: TestData = tree.crack("test-1").expect("Failed to crack data");
        assert_eq!(test_data, retrieved);

        // Test not found
        let result: Result<TestData> = tree.crack("nonexistent");
        assert!(matches!(result, Err(Error::NotFound)));
    }

    #[test]
    fn test_complex_data_structures() {
        let mut tree = get_test_tree();
        
        let complex_data = ComplexData {
            numbers: vec![1, 2, 3, 4, 5],
            text: "Hello, World!".to_string(),
            nested: Some(Box::new(ComplexData {
                numbers: vec![10, 20],
                text: "Nested".to_string(),
                nested: None,
            })),
        };

        tree.stash("complex", &complex_data).expect("Failed to stash complex data");
        let retrieved: ComplexData = tree.crack("complex").expect("Failed to crack complex data");
        assert_eq!(complex_data, retrieved);
    }

    #[test]
    fn test_error_handling() {
        let mut tree = get_test_tree();
        
        // Test invalid ID (contains null byte)
        let invalid_data = TestData {
            id: "test\0invalid".to_string(),
            value: 1,
            name: "Invalid".to_string(),
        };

        let result = tree.stash("test\0invalid", &invalid_data);
        assert!(result.is_err());
        
        // Test serialization error (circular reference would cause this)
        // This is harder to test without creating actual circular references
    }

    #[test]
    fn test_multiple_operations() {
        let mut tree = get_test_tree();
        
        // Store multiple items
        for i in 0..10 {
            let data = TestData {
                id: format!("item-{}", i),
                value: i * 10,
                name: format!("Item {}", i),
            };
            tree.stash(&format!("item-{}", i), &data).expect("Failed to stash");
        }

        // Retrieve and verify
        for i in 0..10 {
            let retrieved: TestData = tree.crack(&format!("item-{}", i)).expect("Failed to crack");
            assert_eq!(retrieved.value, i * 10);
            assert_eq!(retrieved.name, format!("Item {}", i));
        }
    }

    #[test]
    fn test_tree_drop_behavior() {
        // Test that the tree properly closes when dropped
        let tree = get_test_tree();
        // Tree should be automatically closed when dropped
        drop(tree);
        // If we get here without panicking, the drop implementation works
    }
}

// Unit tests that don't require the shim
#[cfg(test)]
mod unit_tests {
    use super::*;

    #[test]
    fn test_data_serialization() {
        let data = TestData {
            id: "test".to_string(),
            value: 42,
            name: "Test".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TestData = serde_json::from_str(&json).unwrap();
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_error_display() {
        let error = Error::Acorn("test error".to_string());
        assert!(error.to_string().contains("test error"));
        
        let not_found = Error::NotFound;
        assert!(not_found.to_string().contains("Not found"));
    }
}

