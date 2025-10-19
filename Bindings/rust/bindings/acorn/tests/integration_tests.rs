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
        // Use memory storage for testing to avoid file deserialization issues
        AcornTree::open("memory://").expect("Failed to open test tree")
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
        let result: Result<TestData, Error> = tree.crack("nonexistent");
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

    #[test]
    fn test_iterator_basic() {
        let mut tree = get_test_tree();

        // Store some test data
        for i in 0..5 {
            let data = TestData {
                id: format!("item-{}", i),
                value: i * 10,
                name: format!("Item {}", i),
            };
            tree.stash(&format!("item-{}", i), &data).unwrap();
        }

        // Iterate over all items
        let mut iter = tree.iter("").unwrap();
        let items: Vec<(String, TestData)> = iter.collect().unwrap();

        assert_eq!(items.len(), 5);

        // Verify items are sorted by key
        for (i, (key, data)) in items.iter().enumerate() {
            assert_eq!(key, &format!("item-{}", i));
            assert_eq!(data.value, (i as i32) * 10);
        }
    }

    #[test]
    fn test_iterator_with_prefix() {
        let mut tree = get_test_tree();

        // Store items with different prefixes
        tree.stash("user:alice", &TestData { id: "alice".to_string(), value: 1, name: "Alice".to_string() }).unwrap();
        tree.stash("user:bob", &TestData { id: "bob".to_string(), value: 2, name: "Bob".to_string() }).unwrap();
        tree.stash("product:laptop", &TestData { id: "laptop".to_string(), value: 1000, name: "Laptop".to_string() }).unwrap();
        tree.stash("product:phone", &TestData { id: "phone".to_string(), value: 500, name: "Phone".to_string() }).unwrap();

        // Iterate over users only
        let mut user_iter = tree.iter("user:").unwrap();
        let users: Vec<(String, TestData)> = user_iter.collect().unwrap();

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].0, "user:alice");
        assert_eq!(users[1].0, "user:bob");

        // Iterate over products only
        let mut product_iter = tree.iter("product:").unwrap();
        let products: Vec<(String, TestData)> = product_iter.collect().unwrap();

        assert_eq!(products.len(), 2);
        assert_eq!(products[0].0, "product:laptop");
        assert_eq!(products[1].0, "product:phone");
    }

    #[test]
    fn test_iterator_manual_next() {
        let mut tree = get_test_tree();

        // Store a few items
        for i in 0..3 {
            let data = TestData {
                id: format!("test-{}", i),
                value: i,
                name: format!("Test {}", i),
            };
            tree.stash(&format!("test-{}", i), &data).unwrap();
        }

        // Manually iterate
        let mut iter = tree.iter("").unwrap();

        let first: Option<(String, TestData)> = iter.next().unwrap();
        assert!(first.is_some());
        assert_eq!(first.unwrap().0, "test-0");

        let second: Option<(String, TestData)> = iter.next().unwrap();
        assert!(second.is_some());
        assert_eq!(second.unwrap().0, "test-1");

        let third: Option<(String, TestData)> = iter.next().unwrap();
        assert!(third.is_some());
        assert_eq!(third.unwrap().0, "test-2");

        let done: Option<(String, TestData)> = iter.next().unwrap();
        assert!(done.is_none());
    }

    #[test]
    fn test_iterator_empty() {
        let tree = get_test_tree();

        // Iterate over empty tree
        let mut iter = tree.iter("").unwrap();
        let items: Vec<(String, TestData)> = iter.collect().unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_iterator_no_match() {
        let mut tree = get_test_tree();

        // Store some items
        tree.stash("foo", &TestData { id: "1".to_string(), value: 1, name: "One".to_string() }).unwrap();
        tree.stash("bar", &TestData { id: "2".to_string(), value: 2, name: "Two".to_string() }).unwrap();

        // Iterate with prefix that matches nothing
        let mut iter = tree.iter("baz:").unwrap();
        let items: Vec<(String, TestData)> = iter.collect().unwrap();

        assert_eq!(items.len(), 0);
    }

    #[test]
    fn test_subscription_basic() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        let mut tree = get_test_tree();

        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        // Subscribe to changes
        let _sub = tree.subscribe(move |key: &str, _value: &serde_json::Value| {
            let mut n = notifications_clone.lock().unwrap();
            n.push(key.to_string());
        }).unwrap();

        // Give subscription time to initialize
        thread::sleep(Duration::from_millis(100));

        // Store some data
        tree.stash("test-1", &TestData { id: "1".to_string(), value: 42, name: "First".to_string() }).unwrap();
        tree.stash("test-2", &TestData { id: "2".to_string(), value: 43, name: "Second".to_string() }).unwrap();

        // Wait for notifications
        thread::sleep(Duration::from_millis(300));

        // Check we received notifications (order may vary due to async nature)
        let n = notifications.lock().unwrap();
        assert_eq!(n.len(), 2);
        assert!(n.contains(&"test-1".to_string()));
        assert!(n.contains(&"test-2".to_string()));
    }

    #[test]
    fn test_subscription_update() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        let mut tree = get_test_tree();

        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        let _sub = tree.subscribe(move |key: &str, _value: &serde_json::Value| {
            let mut n = notifications_clone.lock().unwrap();
            n.push(key.to_string());
        }).unwrap();

        thread::sleep(Duration::from_millis(100));

        // Store initial value
        tree.stash("item", &TestData { id: "1".to_string(), value: 1, name: "One".to_string() }).unwrap();
        thread::sleep(Duration::from_millis(200));

        // Update the value
        tree.stash("item", &TestData { id: "1".to_string(), value: 2, name: "Two".to_string() }).unwrap();
        thread::sleep(Duration::from_millis(200));

        // Should have received 2 notifications for the same key
        let n = notifications.lock().unwrap();
        assert_eq!(n.len(), 2);
        assert_eq!(n[0], "item");
        assert_eq!(n[1], "item");
    }

    #[test]
    fn test_subscription_drop() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        let mut tree = get_test_tree();

        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();

        {
            let _sub = tree.subscribe(move |key: &str, _value: &serde_json::Value| {
                let mut n = notifications_clone.lock().unwrap();
                n.push(key.to_string());
            }).unwrap();

            thread::sleep(Duration::from_millis(100));

            // This should trigger a notification
            tree.stash("before-drop", &TestData { id: "1".to_string(), value: 1, name: "One".to_string() }).unwrap();
            thread::sleep(Duration::from_millis(200));

            // Subscription is dropped here
        }

        // After subscription is dropped, this should NOT trigger a notification
        tree.stash("after-drop", &TestData { id: "2".to_string(), value: 2, name: "Two".to_string() }).unwrap();
        thread::sleep(Duration::from_millis(200));

        // Should only have received 1 notification
        let n = notifications.lock().unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0], "before-drop");
    }

    #[test]
    fn test_multiple_subscriptions() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        use std::time::Duration;

        let mut tree = get_test_tree();

        let notifications1 = Arc::new(Mutex::new(Vec::new()));
        let notifications2 = Arc::new(Mutex::new(Vec::new()));
        let n1_clone = notifications1.clone();
        let n2_clone = notifications2.clone();

        // Create two subscriptions
        let _sub1 = tree.subscribe(move |key: &str, _value: &serde_json::Value| {
            let mut n = n1_clone.lock().unwrap();
            n.push(key.to_string());
        }).unwrap();

        let _sub2 = tree.subscribe(move |key: &str, _value: &serde_json::Value| {
            let mut n = n2_clone.lock().unwrap();
            n.push(key.to_string());
        }).unwrap();

        thread::sleep(Duration::from_millis(100));

        // Store data
        tree.stash("shared", &TestData { id: "1".to_string(), value: 1, name: "One".to_string() }).unwrap();
        thread::sleep(Duration::from_millis(300));

        // Both subscriptions should have received the notification
        let n1 = notifications1.lock().unwrap();
        let n2 = notifications2.lock().unwrap();
        assert_eq!(n1.len(), 1);
        assert_eq!(n2.len(), 1);
        assert_eq!(n1[0], "shared");
        assert_eq!(n2[0], "shared");
    }

    #[test]
    fn test_sync_http_unreachable() {
        let tree = get_test_tree();

        // Test that sync with unreachable URL doesn't panic
        // Note: Branch.ShakeAsync is fault-tolerant and logs errors but doesn't fail
        let result = tree.sync_http("http://nonexistent.invalid:9999/acorn");

        // The current implementation logs errors but returns Ok
        // This is by design - sync is best-effort
        assert!(result.is_ok(), "sync_http should be fault-tolerant");
    }

    #[test]
    fn test_sync_http_invalid_url() {
        let tree = get_test_tree();

        // Test with malformed URL
        let result = tree.sync_http("not a url at all");

        // Should complete without panicking
        // Branch will log an error but won't throw
        assert!(result.is_ok(), "sync_http should handle invalid URLs gracefully");
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

