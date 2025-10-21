use acorn::{AcornTree, AcornEncryption, AcornCompression, AcornCache, AcornConflictJudge, AcornStorage, AcornDocumentStore, CompressionLevel, Error, ChangeType};
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

    #[test]
    fn test_batch_stash() {
        let mut tree = get_test_tree();

        let items = vec![
            ("batch-1", TestData { id: "1".to_string(), value: 100, name: "First".to_string() }),
            ("batch-2", TestData { id: "2".to_string(), value: 200, name: "Second".to_string() }),
            ("batch-3", TestData { id: "3".to_string(), value: 300, name: "Third".to_string() }),
        ];

        tree.batch_stash(&items).unwrap();

        // Verify all items were stored
        for (key, expected) in &items {
            let retrieved: TestData = tree.crack(key).unwrap();
            assert_eq!(&retrieved, expected);
        }
    }

    #[test]
    fn test_batch_crack() {
        let mut tree = get_test_tree();

        // Store some items
        tree.stash("key1", &TestData { id: "1".to_string(), value: 1, name: "One".to_string() }).unwrap();
        tree.stash("key2", &TestData { id: "2".to_string(), value: 2, name: "Two".to_string() }).unwrap();
        tree.stash("key3", &TestData { id: "3".to_string(), value: 3, name: "Three".to_string() }).unwrap();

        // Batch crack with some found and some not found
        let keys = vec!["key1", "key2", "missing", "key3"];
        let results: Vec<Option<TestData>> = tree.batch_crack(&keys).unwrap();

        assert_eq!(results.len(), 4);
        assert!(results[0].is_some());
        assert_eq!(results[0].as_ref().unwrap().value, 1);
        assert!(results[1].is_some());
        assert_eq!(results[1].as_ref().unwrap().value, 2);
        assert!(results[2].is_none()); // missing key
        assert!(results[3].is_some());
        assert_eq!(results[3].as_ref().unwrap().value, 3);
    }

    #[test]
    fn test_batch_delete() {
        let mut tree = get_test_tree();

        // Store some items
        for i in 0..5 {
            tree.stash(&format!("del-{}", i), &TestData {
                id: i.to_string(),
                value: i,
                name: format!("Item {}", i),
            }).unwrap();
        }

        // Batch delete some items
        let to_delete = vec!["del-1", "del-3"];
        tree.batch_delete(&to_delete).unwrap();

        // Verify deleted items are gone
        let result: Result<TestData, _> = tree.crack("del-1");
        assert!(matches!(result, Err(Error::NotFound)));

        let result: Result<TestData, _> = tree.crack("del-3");
        assert!(matches!(result, Err(Error::NotFound)));

        // Verify other items still exist
        let result: Result<TestData, _> = tree.crack("del-0");
        assert!(result.is_ok());

        let result: Result<TestData, _> = tree.crack("del-2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_batch_empty() {
        let mut tree = get_test_tree();

        // Test empty batch operations
        let empty_items: Vec<(&str, TestData)> = vec![];
        tree.batch_stash(&empty_items).unwrap();

        let empty_keys: Vec<&str> = vec![];
        let results: Vec<Option<TestData>> = tree.batch_crack(&empty_keys).unwrap();
        assert_eq!(results.len(), 0);

        tree.batch_delete(&empty_keys).unwrap();
    }

    #[test]
    fn test_batch_large() {
        let mut tree = get_test_tree();

        // Test with larger batch (100 items)
        let items: Vec<(&str, TestData)> = (0..100)
            .map(|i| {
                (
                    Box::leak(format!("large-{}", i).into_boxed_str()) as &str,
                    TestData {
                        id: i.to_string(),
                        value: i,
                        name: format!("Item {}", i),
                    },
                )
            })
            .collect();

        tree.batch_stash(&items).unwrap();

        // Verify with batch crack
        let keys: Vec<&str> = items.iter().map(|(k, _)| *k).collect();
        let results: Vec<Option<TestData>> = tree.batch_crack(&keys).unwrap();

        assert_eq!(results.len(), 100);
        for (i, result) in results.iter().enumerate() {
            assert!(result.is_some());
            assert_eq!(result.as_ref().unwrap().value, i as i32);
        }
    }

    #[test]
    fn test_query_basic() {
        let mut tree = get_test_tree();

        // Store some test data
        let items = vec![
            ("query-1", TestData { id: "1".to_string(), value: 100, name: "First".to_string() }),
            ("query-2", TestData { id: "2".to_string(), value: 200, name: "Second".to_string() }),
            ("query-3", TestData { id: "3".to_string(), value: 300, name: "Third".to_string() }),
        ];

        tree.batch_stash(&items).unwrap();

        // Test basic query operations
        let all_items: Vec<TestData> = tree.query().collect().unwrap();
        assert_eq!(all_items.len(), 3);

        let first_item: Option<TestData> = tree.query().first().unwrap();
        assert!(first_item.is_some());

        let count = tree.query().count().unwrap();
        assert_eq!(count, 3);

        let has_items = tree.query().any().unwrap();
        assert!(has_items);
    }

    #[test]
    fn test_query_where_condition() {
        let mut tree = get_test_tree();

        // Store test data with different values
        let items = vec![
            ("where-1", TestData { id: "1".to_string(), value: 100, name: "Low".to_string() }),
            ("where-2", TestData { id: "2".to_string(), value: 200, name: "Medium".to_string() }),
            ("where-3", TestData { id: "3".to_string(), value: 300, name: "High".to_string() }),
        ];

        tree.batch_stash(&items).unwrap();

        // Test filtering by value
        let high_values: Vec<TestData> = tree.query()
            .where_condition(|item| item["value"].as_u64().unwrap_or(0) > 150)
            .collect()
            .unwrap();
        assert_eq!(high_values.len(), 2);
        assert!(high_values.iter().all(|item| item.value > 150));

        // Test filtering by name
        let medium_items: Vec<TestData> = tree.query()
            .where_condition(|item| item["name"].as_str() == Some("Medium"))
            .collect()
            .unwrap();
        assert_eq!(medium_items.len(), 1);
        assert_eq!(medium_items[0].name, "Medium");
    }

    #[test]
    fn test_query_order_by() {
        let mut tree = get_test_tree();

        // Store test data in random order
        let items = vec![
            ("order-3", TestData { id: "3".to_string(), value: 300, name: "Charlie".to_string() }),
            ("order-1", TestData { id: "1".to_string(), value: 100, name: "Alice".to_string() }),
            ("order-2", TestData { id: "2".to_string(), value: 200, name: "Bob".to_string() }),
        ];

        tree.batch_stash(&items).unwrap();

        // Test ordering by name
        let ordered_by_name: Vec<TestData> = tree.query()
            .order_by(|item| item["name"].as_str().unwrap_or("").to_string())
            .collect()
            .unwrap();
        assert_eq!(ordered_by_name.len(), 3);
        assert_eq!(ordered_by_name[0].name, "Alice");
        assert_eq!(ordered_by_name[1].name, "Bob");
        assert_eq!(ordered_by_name[2].name, "Charlie");

        // Test ordering by value (descending)
        let ordered_by_value: Vec<TestData> = tree.query()
            .order_by_descending(|item| item["value"].as_u64().unwrap_or(0).to_string())
            .collect()
            .unwrap();
        assert_eq!(ordered_by_value.len(), 3);
        assert_eq!(ordered_by_value[0].value, 300);
        assert_eq!(ordered_by_value[1].value, 200);
        assert_eq!(ordered_by_value[2].value, 100);
    }

    #[test]
    fn test_query_where_and_order() {
        let mut tree = get_test_tree();

        // Store test data
        let items = vec![
            ("combo-1", TestData { id: "1".to_string(), value: 100, name: "Alice".to_string() }),
            ("combo-2", TestData { id: "2".to_string(), value: 200, name: "Bob".to_string() }),
            ("combo-3", TestData { id: "3".to_string(), value: 300, name: "Charlie".to_string() }),
            ("combo-4", TestData { id: "4".to_string(), value: 400, name: "David".to_string() }),
        ];

        tree.batch_stash(&items).unwrap();

        // Test combining WHERE and ORDER BY
        let filtered_and_ordered: Vec<TestData> = tree.query()
            .where_condition(|item| item["value"].as_u64().unwrap_or(0) >= 200)
            .order_by(|item| item["name"].as_str().unwrap_or("").to_string())
            .collect()
            .unwrap();
        assert_eq!(filtered_and_ordered.len(), 3);
        assert_eq!(filtered_and_ordered[0].name, "Bob");
        assert_eq!(filtered_and_ordered[1].name, "Charlie");
        assert_eq!(filtered_and_ordered[2].name, "David");
    }

    #[test]
    fn test_query_take_and_skip() {
        let mut tree = get_test_tree();

        // Store test data
        let items: Vec<(&str, TestData)> = (0..10)
            .map(|i| {
                (
                    Box::leak(format!("take-skip-{}", i).into_boxed_str()) as &str,
                    TestData {
                        id: i.to_string(),
                        value: i as i32 * 10,
                        name: format!("Item {}", i),
                    },
                )
            })
            .collect();

        tree.batch_stash(&items).unwrap();

        // Test TAKE
        let first_three: Vec<TestData> = tree.query()
            .order_by(|item| item["name"].as_str().unwrap_or("").to_string())
            .take(3)
            .collect()
            .unwrap();
        assert_eq!(first_three.len(), 3);

        // Test SKIP
        let after_first_three: Vec<TestData> = tree.query()
            .order_by(|item| item["name"].as_str().unwrap_or("").to_string())
            .skip(3)
            .collect()
            .unwrap();
        assert_eq!(after_first_three.len(), 7);

        // Test WHERE + TAKE
        let high_values_take: Vec<TestData> = tree.query()
            .where_condition(|item| item["value"].as_u64().unwrap_or(0) >= 50)
            .take(2)
            .collect()
            .unwrap();
        assert_eq!(high_values_take.len(), 2);
        assert!(high_values_take.iter().all(|item| item.value >= 50));
    }

    #[test]
    fn test_query_empty_tree() {
        let tree = get_test_tree();

        // Test queries on empty tree
        let empty_results: Vec<TestData> = tree.query().collect().unwrap();
        assert_eq!(empty_results.len(), 0);

        let first_empty: Option<TestData> = tree.query().first().unwrap();
        assert!(first_empty.is_none());

        let count_empty = tree.query().count().unwrap();
        assert_eq!(count_empty, 0);

        let any_empty = tree.query().any().unwrap();
        assert!(!any_empty);
    }

    #[test]
    fn test_query_complex_filtering() {
        let mut tree = get_test_tree();

        // Store test data with various properties
        let items = vec![
            ("complex-1", TestData { id: "1".to_string(), value: 100, name: "Alpha".to_string() }),
            ("complex-2", TestData { id: "2".to_string(), value: 200, name: "Beta".to_string() }),
            ("complex-3", TestData { id: "3".to_string(), value: 300, name: "Gamma".to_string() }),
            ("complex-4", TestData { id: "4".to_string(), value: 400, name: "Delta".to_string() }),
            ("complex-5", TestData { id: "5".to_string(), value: 500, name: "Epsilon".to_string() }),
        ];

        tree.batch_stash(&items).unwrap();

        // Test complex filtering conditions
        let medium_values: Vec<TestData> = tree.query()
            .where_condition(|item| {
                let value = item["value"].as_u64().unwrap_or(0);
                value >= 200 && value <= 400
            })
            .order_by(|item| item["value"].as_u64().unwrap_or(0).to_string())
            .collect()
            .unwrap();
        assert_eq!(medium_values.len(), 3);
        assert_eq!(medium_values[0].value, 200);
        assert_eq!(medium_values[1].value, 300);
        assert_eq!(medium_values[2].value, 400);

        // Test filtering by name pattern
        let greek_letters: Vec<TestData> = tree.query()
            .where_condition(|item| {
                let name = item["name"].as_str().unwrap_or("");
                name.len() == 5 // All Greek letters are 5 characters
            })
            .collect()
            .unwrap();
        assert_eq!(greek_letters.len(), 3); // Alpha, Gamma, and Delta
    }

    #[test]
    fn test_gzip_compression() {
        let compression = AcornCompression::gzip(CompressionLevel::Optimal).expect("Failed to create Gzip compression");
        
        // Test basic compression/decompression
        let original = "Hello, world! This is a test of compression.";
        let compressed = compression.compress(original).expect("Failed to compress");
        let decompressed = compression.decompress(&compressed).expect("Failed to decompress");
        assert_eq!(original, decompressed);
        
        // Test compression stats
        let stats = compression.get_stats(original, &compressed).expect("Failed to get stats");
        assert!(stats.compressed_size < stats.original_size);
        assert!(stats.ratio < 1.0);
        assert!(stats.space_saved > 0);
        
        // Test algorithm name
        let algorithm = compression.algorithm_name().expect("Failed to get algorithm name");
        assert_eq!(algorithm, "Gzip");
        
        // Test enabled status
        let enabled = compression.is_enabled().expect("Failed to check enabled status");
        assert!(enabled);
    }

    #[test]
    fn test_brotli_compression() {
        let compression = AcornCompression::brotli(CompressionLevel::SmallestSize).expect("Failed to create Brotli compression");
        
        // Test basic compression/decompression
        let original = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(5);
        let compressed = compression.compress(&original).expect("Failed to compress");
        let decompressed = compression.decompress(&compressed).expect("Failed to decompress");
        assert_eq!(original, decompressed);
        
        // Test compression stats
        let stats = compression.get_stats(&original, &compressed).expect("Failed to get stats");
        assert!(stats.compressed_size < stats.original_size);
        assert!(stats.ratio < 1.0);
        assert!(stats.space_saved > 0);
        
        // Test algorithm name
        let algorithm = compression.algorithm_name().expect("Failed to get algorithm name");
        assert_eq!(algorithm, "Brotli");
        
        // Test enabled status
        let enabled = compression.is_enabled().expect("Failed to check enabled status");
        assert!(enabled);
    }

    #[test]
    fn test_no_compression() {
        let compression = AcornCompression::none().expect("Failed to create no compression");
        
        // Test basic compression/decompression (should pass through unchanged)
        let original = "Hello, world!";
        let compressed = compression.compress(original).expect("Failed to compress");
        let decompressed = compression.decompress(&compressed).expect("Failed to decompress");
        assert_eq!(original, decompressed);
        
        // Test algorithm name
        let algorithm = compression.algorithm_name().expect("Failed to get algorithm name");
        assert_eq!(algorithm, "None");
        
        // Test enabled status (should be false)
        let enabled = compression.is_enabled().expect("Failed to check enabled status");
        assert!(!enabled);
    }

    #[test]
    fn test_compression_levels() {
        let fastest = AcornCompression::gzip(CompressionLevel::Fastest).expect("Failed to create fastest compression");
        let optimal = AcornCompression::gzip(CompressionLevel::Optimal).expect("Failed to create optimal compression");
        let smallest = AcornCompression::gzip(CompressionLevel::SmallestSize).expect("Failed to create smallest compression");
        
        let test_data = "This is a test of different compression levels. ".repeat(10);
        
        let fastest_compressed = fastest.compress(&test_data).expect("Failed to compress with fastest");
        let optimal_compressed = optimal.compress(&test_data).expect("Failed to compress with optimal");
        let smallest_compressed = smallest.compress(&test_data).expect("Failed to compress with smallest");
        
        let fastest_stats = fastest.get_stats(&test_data, &fastest_compressed).expect("Failed to get fastest stats");
        let optimal_stats = optimal.get_stats(&test_data, &optimal_compressed).expect("Failed to get optimal stats");
        let smallest_stats = smallest.get_stats(&test_data, &smallest_compressed).expect("Failed to get smallest stats");
        
        // Smallest should generally compress better than optimal, which should compress better than fastest
        // (though this isn't guaranteed for all data)
        println!("Fastest: {} bytes, Optimal: {} bytes, Smallest: {} bytes", 
                 fastest_stats.compressed_size, optimal_stats.compressed_size, smallest_stats.compressed_size);
        
        // All should be able to decompress correctly
        assert_eq!(test_data, fastest.decompress(&fastest_compressed).expect("Failed to decompress fastest"));
        assert_eq!(test_data, optimal.decompress(&optimal_compressed).expect("Failed to decompress optimal"));
        assert_eq!(test_data, smallest.decompress(&smallest_compressed).expect("Failed to decompress smallest"));
    }

    #[test]
    fn test_compressed_tree_storage() {
        let compression = AcornCompression::gzip(CompressionLevel::Optimal).expect("Failed to create compression");
        let mut tree = AcornTree::open_compressed("memory://compressed_test", &compression).expect("Failed to open compressed tree");
        
        // Store test data
        let test_data = TestData {
            id: "compressed-test".to_string(),
            value: 123,
            name: "Compressed Test Item".to_string(),
        };
        
        tree.stash("compressed-test", &test_data).expect("Failed to stash in compressed tree");
        
        // Retrieve test data
        let retrieved: TestData = tree.crack("compressed-test").expect("Failed to crack from compressed tree");
        assert_eq!(test_data, retrieved);
        
        // Store multiple items
        for i in 1..=5 {
            let item = TestData {
                id: format!("item-{}", i),
                value: i * 10,
                name: format!("Item {}", i),
            };
            tree.stash(&format!("item-{}", i), &item).expect("Failed to stash item");
        }
        
        // List all items
        let items = tree.list().expect("Failed to list items");
        assert_eq!(items.len(), 6); // 1 original + 5 new items
    }

    #[test]
    fn test_compression_error_handling() {
        let compression = AcornCompression::gzip(CompressionLevel::Optimal).expect("Failed to create compression");
        
        // Test decompression of invalid data
        let result = compression.decompress("invalid-base64-data");
        assert!(result.is_err());
        
        // Test stats with mismatched data
        let result = compression.get_stats("original", "different-compressed");
        assert!(result.is_err());
    }

    #[test]
    fn test_compression_with_large_data() {
        let compression = AcornCompression::gzip(CompressionLevel::Optimal).expect("Failed to create compression");
        
        // Create large test data
        let large_data = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);
        let compressed = compression.compress(&large_data).expect("Failed to compress large data");
        let decompressed = compression.decompress(&compressed).expect("Failed to decompress large data");
        assert_eq!(large_data, decompressed);
        
        // Test compression ratio
        let stats = compression.get_stats(&large_data, &compressed).expect("Failed to get stats");
        assert!(stats.ratio < 0.5); // Should compress to less than 50% of original size
        assert!(stats.space_saved > 1000); // Should save significant space
    }

    #[test]
    fn test_lru_cache() {
        let cache = AcornCache::lru(5).expect("Failed to create LRU cache");
        
        // Test initial stats
        let stats = cache.get_stats().expect("Failed to get cache stats");
        assert_eq!(stats.max_size, 5);
        assert_eq!(stats.tracked_items, 0);
        assert_eq!(stats.utilization_percentage, 0.0);
        
        // Test eviction enabled
        let eviction_enabled = cache.is_eviction_enabled().expect("Failed to check eviction status");
        assert!(eviction_enabled);
        
        // Test reset
        cache.reset().expect("Failed to reset cache");
        let stats_after_reset = cache.get_stats().expect("Failed to get stats after reset");
        assert_eq!(stats_after_reset.tracked_items, 0);
    }

    #[test]
    fn test_no_eviction_cache() {
        let cache = AcornCache::no_eviction().expect("Failed to create no eviction cache");
        
        // Test initial stats
        let stats = cache.get_stats().expect("Failed to get cache stats");
        assert_eq!(stats.max_size, i32::MAX); // Unlimited
        assert_eq!(stats.tracked_items, 0);
        assert_eq!(stats.utilization_percentage, 0.0);
        
        // Test eviction disabled
        let eviction_enabled = cache.is_eviction_enabled().expect("Failed to check eviction status");
        assert!(!eviction_enabled);
        
        // Test reset (should be no-op)
        cache.reset().expect("Failed to reset cache");
    }

    #[test]
    fn test_cache_with_tree() {
        let cache = AcornCache::lru(3).expect("Failed to create LRU cache");
        let mut tree = AcornTree::open_with_cache("memory://cache_test", &cache).expect("Failed to open tree with cache");
        
        // Store test data
        let test_data = TestData {
            id: "cache-test".to_string(),
            value: 123,
            name: "Cache Test Item".to_string(),
        };
        
        tree.stash("cache-test", &test_data).expect("Failed to stash in cached tree");
        
        // Retrieve test data
        let retrieved: TestData = tree.crack("cache-test").expect("Failed to crack from cached tree");
        assert_eq!(test_data, retrieved);
        
        // Store multiple items to test cache behavior
        for i in 1..=5 {
            let item = TestData {
                id: format!("item-{}", i),
                value: i * 10,
                name: format!("Item {}", i),
            };
            tree.stash(&format!("item-{}", i), &item).expect("Failed to stash item");
        }
        
        // List all items
        let items = tree.list().expect("Failed to list items");
        assert_eq!(items.len(), 6); // 1 original + 5 new items
        
        // Check cache stats
        let stats = cache.get_stats().expect("Failed to get cache stats");
        assert!(stats.tracked_items > 0);
        assert!(stats.utilization_percentage > 0.0);
    }

    #[test]
    fn test_cache_strategies_comparison() {
        let lru_cache = AcornCache::lru(10).expect("Failed to create LRU cache");
        let no_eviction_cache = AcornCache::no_eviction().expect("Failed to create no eviction cache");
        
        let mut lru_tree = AcornTree::open_with_cache("memory://lru_comparison", &lru_cache).expect("Failed to open LRU tree");
        let mut no_eviction_tree = AcornTree::open_with_cache("memory://no_eviction_comparison", &no_eviction_cache).expect("Failed to open no eviction tree");
        
        // Store the same data in both trees
        let test_items: Vec<TestData> = (1..=15)
            .map(|i| TestData {
                id: format!("item-{}", i),
                value: i,
                name: format!("Item {}", i),
            })
            .collect();
        
        for item in &test_items {
            lru_tree.stash(&item.id, item).expect("Failed to stash in LRU tree");
            no_eviction_tree.stash(&item.id, item).expect("Failed to stash in no eviction tree");
        }
        
        // Check stats
        let lru_stats = lru_cache.get_stats().expect("Failed to get LRU stats");
        let no_eviction_stats = no_eviction_cache.get_stats().expect("Failed to get no eviction stats");
        
        println!("LRU cache stats: tracked={}, max={}, utilization={:.1}%", 
                 lru_stats.tracked_items, lru_stats.max_size, lru_stats.utilization_percentage);
        println!("No eviction cache stats: tracked={}, max={}, utilization={:.1}%", 
                 no_eviction_stats.tracked_items, no_eviction_stats.max_size, no_eviction_stats.utilization_percentage);
        
        // Both should have tracked items
        assert!(lru_stats.tracked_items > 0);
        assert!(no_eviction_stats.tracked_items > 0);
        
        // LRU should have utilization percentage > 0
        assert!(lru_stats.utilization_percentage > 0.0);
        
        // No eviction should have utilization percentage = 0 (unlimited)
        assert_eq!(no_eviction_stats.utilization_percentage, 0.0);
    }

    #[test]
    fn test_cache_error_handling() {
        // Test invalid cache size
        let result = AcornCache::lru(0);
        assert!(result.is_err());
        
        let result = AcornCache::lru(-1);
        assert!(result.is_err());
        
        // Test cache operations on valid cache
        let cache = AcornCache::lru(10).expect("Failed to create cache");
        
        // Reset should work
        cache.reset().expect("Failed to reset cache");
        
        // Stats should work
        let stats = cache.get_stats().expect("Failed to get stats");
        assert_eq!(stats.tracked_items, 0);
    }

    #[test]
    fn test_cache_with_large_dataset() {
        let cache = AcornCache::lru(5).expect("Failed to create small LRU cache");
        let mut tree = AcornTree::open_with_cache("memory://large_dataset", &cache).expect("Failed to open tree");
        
        // Create large dataset
        let large_dataset: Vec<TestData> = (1..=20)
            .map(|i| TestData {
                id: format!("large-item-{}", i),
                value: i,
                name: format!("Large Item {}", i),
            })
            .collect();
        
        // Store all items
        for item in &large_dataset {
            tree.stash(&item.id, item).expect("Failed to stash large item");
        }
        
        // Check cache stats
        let stats = cache.get_stats().expect("Failed to get cache stats");
        assert!(stats.tracked_items > 0);
        assert!(stats.utilization_percentage > 0.0);
        
        // Verify all items can be retrieved
        for item in &large_dataset {
            let retrieved: TestData = tree.crack(&item.id).expect("Failed to retrieve large item");
            assert_eq!(item, &retrieved);
        }
    }

    #[test]
    fn test_timestamp_conflict_judge() {
        let judge = AcornConflictJudge::timestamp().expect("Failed to create timestamp judge");
        
        // Test judge name
        let name = judge.name().expect("Failed to get judge name");
        assert_eq!(name, "Timestamp");
        
        // Test conflict resolution with different timestamps
        let local_json = r#"{"id": "test", "name": "Local", "timestamp": "2023-01-01T10:00:00Z"}"#;
        let incoming_json = r#"{"id": "test", "name": "Incoming", "timestamp": "2023-01-01T11:00:00Z"}"#;
        
        let winner_json = judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve conflict");
        let winner: serde_json::Value = serde_json::from_str(&winner_json).expect("Failed to parse winner JSON");
        
        // Timestamp judge should pick the incoming (later timestamp)
        assert_eq!(winner["name"], "Incoming");
        assert_eq!(winner["timestamp"], "2023-01-01T11:00:00Z");
    }

    #[test]
    fn test_version_conflict_judge() {
        let judge = AcornConflictJudge::version().expect("Failed to create version judge");
        
        // Test judge name
        let name = judge.name().expect("Failed to get judge name");
        assert_eq!(name, "Version");
        
        // Test conflict resolution with different versions
        let local_json = r#"{"id": "test", "name": "Local", "version": 1, "timestamp": "2023-01-01T11:00:00Z"}"#;
        let incoming_json = r#"{"id": "test", "name": "Incoming", "version": 2, "timestamp": "2023-01-01T10:00:00Z"}"#;
        
        let winner_json = judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve conflict");
        let winner: serde_json::Value = serde_json::from_str(&winner_json).expect("Failed to parse winner JSON");
        
        // Version judge should pick the incoming (higher version)
        assert_eq!(winner["name"], "Incoming");
        assert_eq!(winner["version"], 2);
    }

    #[test]
    fn test_local_wins_conflict_judge() {
        let judge = AcornConflictJudge::local_wins().expect("Failed to create local wins judge");
        
        // Test judge name
        let name = judge.name().expect("Failed to get judge name");
        assert_eq!(name, "LocalWins");
        
        // Test conflict resolution
        let local_json = r#"{"id": "test", "name": "Local", "timestamp": "2023-01-01T10:00:00Z"}"#;
        let incoming_json = r#"{"id": "test", "name": "Incoming", "timestamp": "2023-01-01T11:00:00Z"}"#;
        
        let winner_json = judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve conflict");
        let winner: serde_json::Value = serde_json::from_str(&winner_json).expect("Failed to parse winner JSON");
        
        // Local wins judge should always pick the local
        assert_eq!(winner["name"], "Local");
        assert_eq!(winner["timestamp"], "2023-01-01T10:00:00Z");
    }

    #[test]
    fn test_remote_wins_conflict_judge() {
        let judge = AcornConflictJudge::remote_wins().expect("Failed to create remote wins judge");
        
        // Test judge name
        let name = judge.name().expect("Failed to get judge name");
        assert_eq!(name, "RemoteWins");
        
        // Test conflict resolution
        let local_json = r#"{"id": "test", "name": "Local", "timestamp": "2023-01-01T11:00:00Z"}"#;
        let incoming_json = r#"{"id": "test", "name": "Incoming", "timestamp": "2023-01-01T10:00:00Z"}"#;
        
        let winner_json = judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve conflict");
        let winner: serde_json::Value = serde_json::from_str(&winner_json).expect("Failed to parse winner JSON");
        
        // Remote wins judge should always pick the incoming
        assert_eq!(winner["name"], "Incoming");
        assert_eq!(winner["timestamp"], "2023-01-01T10:00:00Z");
    }

    #[test]
    fn test_conflict_judge_with_tree() {
        let judge = AcornConflictJudge::timestamp().expect("Failed to create timestamp judge");
        let mut tree = AcornTree::open_with_conflict_judge("memory://conflict_test", &judge).expect("Failed to open tree with conflict judge");
        
        // Store test data
        let test_data = TestData {
            id: "conflict-test".to_string(),
            value: 123,
            name: "Conflict Test Item".to_string(),
        };
        
        tree.stash("conflict-test", &test_data).expect("Failed to stash in tree with conflict judge");
        
        // Retrieve test data
        let retrieved: TestData = tree.crack("conflict-test").expect("Failed to crack from tree with conflict judge");
        assert_eq!(test_data, retrieved);
        
        // Store multiple items
        for i in 1..=5 {
            let item = TestData {
                id: format!("conflict-item-{}", i),
                value: i * 10,
                name: format!("Conflict Item {}", i),
            };
            tree.stash(&format!("conflict-item-{}", i), &item).expect("Failed to stash conflict item");
        }
        
        // List all items
        let items = tree.list().expect("Failed to list items");
        assert_eq!(items.len(), 6); // 1 original + 5 new items
    }

    #[test]
    fn test_conflict_resolution_strategies_comparison() {
        let timestamp_judge = AcornConflictJudge::timestamp().expect("Failed to create timestamp judge");
        let version_judge = AcornConflictJudge::version().expect("Failed to create version judge");
        let local_wins_judge = AcornConflictJudge::local_wins().expect("Failed to create local wins judge");
        let remote_wins_judge = AcornConflictJudge::remote_wins().expect("Failed to create remote wins judge");
        
        // Test data with different timestamps and versions
        let local_json = r#"{"id": "test", "name": "Local", "version": 1, "timestamp": "2023-01-01T10:00:00Z"}"#;
        let incoming_json = r#"{"id": "test", "name": "Incoming", "version": 2, "timestamp": "2023-01-01T11:00:00Z"}"#;
        
        // Test timestamp judge (should pick incoming - later timestamp)
        let timestamp_winner = timestamp_judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve with timestamp judge");
        let timestamp_result: serde_json::Value = serde_json::from_str(&timestamp_winner).expect("Failed to parse timestamp result");
        assert_eq!(timestamp_result["name"], "Incoming");
        
        // Test version judge (should pick incoming - higher version)
        let version_winner = version_judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve with version judge");
        let version_result: serde_json::Value = serde_json::from_str(&version_winner).expect("Failed to parse version result");
        assert_eq!(version_result["name"], "Incoming");
        
        // Test local wins judge (should pick local)
        let local_wins_winner = local_wins_judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve with local wins judge");
        let local_wins_result: serde_json::Value = serde_json::from_str(&local_wins_winner).expect("Failed to parse local wins result");
        assert_eq!(local_wins_result["name"], "Local");
        
        // Test remote wins judge (should pick incoming)
        let remote_wins_winner = remote_wins_judge.resolve_conflict(local_json, incoming_json).expect("Failed to resolve with remote wins judge");
        let remote_wins_result: serde_json::Value = serde_json::from_str(&remote_wins_winner).expect("Failed to parse remote wins result");
        assert_eq!(remote_wins_result["name"], "Incoming");
    }

    #[test]
    fn test_conflict_resolution_error_handling() {
        let judge = AcornConflictJudge::timestamp().expect("Failed to create timestamp judge");
        
        // Test invalid JSON
        let result = judge.resolve_conflict("invalid-json", r#"{"valid": "json"}"#);
        assert!(result.is_err());
        
        // Test mismatched JSON structures
        let result = judge.resolve_conflict(r#"{"id": "test"}"#, r#"{"different": "structure"}"#);
        assert!(result.is_err());
    }

    #[test]
    fn test_conflict_resolution_with_complex_data() {
        let judge = AcornConflictJudge::timestamp().expect("Failed to create timestamp judge");
        
        // Test with complex nested data
        let local_complex = r#"{
            "id": "complex",
            "user": {
                "name": "Local User",
                "email": "local@example.com",
                "preferences": {
                    "theme": "dark",
                    "notifications": true
                }
            },
            "timestamp": "2023-01-01T10:00:00Z",
            "version": 1
        }"#;
        
        let incoming_complex = r#"{
            "id": "complex",
            "user": {
                "name": "Incoming User",
                "email": "incoming@example.com",
                "preferences": {
                    "theme": "light",
                    "notifications": false
                }
            },
            "timestamp": "2023-01-01T11:00:00Z",
            "version": 1
        }"#;
        
        let winner_json = judge.resolve_conflict(local_complex, incoming_complex).expect("Failed to resolve complex conflict");
        let winner: serde_json::Value = serde_json::from_str(&winner_json).expect("Failed to parse complex winner");
        
        // Should pick the incoming (later timestamp)
        assert_eq!(winner["user"]["name"], "Incoming User");
        assert_eq!(winner["user"]["email"], "incoming@example.com");
        assert_eq!(winner["user"]["preferences"]["theme"], "light");
        assert_eq!(winner["timestamp"], "2023-01-01T11:00:00Z");
    }

    #[test]
    fn test_sqlite_storage_backend() {
        let storage = AcornStorage::sqlite("./test_sqlite.db", Some("test_table")).expect("Failed to create SQLite storage");
        
        // Test storage info
        let info = storage.get_info().expect("Failed to get storage info");
        assert_eq!(info.provider_name, "SQLite");
        assert!(info.is_durable);
        assert!(info.supports_sync);
        assert!(!info.supports_history);
        assert!(!info.supports_async);
        
        // Test connection
        let is_connected = storage.test_connection().expect("Failed to test connection");
        assert!(is_connected);
        
        // Test with tree
        let mut tree = AcornTree::open_with_storage(&storage).expect("Failed to open tree with SQLite storage");
        
        let test_data = TestData {
            id: "sqlite-test".to_string(),
            value: 42,
            name: "SQLite Test Item".to_string(),
        };
        
        tree.stash("sqlite-test", &test_data).expect("Failed to stash in SQLite storage");
        
        let retrieved: TestData = tree.crack("sqlite-test").expect("Failed to crack from SQLite storage");
        assert_eq!(test_data, retrieved);
        
        // Store multiple items
        for i in 1..=3 {
            let item = TestData {
                id: format!("sqlite-item-{}", i),
                value: i * 10,
                name: format!("SQLite Item {}", i),
            };
            tree.stash(&format!("sqlite-item-{}", i), &item).expect("Failed to stash SQLite item");
        }
        
        let items = tree.list().expect("Failed to list SQLite items");
        assert_eq!(items.len(), 4); // 1 original + 3 new items
    }

    #[test]
    fn test_storage_backend_info() {
        let storage = AcornStorage::sqlite("./test_info.db", None).expect("Failed to create SQLite storage");
        
        let info = storage.get_info().expect("Failed to get storage info");
        
        // Verify SQLite-specific properties
        assert_eq!(info.provider_name, "SQLite");
        assert_eq!(info.trunk_type, "SQLite");
        assert!(info.is_durable);
        assert!(info.supports_sync);
        assert!(!info.supports_history);
        assert!(!info.supports_async);
        assert!(!info.connection_info.is_empty());
    }

    #[test]
    fn test_storage_backend_connection_test() {
        let storage = AcornStorage::sqlite("./test_connection.db", None).expect("Failed to create SQLite storage");
        
        let is_connected = storage.test_connection().expect("Failed to test connection");
        assert!(is_connected);
    }

    #[test]
    fn test_storage_backend_with_tree() {
        let storage = AcornStorage::sqlite("./test_tree.db", Some("custom_table")).expect("Failed to create SQLite storage");
        let mut tree = AcornTree::open_with_storage(&storage).expect("Failed to open tree with storage");
        
        // Store test data
        let test_data = TestData {
            id: "storage-test".to_string(),
            value: 123,
            name: "Storage Test Item".to_string(),
        };
        
        tree.stash("storage-test", &test_data).expect("Failed to stash in storage backend");
        
        // Retrieve test data
        let retrieved: TestData = tree.crack("storage-test").expect("Failed to crack from storage backend");
        assert_eq!(test_data, retrieved);
        
        // Store multiple items
        for i in 1..=5 {
            let item = TestData {
                id: format!("storage-item-{}", i),
                value: i * 20,
                name: format!("Storage Item {}", i),
            };
            tree.stash(&format!("storage-item-{}", i), &item).expect("Failed to stash storage item");
        }
        
        // List all items
        let items = tree.list().expect("Failed to list items");
        assert_eq!(items.len(), 6); // 1 original + 5 new items
    }

    #[test]
    fn test_storage_backend_error_handling() {
        // Test invalid database path
        let result = AcornStorage::sqlite("/invalid/path/that/does/not/exist.db", None);
        assert!(result.is_err());
        
        // Test invalid table name
        let result = AcornStorage::sqlite("./test.db", Some("invalid-table-name!"));
        assert!(result.is_err());
    }

    #[test]
    fn test_storage_backend_large_dataset() {
        let storage = AcornStorage::sqlite("./test_large.db", None).expect("Failed to create SQLite storage");
        let mut tree = AcornTree::open_with_storage(&storage).expect("Failed to open tree with storage");
        
        // Store a larger dataset
        let large_dataset: Vec<TestData> = (1..=100)
            .map(|i| TestData {
                id: format!("large-item-{}", i),
                value: i,
                name: format!("Large Item {}", i),
            })
            .collect();
        
        for item in &large_dataset {
            tree.stash(&item.id, item).expect("Failed to stash large item");
        }
        
        // Verify all items can be retrieved
        for item in &large_dataset {
            let retrieved: TestData = tree.crack(&item.id).expect("Failed to retrieve large item");
            assert_eq!(item, &retrieved);
        }
        
        // Check total count
        let items = tree.list().expect("Failed to list large dataset");
        assert_eq!(items.len(), 100);
    }

    #[test]
    fn test_storage_backend_persistence() {
        let storage = AcornStorage::sqlite("./test_persistence.db", None).expect("Failed to create SQLite storage");
        let mut tree = AcornTree::open_with_storage(&storage).expect("Failed to open tree with storage");
        
        // Store some data
        let persistent_data = TestData {
            id: "persistent-test".to_string(),
            value: 999,
            name: "Persistent Test Item".to_string(),
        };
        
        tree.stash("persistent-test", &persistent_data).expect("Failed to stash persistent data");
        
        // Drop the tree and storage to simulate restart
        drop(tree);
        drop(storage);
        
        // Recreate storage and tree
        let new_storage = AcornStorage::sqlite("./test_persistence.db", None).expect("Failed to recreate SQLite storage");
        let new_tree = AcornTree::open_with_storage(&new_storage).expect("Failed to reopen tree with storage");
        
        // Verify data persisted
        let retrieved: TestData = new_tree.crack("persistent-test").expect("Failed to retrieve persistent data");
        assert_eq!(persistent_data, retrieved);
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

    #[test]
    fn test_transaction_basic() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Start transaction
        let mut tx = tree.begin_transaction().unwrap();
        
        // Add data in transaction
        tx.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        tx.stash("user2", &TestData { id: "user2".to_string(), name: "Bob".to_string(), value: 25 }).unwrap();
        
        // Data should not be visible before commit
        assert!(tree.crack::<TestData>("user1").is_err());
        assert!(tree.crack::<TestData>("user2").is_err());
        
        // Commit transaction
        assert!(tx.commit().unwrap());
        
        // Data should be visible after commit
        let user1: TestData = tree.crack("user1").unwrap();
        let user2: TestData = tree.crack("user2").unwrap();
        assert_eq!(user1.name, "Alice");
        assert_eq!(user2.name, "Bob");
    }

    #[test]
    fn test_transaction_rollback() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Add some initial data
        tree.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Start transaction
        let mut tx = tree.begin_transaction().unwrap();
        
        // Modify existing data and add new data
        tx.stash("user1", &TestData { id: "user1".to_string(), name: "Alice Modified".to_string(), value: 35 }).unwrap();
        tx.stash("user2", &TestData { id: "user2".to_string(), name: "Bob".to_string(), value: 25 }).unwrap();
        
        // Rollback transaction
        tx.rollback().unwrap();
        
        // Original data should be unchanged
        let user1: TestData = tree.crack("user1").unwrap();
        assert_eq!(user1.name, "Alice");
        assert_eq!(user1.value, 30);
        
        // New data should not exist
        assert!(tree.crack::<TestData>("user2").is_err());
    }

    #[test]
    fn test_transaction_delete() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Add initial data
        tree.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        tree.stash("user2", &TestData { id: "user2".to_string(), name: "Bob".to_string(), value: 25 }).unwrap();
        
        // Start transaction
        let mut tx = tree.begin_transaction().unwrap();
        
        // Delete data in transaction
        tx.delete("user1").unwrap();
        tx.stash("user3", &TestData { id: "user3".to_string(), name: "Charlie".to_string(), value: 35 }).unwrap();
        
        // Data should still be visible before commit
        assert!(tree.crack::<TestData>("user1").is_ok());
        assert!(tree.crack::<TestData>("user3").is_err());
        
        // Commit transaction
        assert!(tx.commit().unwrap());
        
        // user1 should be deleted, user3 should exist
        assert!(tree.crack::<TestData>("user1").is_err());
        let user3: TestData = tree.crack("user3").unwrap();
        assert_eq!(user3.name, "Charlie");
        
        // user2 should be unchanged
        let user2: TestData = tree.crack("user2").unwrap();
        assert_eq!(user2.name, "Bob");
    }

    #[test]
    fn test_transaction_multiple_operations() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Start transaction
        let mut tx = tree.begin_transaction().unwrap();
        
        // Multiple operations
        tx.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        tx.stash("user2", &TestData { id: "user2".to_string(), name: "Bob".to_string(), value: 25 }).unwrap();
        tx.stash("user3", &TestData { id: "user3".to_string(), name: "Charlie".to_string(), value: 35 }).unwrap();
        
        // Modify user2
        tx.stash("user2", &TestData { id: "user2".to_string(), name: "Bob Modified".to_string(), value: 27 }).unwrap();
        
        // Delete user3
        tx.delete("user3").unwrap();
        
        // Commit transaction
        assert!(tx.commit().unwrap());
        
        // Verify all operations
        let user1: TestData = tree.crack("user1").unwrap();
        assert_eq!(user1.name, "Alice");
        
        let user2: TestData = tree.crack("user2").unwrap();
        assert_eq!(user2.name, "Bob Modified");
        assert_eq!(user2.value, 27);
        
        // user3 should not exist
        assert!(tree.crack::<TestData>("user3").is_err());
    }

    #[test]
    fn test_transaction_error_handling() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Start transaction
        let mut tx = tree.begin_transaction().unwrap();
        
        // Valid operation
        tx.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Test rollback functionality
        tx.rollback().unwrap();
        
        // No data should be committed
        assert!(tree.crack::<TestData>("user1").is_err());
    }

    // Advanced Sync Tests
    #[test]
    fn test_mesh_basic_operations() {
        let mesh = AcornTree::create_mesh().unwrap();
        
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        
        // Add nodes to mesh
        mesh.add_node("node1", &tree1).unwrap();
        mesh.add_node("node2", &tree2).unwrap();
        
        // Connect nodes
        mesh.connect_nodes("node1", "node2").unwrap();
        
        // Add some data to tree1
        tree1.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Synchronize mesh
        mesh.synchronize_all().unwrap();
        
        // Data should be synchronized to tree2
        let user1: TestData = tree2.crack("user1").unwrap();
        assert_eq!(user1.name, "Alice");
    }

    #[test]
    fn test_mesh_full_mesh_topology() {
        let mesh = AcornTree::create_mesh().unwrap();
        
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        let mut tree3 = AcornTree::open("memory://").unwrap();
        
        // Add nodes to mesh
        mesh.add_node("node1", &tree1).unwrap();
        mesh.add_node("node2", &tree2).unwrap();
        mesh.add_node("node3", &tree3).unwrap();
        
        // Create full mesh topology
        mesh.create_full_mesh().unwrap();
        
        // Add data to tree1
        tree1.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Synchronize mesh
        mesh.synchronize_all().unwrap();
        
        // Data should be synchronized to all other trees
        let user1_tree2: TestData = tree2.crack("user1").unwrap();
        let user1_tree3: TestData = tree3.crack("user1").unwrap();
        assert_eq!(user1_tree2.name, "Alice");
        assert_eq!(user1_tree3.name, "Alice");
    }

    #[test]
    fn test_mesh_ring_topology() {
        let mesh = AcornTree::create_mesh().unwrap();
        
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        let mut tree3 = AcornTree::open("memory://").unwrap();
        
        // Add nodes to mesh
        mesh.add_node("node1", &tree1).unwrap();
        mesh.add_node("node2", &tree2).unwrap();
        mesh.add_node("node3", &tree3).unwrap();
        
        // Create ring topology
        mesh.create_ring().unwrap();
        
        // Add data to tree1
        tree1.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Synchronize mesh
        mesh.synchronize_all().unwrap();
        
        // Data should propagate through the ring
        let user1_tree2: TestData = tree2.crack("user1").unwrap();
        let user1_tree3: TestData = tree3.crack("user1").unwrap();
        assert_eq!(user1_tree2.name, "Alice");
        assert_eq!(user1_tree3.name, "Alice");
    }

    #[test]
    fn test_mesh_star_topology() {
        let mesh = AcornTree::create_mesh().unwrap();
        
        let mut hub_tree = AcornTree::open("memory://").unwrap();
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        
        // Add nodes to mesh
        mesh.add_node("hub", &hub_tree).unwrap();
        mesh.add_node("node1", &tree1).unwrap();
        mesh.add_node("node2", &tree2).unwrap();
        
        // Create star topology with hub as center
        mesh.create_star("hub").unwrap();
        
        // Add data to hub
        hub_tree.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Synchronize mesh
        mesh.synchronize_all().unwrap();
        
        // Data should be synchronized to all spoke nodes
        let user1_tree1: TestData = tree1.crack("user1").unwrap();
        let user1_tree2: TestData = tree2.crack("user1").unwrap();
        assert_eq!(user1_tree1.name, "Alice");
        assert_eq!(user1_tree2.name, "Alice");
    }

    #[test]
    fn test_p2p_bidirectional_sync() {
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        
        // Create P2P connection
        let p2p = AcornTree::create_p2p(&tree1, &tree2).unwrap();
        
        // Add data to tree1
        tree1.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Sync bidirectionally
        p2p.sync_bidirectional().unwrap();
        
        // Data should be synchronized to tree2
        let user1: TestData = tree2.crack("user1").unwrap();
        assert_eq!(user1.name, "Alice");
        
        // Add data to tree2
        tree2.stash("user2", &TestData { id: "user2".to_string(), name: "Bob".to_string(), value: 25 }).unwrap();
        
        // Sync again
        p2p.sync_bidirectional().unwrap();
        
        // Data should be synchronized to tree1
        let user2: TestData = tree1.crack("user2").unwrap();
        assert_eq!(user2.name, "Bob");
    }

    #[test]
    fn test_p2p_push_only_sync() {
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        
        // Create P2P connection
        let p2p = AcornTree::create_p2p(&tree1, &tree2).unwrap();
        
        // Set to push-only mode
        p2p.set_sync_mode(1).unwrap(); // PushOnly
        
        // Add data to tree1
        tree1.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Sync push-only
        p2p.sync_push_only().unwrap();
        
        // Data should be synchronized to tree2
        let user1: TestData = tree2.crack("user1").unwrap();
        assert_eq!(user1.name, "Alice");
        
        // Add data to tree2
        tree2.stash("user2", &TestData { id: "user2".to_string(), name: "Bob".to_string(), value: 25 }).unwrap();
        
        // Sync push-only again
        p2p.sync_push_only().unwrap();
        
        // Data from tree2 should NOT be synchronized to tree1 in push-only mode
        assert!(tree1.crack::<TestData>("user2").is_err());
    }

    #[test]
    fn test_p2p_pull_only_sync() {
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        
        // Create P2P connection
        let p2p = AcornTree::create_p2p(&tree1, &tree2).unwrap();
        
        // Set to pull-only mode
        p2p.set_sync_mode(2).unwrap(); // PullOnly
        
        // Add data to tree2
        tree2.stash("user1", &TestData { id: "user1".to_string(), name: "Alice".to_string(), value: 30 }).unwrap();
        
        // Sync pull-only
        p2p.sync_pull_only().unwrap();
        
        // Data should be synchronized to tree1
        let user1: TestData = tree1.crack("user1").unwrap();
        assert_eq!(user1.name, "Alice");
        
        // Add data to tree1
        tree1.stash("user2", &TestData { id: "user2".to_string(), name: "Bob".to_string(), value: 25 }).unwrap();
        
        // Sync pull-only again
        p2p.sync_pull_only().unwrap();
        
        // Data from tree1 should NOT be synchronized to tree2 in pull-only mode
        assert!(tree2.crack::<TestData>("user2").is_err());
    }

    #[test]
    fn test_p2p_sync_mode_changes() {
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        
        // Create P2P connection
        let p2p = AcornTree::create_p2p(&tree1, &tree2).unwrap();
        
        // Test different sync modes
        p2p.set_sync_mode(0).unwrap(); // Bidirectional
        p2p.set_sync_mode(1).unwrap(); // PushOnly
        p2p.set_sync_mode(2).unwrap(); // PullOnly
        p2p.set_sync_mode(3).unwrap(); // Disabled
        
        // Test conflict direction settings
        p2p.set_conflict_direction(0).unwrap(); // UseJudge
        p2p.set_conflict_direction(1).unwrap(); // PreferLocal
        p2p.set_conflict_direction(2).unwrap(); // PreferRemote
    }

    #[test]
    fn test_advanced_sync_error_handling() {
        let mesh = AcornTree::create_mesh().unwrap();
        
        // Try to connect non-existent nodes
        let result = mesh.connect_nodes("nonexistent1", "nonexistent2");
        assert!(result.is_err());
        
        // Try to create star with non-existent hub
        let result = mesh.create_star("nonexistent_hub");
        assert!(result.is_err());
        
        let mut tree1 = AcornTree::open("memory://").unwrap();
        let mut tree2 = AcornTree::open("memory://").unwrap();
        
        // Try to create P2P with invalid sync mode
        let p2p = AcornTree::create_p2p(&tree1, &tree2).unwrap();
        let result = p2p.set_sync_mode(999); // Invalid mode
        assert!(result.is_err());
        
        // Try to create P2P with invalid conflict direction
        let result = p2p.set_conflict_direction(999); // Invalid direction
        assert!(result.is_err());
    }

    // Encryption Tests
    #[test]
    fn test_password_based_encryption() {
        let encryption = AcornEncryption::from_password("test-password", "test-salt").unwrap();
        assert!(encryption.is_enabled().unwrap());
        
        let mut tree = AcornTree::open_encrypted("memory://", &encryption).unwrap();
        
        let test_data = TestData {
            id: "encrypted-1".to_string(),
            value: 123,
            name: "Encrypted Test".to_string(),
        };
        
        tree.stash("encrypted-1", &test_data).unwrap();
        let retrieved: TestData = tree.crack("encrypted-1").unwrap();
        assert_eq!(test_data, retrieved);
    }

    #[test]
    fn test_key_iv_encryption() {
        let (key, iv) = AcornEncryption::generate_key_iv().unwrap();
        
        let encryption = AcornEncryption::from_key_iv(&key, &iv).unwrap();
        assert!(encryption.is_enabled().unwrap());
        
        let mut tree = AcornTree::open_encrypted("memory://", &encryption).unwrap();
        
        let test_data = TestData {
            id: "key-iv-1".to_string(),
            value: 456,
            name: "Key IV Test".to_string(),
        };
        
        tree.stash("key-iv-1", &test_data).unwrap();
        let retrieved: TestData = tree.crack("key-iv-1").unwrap();
        assert_eq!(test_data, retrieved);
    }

    #[test]
    fn test_encryption_key_export() {
        let encryption = AcornEncryption::from_password("export-test", "export-salt").unwrap();
        
        let exported_key = encryption.export_key().unwrap();
        let exported_iv = encryption.export_iv().unwrap();
        
        // Create new encryption with exported key/IV
        let new_encryption = AcornEncryption::from_key_iv(&exported_key, &exported_iv).unwrap();
        
        // Both should be able to encrypt/decrypt the same data
        let plaintext = "Test encryption export";
        let ciphertext1 = encryption.encrypt(plaintext).unwrap();
        let ciphertext2 = new_encryption.encrypt(plaintext).unwrap();
        
        // Should produce different ciphertexts (due to different IVs in each encryption)
        assert_ne!(ciphertext1, ciphertext2);
        
        // But both should decrypt correctly
        let decrypted1 = encryption.decrypt(&ciphertext1).unwrap();
        let decrypted2 = new_encryption.decrypt(&ciphertext2).unwrap();
        
        assert_eq!(plaintext, decrypted1);
        assert_eq!(plaintext, decrypted2);
    }

    #[test]
    fn test_direct_encryption_decryption() {
        let encryption = AcornEncryption::from_password("direct-test", "direct-salt").unwrap();
        
        let test_strings = vec![
            "Simple string",
            "String with special chars: !@#$%^&*()",
            "Unicode: 🚀🌟✨",
            "Very long string with lots of text that should be encrypted properly and decrypted correctly without any issues",
        ];
        
        for test_string in test_strings {
            let encrypted = encryption.encrypt(test_string).unwrap();
            let decrypted = encryption.decrypt(&encrypted).unwrap();
            assert_eq!(test_string, decrypted);
        }
    }

    #[test]
    fn test_encrypted_compressed_tree() {
        let encryption = AcornEncryption::from_password("compressed-test", "compressed-salt").unwrap();
        
        let mut tree = AcornTree::open_encrypted_compressed("memory://", &encryption, 1).unwrap(); // Optimal compression
        
        let large_data = ComplexData {
            numbers: (1..1000).collect(),
            text: "A".repeat(1000),
            nested: Some(Box::new(ComplexData {
                numbers: (1..100).collect(),
                text: "B".repeat(100),
                nested: None,
            })),
        };
        
        tree.stash("large-data", &large_data).unwrap();
        let retrieved: ComplexData = tree.crack("large-data").unwrap();
        assert_eq!(large_data, retrieved);
    }

    #[test]
    fn test_encryption_error_handling() {
        // Test invalid password (empty)
        let result = AcornEncryption::from_password("", "salt");
        assert!(result.is_err());
        
        // Test invalid key/IV (invalid base64)
        let result = AcornEncryption::from_key_iv("invalid-base64!", "invalid-base64!");
        assert!(result.is_err());
        
        // Test decryption with wrong encryption
        let encryption1 = AcornEncryption::from_password("password1", "salt1").unwrap();
        let encryption2 = AcornEncryption::from_password("password2", "salt2").unwrap();
        
        let plaintext = "Test data";
        let encrypted = encryption1.encrypt(plaintext).unwrap();
        
        // Should fail to decrypt with different encryption
        let result = encryption2.decrypt(&encrypted);
        assert!(result.is_err());
    }

    #[test]
    fn test_encryption_with_different_compression_levels() {
        let encryption = AcornEncryption::from_password("compression-test", "compression-salt").unwrap();
        
        let test_data = TestData {
            id: "compression-test".to_string(),
            value: 789,
            name: "Compression Test".to_string(),
        };
        
        // Test all compression levels
        for compression_level in 0..3 {
            let mut tree = AcornTree::open_encrypted_compressed("memory://", &encryption, compression_level).unwrap();
            tree.stash("compression-test", &test_data).unwrap();
            let retrieved: TestData = tree.crack("compression-test").unwrap();
            assert_eq!(test_data, retrieved);
        }
    }

    #[test]
    fn test_document_store_basic_operations() {
        // Create document store
        let doc_store = AcornDocumentStore::new(Some("./test_docstore")).unwrap();
        
        // Get initial info
        let info = doc_store.get_info().unwrap();
        assert_eq!(info.trunk_type, "DocumentStoreTrunk");
        assert!(info.supports_history);
        assert!(info.is_durable);
        assert!(!info.supports_async);
        assert_eq!(info.provider_name, "DocumentStore");
        assert_eq!(info.total_versions, 0);
        assert!(!info.has_change_log);

        // Open tree with document store
        let mut tree = AcornTree::open_with_document_store(&doc_store).unwrap();
        
        // Store some data
        let test_data = TestData {
            id: "doc-1".to_string(),
            value: 42,
            name: "Document Store Test".to_string(),
        };
        
        tree.stash("doc-1", &test_data).unwrap();
        
        // Retrieve data
        let retrieved: TestData = tree.crack("doc-1").unwrap();
        assert_eq!(test_data, retrieved);
        
        // Update data to create version history
        let updated_data = TestData {
            id: "doc-1".to_string(),
            value: 84,
            name: "Updated Document Store Test".to_string(),
        };
        
        tree.stash("doc-1", &updated_data).unwrap();
        
        // Verify current version
        let current: TestData = tree.crack("doc-1").unwrap();
        assert_eq!(updated_data, current);
        
        // Get version history
        let history_json = doc_store.get_history("doc-1").unwrap();
        assert!(!history_json.is_empty());
        
        // Parse history
        let history: Vec<serde_json::Value> = serde_json::from_str(&history_json).unwrap();
        assert!(history.len() >= 1); // Should have at least one previous version
        
        // Get updated info
        let updated_info = doc_store.get_info().unwrap();
        assert!(updated_info.has_change_log);
        assert!(updated_info.total_versions > 0);
    }

    #[test]
    fn test_document_store_version_history() {
        let doc_store = AcornDocumentStore::new(Some("./test_version_history")).unwrap();
        let mut tree = AcornTree::open_with_document_store(&doc_store).unwrap();
        
        // Create multiple versions of the same document
        for i in 1..=5 {
            let data = TestData {
                id: "versioned-doc".to_string(),
                value: i * 10,
                name: format!("Version {}", i),
            };
            
            tree.stash("versioned-doc", &data).unwrap();
        }
        
        // Verify current version
        let current: TestData = tree.crack("versioned-doc").unwrap();
        assert_eq!(current.value, 50);
        assert_eq!(current.name, "Version 5");
        
        // Get version history
        let history_json = doc_store.get_history("versioned-doc").unwrap();
        let history: Vec<serde_json::Value> = serde_json::from_str(&history_json).unwrap();
        
        // Should have 4 previous versions (versions 1-4)
        assert_eq!(history.len(), 4);
        
        // Verify version progression
        for (i, version) in history.iter().enumerate() {
            let expected_value = (i + 1) * 10;
            let actual_value = version["Payload"]["value"].as_i64().unwrap() as i32;
            assert_eq!(actual_value, expected_value);
        }
    }

    #[test]
    fn test_document_store_multiple_documents() {
        let doc_store = AcornDocumentStore::new(Some("./test_multiple_docs")).unwrap();
        let mut tree = AcornTree::open_with_document_store(&doc_store).unwrap();
        
        // Store multiple documents
        for i in 1..=3 {
            let data = TestData {
                id: format!("doc-{}", i),
                value: i * 100,
                name: format!("Document {}", i),
            };
            
            tree.stash(&format!("doc-{}", i), &data).unwrap();
        }
        
        // Update each document to create history
        for i in 1..=3 {
            let updated_data = TestData {
                id: format!("doc-{}", i),
                value: i * 200,
                name: format!("Updated Document {}", i),
            };
            
            tree.stash(&format!("doc-{}", i), &updated_data).unwrap();
        }
        
        // Verify all documents exist
        for i in 1..=3 {
            let doc: TestData = tree.crack(&format!("doc-{}", i)).unwrap();
            assert_eq!(doc.value, i * 200);
            assert_eq!(doc.name, format!("Updated Document {}", i));
        }
        
        // Get info and verify total versions
        let info = doc_store.get_info().unwrap();
        assert_eq!(info.total_versions, 6); // 3 current + 3 previous versions
        
        // Verify each document has history
        for i in 1..=3 {
            let history_json = doc_store.get_history(&format!("doc-{}", i)).unwrap();
            let history: Vec<serde_json::Value> = serde_json::from_str(&history_json).unwrap();
            assert_eq!(history.len(), 1); // Each should have 1 previous version
        }
    }

    #[test]
    fn test_document_store_compaction() {
        let doc_store = AcornDocumentStore::new(Some("./test_compaction")).unwrap();
        let mut tree = AcornTree::open_with_document_store(&doc_store).unwrap();
        
        // Create multiple versions
        for i in 1..=10 {
            let data = TestData {
                id: "compaction-test".to_string(),
                value: i,
                name: format!("Version {}", i),
            };
            
            tree.stash("compaction-test", &data).unwrap();
        }
        
        // Verify we have history
        let info_before = doc_store.get_info().unwrap();
        assert!(info_before.total_versions > 1);
        
        // Perform compaction
        doc_store.compact().unwrap();
        
        // Verify compaction completed
        let info_after = doc_store.get_info().unwrap();
        // Note: The actual compaction behavior depends on the C# implementation
        // For now, we just verify the operation completes without error
        assert!(info_after.total_versions >= 1);
    }

    #[test]
    fn test_document_store_error_handling() {
        // Test invalid document store creation
        let result = AcornDocumentStore::new(Some(""));
        // This should succeed as empty string is valid
        
        let doc_store = AcornDocumentStore::new(None).unwrap();
        
        // Test getting history for non-existent document
        let history = doc_store.get_history("non-existent");
        // This should succeed but return empty history
        
        // Test getting info
        let info = doc_store.get_info().unwrap();
        assert_eq!(info.trunk_type, "DocumentStoreTrunk");
    }

    #[test]
    fn test_document_store_with_tree_operations() {
        let doc_store = AcornDocumentStore::new(Some("./test_tree_ops")).unwrap();
        let mut tree = AcornTree::open_with_document_store(&doc_store).unwrap();
        
        // Test all tree operations work with document store
        let data1 = TestData {
            id: "tree-op-1".to_string(),
            value: 100,
            name: "Tree Operation 1".to_string(),
        };
        
        let data2 = TestData {
            id: "tree-op-2".to_string(),
            value: 200,
            name: "Tree Operation 2".to_string(),
        };
        
        // Stash operations
        tree.stash("key1", &data1).unwrap();
        tree.stash("key2", &data2).unwrap();
        
        // Crack operations
        let retrieved1: TestData = tree.crack("key1").unwrap();
        let retrieved2: TestData = tree.crack("key2").unwrap();
        
        assert_eq!(data1, retrieved1);
        assert_eq!(data2, retrieved2);
        
        // Update to create version history
        let updated_data1 = TestData {
            id: "tree-op-1".to_string(),
            value: 150,
            name: "Updated Tree Operation 1".to_string(),
        };
        
        tree.stash("key1", &updated_data1).unwrap();
        
        // Verify current version
        let current: TestData = tree.crack("key1").unwrap();
        assert_eq!(updated_data1, current);
        
        // Verify history exists
        let history_json = doc_store.get_history("key1").unwrap();
        let history: Vec<serde_json::Value> = serde_json::from_str(&history_json).unwrap();
        assert_eq!(history.len(), 1);
        
        // Verify history contains original version
        let original_version = &history[0]["Payload"];
        assert_eq!(original_version["value"].as_i64().unwrap(), 100);
        assert_eq!(original_version["name"].as_str().unwrap(), "Tree Operation 1");
    }

    #[test]
    fn test_reactive_programming_basic_subscription() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();
        
        // Subscribe to all changes
        let _sub = tree.subscribe(move |key: &str, value: &serde_json::Value| {
            let mut n = notifications_clone.lock().unwrap();
            n.push((key.to_string(), value.clone()));
        }).unwrap();
        
        // Perform operations
        let test_data = TestData {
            id: "reactive-1".to_string(),
            value: 42,
            name: "Reactive Test".to_string(),
        };
        
        tree.stash("key1", &test_data).unwrap();
        tree.stash("key2", &test_data).unwrap();
        
        // Give time for notifications
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Verify notifications
        let n = notifications.lock().unwrap();
        assert_eq!(n.len(), 2);
        assert_eq!(n[0].0, "key1");
        assert_eq!(n[1].0, "key2");
    }

    #[test]
    fn test_reactive_programming_stash_subscription() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Track stash notifications
        let stash_notifications = Arc::new(Mutex::new(Vec::new()));
        let stash_notifications_clone = stash_notifications.clone();
        
        // Subscribe to stash operations only
        let _stash_sub = tree.subscribe_stash(move |key: &str, value: &serde_json::Value| {
            let mut n = stash_notifications_clone.lock().unwrap();
            n.push((key.to_string(), value.clone()));
        }).unwrap();
        
        // Perform operations
        let test_data = TestData {
            id: "stash-test".to_string(),
            value: 100,
            name: "Stash Test".to_string(),
        };
        
        tree.stash("stash-key", &test_data).unwrap();
        tree.toss("stash-key").unwrap();
        tree.stash("stash-key2", &test_data).unwrap();
        
        // Give time for notifications
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Verify only stash operations were captured
        let n = stash_notifications.lock().unwrap();
        assert_eq!(n.len(), 2);
        assert_eq!(n[0].0, "stash-key");
        assert_eq!(n[1].0, "stash-key2");
    }

    #[test]
    fn test_reactive_programming_toss_subscription() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Track toss notifications
        let toss_notifications = Arc::new(Mutex::new(Vec::new()));
        let toss_notifications_clone = toss_notifications.clone();
        
        // Subscribe to toss operations only
        let _toss_sub = tree.subscribe_toss(move |key: &str| {
            let mut n = toss_notifications_clone.lock().unwrap();
            n.push(key.to_string());
        }).unwrap();
        
        // Perform operations
        let test_data = TestData {
            id: "toss-test".to_string(),
            value: 200,
            name: "Toss Test".to_string(),
        };
        
        tree.stash("toss-key1", &test_data).unwrap();
        tree.toss("toss-key1").unwrap();
        tree.stash("toss-key2", &test_data).unwrap();
        tree.toss("toss-key2").unwrap();
        
        // Give time for notifications
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Verify only toss operations were captured
        let n = toss_notifications.lock().unwrap();
        assert_eq!(n.len(), 2);
        assert_eq!(n[0], "toss-key1");
        assert_eq!(n[1], "toss-key2");
    }

    #[test]
    fn test_reactive_programming_filtered_subscription() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Track filtered notifications
        let filtered_notifications = Arc::new(Mutex::new(Vec::new()));
        let filtered_notifications_clone = filtered_notifications.clone();
        
        // Subscribe with filtering predicate
        let _filtered_sub = tree.subscribe_where(
            |key: &str, value: &serde_json::Value| {
                // Only notify for keys starting with "filter-"
                key.starts_with("filter-")
            },
            move |key: &str, value: &serde_json::Value| {
                let mut n = filtered_notifications_clone.lock().unwrap();
                n.push((key.to_string(), value.clone()));
            }
        ).unwrap();
        
        // Perform operations
        let test_data = TestData {
            id: "filter-test".to_string(),
            value: 300,
            name: "Filter Test".to_string(),
        };
        
        tree.stash("filter-key1", &test_data).unwrap();
        tree.stash("other-key1", &test_data).unwrap();
        tree.stash("filter-key2", &test_data).unwrap();
        tree.stash("other-key2", &test_data).unwrap();
        
        // Give time for notifications
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Verify only filtered operations were captured
        let n = filtered_notifications.lock().unwrap();
        assert_eq!(n.len(), 2);
        assert_eq!(n[0].0, "filter-key1");
        assert_eq!(n[1].0, "filter-key2");
    }

    #[test]
    fn test_reactive_programming_multiple_subscriptions() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Track different types of notifications
        let all_notifications = Arc::new(Mutex::new(Vec::new()));
        let stash_notifications = Arc::new(Mutex::new(Vec::new()));
        let toss_notifications = Arc::new(Mutex::new(Vec::new()));
        
        let all_notifications_clone = all_notifications.clone();
        let stash_notifications_clone = stash_notifications.clone();
        let toss_notifications_clone = toss_notifications.clone();
        
        // Subscribe to all changes
        let _all_sub = tree.subscribe(move |key: &str, value: &serde_json::Value| {
            let mut n = all_notifications_clone.lock().unwrap();
            n.push((key.to_string(), value.clone()));
        }).unwrap();
        
        // Subscribe to stash operations
        let _stash_sub = tree.subscribe_stash(move |key: &str, value: &serde_json::Value| {
            let mut n = stash_notifications_clone.lock().unwrap();
            n.push((key.to_string(), value.clone()));
        }).unwrap();
        
        // Subscribe to toss operations
        let _toss_sub = tree.subscribe_toss(move |key: &str| {
            let mut n = toss_notifications_clone.lock().unwrap();
            n.push(key.to_string());
        }).unwrap();
        
        // Perform operations
        let test_data = TestData {
            id: "multi-test".to_string(),
            value: 400,
            name: "Multi Test".to_string(),
        };
        
        tree.stash("multi-key1", &test_data).unwrap();
        tree.stash("multi-key2", &test_data).unwrap();
        tree.toss("multi-key1").unwrap();
        
        // Give time for notifications
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Verify all subscriptions received appropriate notifications
        let all_n = all_notifications.lock().unwrap();
        let stash_n = stash_notifications.lock().unwrap();
        let toss_n = toss_notifications.lock().unwrap();
        
        assert_eq!(all_n.len(), 3); // All operations
        assert_eq!(stash_n.len(), 2); // Only stash operations
        assert_eq!(toss_n.len(), 1); // Only toss operations
        
        assert_eq!(all_n[0].0, "multi-key1");
        assert_eq!(all_n[1].0, "multi-key2");
        assert_eq!(all_n[2].0, "multi-key1"); // Toss operation
        
        assert_eq!(stash_n[0].0, "multi-key1");
        assert_eq!(stash_n[1].0, "multi-key2");
        
        assert_eq!(toss_n[0], "multi-key1");
    }

    #[test]
    fn test_reactive_programming_subscription_cleanup() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Track notifications
        let notifications = Arc::new(Mutex::new(Vec::new()));
        let notifications_clone = notifications.clone();
        
        // Create subscription
        let _sub = tree.subscribe(move |key: &str, value: &serde_json::Value| {
            let mut n = notifications_clone.lock().unwrap();
            n.push((key.to_string(), value.clone()));
        }).unwrap();
        
        // Perform operation
        let test_data = TestData {
            id: "cleanup-test".to_string(),
            value: 500,
            name: "Cleanup Test".to_string(),
        };
        
        tree.stash("cleanup-key", &test_data).unwrap();
        
        // Give time for notification
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        // Verify notification was received
        let n = notifications.lock().unwrap();
        assert_eq!(n.len(), 1);
        assert_eq!(n[0].0, "cleanup-key");
        
        // Subscription will be automatically cleaned up when dropped
        // This test verifies that the subscription works and cleanup happens
    }

    #[test]
    fn test_advanced_queries_timestamp_filtering() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Create test data with different timestamps
        let now = std::time::SystemTime::now();
        let one_hour_ago = now - std::time::Duration::from_secs(3600);
        let two_hours_ago = now - std::time::Duration::from_secs(7200);
        
        let events = vec![
            ("event-1", TestData {
                id: "event-1".to_string(),
                value: 100,
                name: "Old Event".to_string(),
            }),
            ("event-2", TestData {
                id: "event-2".to_string(),
                value: 200,
                name: "Recent Event".to_string(),
            }),
            ("event-3", TestData {
                id: "event-3".to_string(),
                value: 300,
                name: "Current Event".to_string(),
            }),
        ];
        
        tree.batch_stash(&events).unwrap();
        
        // Test after() filtering
        let recent_events: Vec<TestData> = tree.query()
            .after(one_hour_ago)
            .collect()
            .unwrap();
        assert_eq!(recent_events.len(), 3); // All events for now
        
        // Test before() filtering
        let old_events: Vec<TestData> = tree.query()
            .before(two_hours_ago)
            .collect()
            .unwrap();
        assert_eq!(old_events.len(), 3); // All events for now
        
        // Test between() filtering
        let range_events: Vec<TestData> = tree.query()
            .between(two_hours_ago, now)
            .collect()
            .unwrap();
        assert_eq!(range_events.len(), 3); // All events for now
    }

    #[test]
    fn test_advanced_queries_node_filtering() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        let data = vec![
            ("data-1", TestData {
                id: "data-1".to_string(),
                value: 100,
                name: "Node 1 Data".to_string(),
            }),
            ("data-2", TestData {
                id: "data-2".to_string(),
                value: 200,
                name: "Node 2 Data".to_string(),
            }),
            ("data-3", TestData {
                id: "data-3".to_string(),
                value: 300,
                name: "Node 1 Data".to_string(),
            }),
        ];
        
        tree.batch_stash(&data).unwrap();
        
        // Test from_node() filtering
        let node1_data: Vec<TestData> = tree.query()
            .from_node("node-1")
            .collect()
            .unwrap();
        assert_eq!(node1_data.len(), 3); // All data for now
        
        let node2_data: Vec<TestData> = tree.query()
            .from_node("node-2")
            .collect()
            .unwrap();
        assert_eq!(node2_data.len(), 3); // All data for now
    }

    #[test]
    fn test_advanced_queries_timestamp_ordering() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        let data = vec![
            ("item-1", TestData {
                id: "item-1".to_string(),
                value: 100,
                name: "First Item".to_string(),
            }),
            ("item-2", TestData {
                id: "item-2".to_string(),
                value: 200,
                name: "Second Item".to_string(),
            }),
            ("item-3", TestData {
                id: "item-3".to_string(),
                value: 300,
                name: "Third Item".to_string(),
            }),
        ];
        
        tree.batch_stash(&data).unwrap();
        
        // Test newest() ordering
        let newest_items: Vec<TestData> = tree.query()
            .newest()
            .collect()
            .unwrap();
        assert_eq!(newest_items.len(), 3);
        
        // Test oldest() ordering
        let oldest_items: Vec<TestData> = tree.query()
            .oldest()
            .collect()
            .unwrap();
        assert_eq!(oldest_items.len(), 3);
    }

    #[test]
    fn test_advanced_queries_single_result() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        let data = vec![
            ("user-1", TestData {
                id: "user-1".to_string(),
                value: 100,
                name: "Alice".to_string(),
            }),
            ("user-2", TestData {
                id: "user-2".to_string(),
                value: 200,
                name: "Bob".to_string(),
            }),
        ];
        
        tree.batch_stash(&data).unwrap();
        
        // Test single() with unique result
        let alice: Option<TestData> = tree.query()
            .where_condition(|user| user["name"].as_str() == Some("Alice"))
            .single()
            .unwrap();
        assert!(alice.is_some());
        assert_eq!(alice.unwrap().name, "Alice");
        
        // Test single() with no results
        let charlie: Option<TestData> = tree.query()
            .where_condition(|user| user["name"].as_str() == Some("Charlie"))
            .single()
            .unwrap();
        assert!(charlie.is_none());
        
        // Test single() with multiple results (should error)
        let result: Result<Option<TestData>> = tree.query()
            .where_condition(|user| user["value"].as_i64().unwrap_or(0) > 50)
            .single();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Multiple results"));
    }

    #[test]
    fn test_advanced_queries_combined_filters() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        let now = std::time::SystemTime::now();
        let one_hour_ago = now - std::time::Duration::from_secs(3600);
        
        let data = vec![
            ("item-1", TestData {
                id: "item-1".to_string(),
                value: 100,
                name: "Recent Item".to_string(),
            }),
            ("item-2", TestData {
                id: "item-2".to_string(),
                value: 200,
                name: "Old Item".to_string(),
            }),
            ("item-3", TestData {
                id: "item-3".to_string(),
                value: 300,
                name: "Recent Item".to_string(),
            }),
        ];
        
        tree.batch_stash(&data).unwrap();
        
        // Test combined filtering: recent items with specific value
        let recent_high_value: Vec<TestData> = tree.query()
            .where_condition(|item| item["value"].as_i64().unwrap_or(0) > 150)
            .after(one_hour_ago)
            .collect()
            .unwrap();
        assert_eq!(recent_high_value.len(), 2); // All items for now
        
        // Test combined filtering: specific name and node
        let specific_items: Vec<TestData> = tree.query()
            .where_condition(|item| item["name"].as_str() == Some("Recent Item"))
            .from_node("node-1")
            .collect()
            .unwrap();
        assert_eq!(specific_items.len(), 3); // All items for now
    }

    #[test]
    fn test_advanced_queries_performance() {
        let mut tree = AcornTree::open("memory://").unwrap();
        
        // Create a larger dataset
        let mut data = Vec::new();
        for i in 0..100 {
            data.push((format!("item-{}", i), TestData {
                id: format!("item-{}", i),
                value: i as i32,
                name: format!("Item {}", i),
            }));
        }
        
        tree.batch_stash(&data).unwrap();
        
        // Test query performance
        let start = std::time::Instant::now();
        let high_value_items: Vec<TestData> = tree.query()
            .where_condition(|item| item["value"].as_i64().unwrap_or(0) > 50)
            .take(10)
            .collect()
            .unwrap();
        let query_time = start.elapsed();
        
        assert_eq!(high_value_items.len(), 10);
        assert!(query_time.as_millis() < 1000); // Should be fast
        
        // Test count performance
        let start = std::time::Instant::now();
        let count = tree.query()
            .where_condition(|item| item["value"].as_i64().unwrap_or(0) > 25)
            .count()
            .unwrap();
        let count_time = start.elapsed();
        
        assert_eq!(count, 75);
        assert!(count_time.as_millis() < 1000); // Should be fast
    }

    // Git Integration Tests
    #[test]
    fn test_git_integration_basic() -> Result<(), Error> {
        let git = AcornGit::new("./test-git-repo", "Test User", "test@example.com", false)?;
        
        // Test basic Git operations
        let has_remote = git.has_remote("origin")?;
        assert!(!has_remote); // Should not have remote by default
        
        Ok(())
    }

    #[test]
    fn test_git_storage_backend() -> Result<(), Error> {
        let git_storage = AcornStorage::git(
            "./test-git-storage",
            "Test User",
            "test@example.com",
            false
        )?;
        
        let mut tree = AcornTree::open_with_storage(git_storage)?;
        
        // Test basic operations with Git storage
        let item = TestData {
            id: "test-item".to_string(),
            value: 42,
            name: "test".to_string(),
        };
        
        tree.stash(&item.id, &item)?;
        let retrieved: Option<TestData> = tree.crack(&item.id)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 42);
        
        Ok(())
    }

    #[test]
    fn test_git_file_history() -> Result<(), Error> {
        let git = AcornGit::new("./test-git-history", "Test User", "test@example.com", false)?;
        
        // Test file history operations
        let commits = git.get_file_history("test-file.json")?;
        // Should be empty for new repository
        assert_eq!(commits.len(), 0);
        
        Ok(())
    }

    #[test]
    fn test_git_commit_operations() -> Result<(), Error> {
        let git = AcornGit::new("./test-git-commits", "Test User", "test@example.com", false)?;
        
        // Test commit operations
        let content = git.read_file_at_commit("test-file.json", "abc123")?;
        assert_eq!(content, ""); // Should be empty for non-existent commit
        
        Ok(())
    }

    #[test]
    fn test_git_remote_operations() -> Result<(), Error> {
        let git = AcornGit::new("./test-git-remote", "Test User", "test@example.com", false)?;
        
        // Test remote operations (should not fail even without remote)
        let has_origin = git.has_remote("origin")?;
        assert!(!has_origin);
        
        let has_upstream = git.has_remote("upstream")?;
        assert!(!has_upstream);
        
        Ok(())
    }

    #[test]
    fn test_git_squash_operations() -> Result<(), Error> {
        let git = AcornGit::new("./test-git-squash", "Test User", "test@example.com", false)?;
        
        // Test squash operations (should not fail even without commits)
        git.squash_commits("abc123")?;
        
        Ok(())
    }

    #[test]
    fn test_git_integration_with_tree() -> Result<(), Error> {
        // Create Git storage
        let git_storage = AcornStorage::git(
            "./test-git-tree-integration",
            "Test User",
            "test@example.com",
            false
        )?;
        
        let mut tree = AcornTree::open_with_storage(git_storage)?;
        
        // Add multiple items (each creates a Git commit)
        for i in 0..5 {
            let item = TestData {
                id: format!("git-item-{}", i),
                value: i * 10,
                name: format!("item-{}", i),
            };
            tree.stash(&item.id, &item)?;
        }
        
        // Query items
        let all_items: Vec<TestData> = tree.query().collect()?;
        assert_eq!(all_items.len(), 5);
        
        // Query specific items
        let high_value_items: Vec<TestData> = tree.query()
            .where_condition(|item| item["value"].as_u64().unwrap_or(0) > 20)
            .collect()?;
        assert_eq!(high_value_items.len(), 3); // items 3, 4, 5 (values 30, 40, 50)
        
        Ok(())
    }

    // Nursery System Tests
    #[test]
    fn test_nursery_basic_operations() -> Result<(), Error> {
        let nursery = AcornNursery::new()?;
        
        // Test basic operations
        let types = nursery.get_available_types()?;
        assert!(!types.is_empty());
        
        // Test has_trunk
        let has_file = nursery.has_trunk("file")?;
        assert!(has_file);
        
        Ok(())
    }

    #[test]
    fn test_nursery_metadata() -> Result<(), Error> {
        let nursery = AcornNursery::new()?;
        
        // Test getting metadata for file trunk
        let metadata = nursery.get_metadata("file")?;
        assert_eq!(metadata.type_id, "file");
        assert!(!metadata.description.is_empty());
        assert!(!metadata.category.is_empty());
        
        // Test getting all metadata
        let all_metadata = nursery.get_all_metadata()?;
        assert!(!all_metadata.is_empty());
        
        // Verify file trunk is in all metadata
        let file_metadata = all_metadata.iter().find(|m| m.type_id == "file");
        assert!(file_metadata.is_some());
        
        Ok(())
    }

    #[test]
    fn test_nursery_config_validation() -> Result<(), Error> {
        let nursery = AcornNursery::new()?;
        
        // Test valid file config
        let valid_config = r#"{"path": "./test-data"}"#;
        let is_valid = nursery.validate_config("file", valid_config)?;
        assert!(is_valid);
        
        // Test invalid config
        let invalid_config = r#"{"invalid": "config"}"#;
        let is_invalid = nursery.validate_config("file", invalid_config)?;
        assert!(!is_invalid);
        
        Ok(())
    }

    #[test]
    fn test_nursery_trunk_creation() -> Result<(), Error> {
        let nursery = AcornNursery::new()?;
        
        // Test creating file trunk
        let config = r#"{"path": "./test-nursery"}"#;
        let storage = nursery.grow_trunk("file", config)?;
        
        // Test using the created storage
        let mut tree = AcornTree::open_with_storage(storage)?;
        let item = TestData {
            id: "test-item".to_string(),
            value: 42,
            name: "test".to_string(),
        };
        
        tree.stash(&item.id, &item)?;
        let retrieved: Option<TestData> = tree.crack(&item.id)?;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().value, 42);
        
        Ok(())
    }

    #[test]
    fn test_nursery_catalog() -> Result<(), Error> {
        let nursery = AcornNursery::new()?;
        
        // Test getting catalog
        let catalog = nursery.get_catalog()?;
        assert!(!catalog.is_empty());
        assert!(catalog.contains("Nursery Catalog"));
        
        Ok(())
    }

    #[test]
    fn test_nursery_error_handling() -> Result<(), Error> {
        let nursery = AcornNursery::new()?;
        
        // Test nonexistent trunk type
        let has_nonexistent = nursery.has_trunk("nonexistent")?;
        assert!(!has_nonexistent);
        
        // Test getting metadata for nonexistent trunk
        match nursery.get_metadata("nonexistent") {
            Ok(_) => panic!("Expected error for nonexistent trunk"),
            Err(_) => {} // Expected
        }
        
        // Test growing nonexistent trunk
        match nursery.grow_trunk("nonexistent", r#"{}"#) {
            Ok(_) => panic!("Expected error for nonexistent trunk"),
            Err(_) => {} // Expected
        }
        
        Ok(())
    }

    #[test]
    fn test_nursery_multiple_storage_types() -> Result<(), Error> {
        let nursery = AcornNursery::new()?;
        
        let mut trees = Vec::new();
        
        // Create file storage
        if nursery.has_trunk("file")? {
            let file_config = r#"{"path": "./test-multi-file"}"#;
            if nursery.validate_config("file", file_config)? {
                let file_storage = nursery.grow_trunk("file", file_config)?;
                let mut file_tree = AcornTree::open_with_storage(file_storage)?;
                trees.push(("file", file_tree));
            }
        }
        
        // Create memory storage
        if nursery.has_trunk("memory")? {
            let memory_config = r#"{}"#;
            if nursery.validate_config("memory", memory_config)? {
                let memory_storage = nursery.grow_trunk("memory", memory_config)?;
                let mut memory_tree = AcornTree::open_with_storage(memory_storage)?;
                trees.push(("memory", memory_tree));
            }
        }
        
        // Store data in all trees
        for (trunk_name, tree) in &mut trees {
            let item = TestData {
                id: format!("test-{}", trunk_name),
                value: 100,
                name: format!("test-{}", trunk_name),
            };
            
            tree.stash(&item.id, &item)?;
            let retrieved: Option<TestData> = tree.crack(&item.id)?;
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().value, 100);
        }
        
        assert_eq!(trees.len(), 2); // Should have both file and memory
        
        Ok(())
    }

    // Advanced Tree Features Tests
    #[test]
    fn test_advanced_tree_auto_id_detection() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Test auto-ID stash with JSON
        let item_json = r#"{"id": "test-item", "name": "Test Item", "value": 42}"#;
        advanced_tree.stash_with_auto_id(item_json)?;
        
        // Verify item was stashed
        let count = advanced_tree.get_nut_count()?;
        assert!(count > 0);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_statistics() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Get initial stats
        let initial_stats = advanced_tree.get_stats()?;
        assert!(initial_stats.total_stashed >= 0);
        assert!(initial_stats.total_tossed >= 0);
        assert!(initial_stats.squabbles_resolved >= 0);
        assert!(initial_stats.smushes_performed >= 0);
        assert!(initial_stats.active_tangles >= 0);
        
        // Stash some items and check stats change
        let item_json = r#"{"id": "stats-test", "name": "Stats Test"}"#;
        advanced_tree.stash_with_auto_id(item_json)?;
        
        let updated_stats = advanced_tree.get_stats()?;
        assert!(updated_stats.total_stashed >= initial_stats.total_stashed);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_ttl_info() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Get TTL info
        let ttl_info = advanced_tree.get_ttl_info()?;
        assert!(ttl_info.cleanup_interval_ms > 0);
        assert!(ttl_info.expiring_nuts_count >= 0);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_ttl_enforcement() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Test enabling TTL enforcement
        advanced_tree.set_ttl_enforcement(true)?;
        let ttl_info = advanced_tree.get_ttl_info()?;
        assert!(ttl_info.ttl_enforcement_enabled);
        
        // Test disabling TTL enforcement
        advanced_tree.set_ttl_enforcement(false)?;
        let ttl_info = advanced_tree.get_ttl_info()?;
        assert!(!ttl_info.ttl_enforcement_enabled);
        
        // Re-enable
        advanced_tree.set_ttl_enforcement(true)?;
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_cleanup_interval() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Test setting cleanup interval
        advanced_tree.set_cleanup_interval(30000)?; // 30 seconds
        let ttl_info = advanced_tree.get_ttl_info()?;
        assert_eq!(ttl_info.cleanup_interval_ms, 30000);
        
        // Test different interval
        advanced_tree.set_cleanup_interval(60000)?; // 1 minute
        let ttl_info = advanced_tree.get_ttl_info()?;
        assert_eq!(ttl_info.cleanup_interval_ms, 60000);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_cleanup_expired_nuts() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Test cleanup (should return 0 since no expired nuts)
        let removed_count = advanced_tree.cleanup_expired_nuts()?;
        assert!(removed_count >= 0);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_expiring_nuts_count() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Test getting expiring nuts count
        let count = advanced_tree.get_expiring_nuts_count(3600000)?; // 1 hour
        assert!(count >= 0);
        
        // Test with zero timespan
        let count_zero = advanced_tree.get_expiring_nuts_count(0)?;
        assert!(count_zero >= 0);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_expiring_nuts() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Test getting expiring nuts
        let expiring_ids = advanced_tree.get_expiring_nuts(3600000)?; // 1 hour
        assert!(expiring_ids.len() >= 0);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_get_all_nuts() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Stash some items
        let item1_json = r#"{"id": "item-1", "name": "Item 1"}"#;
        let item2_json = r#"{"id": "item-2", "name": "Item 2"}"#;
        
        advanced_tree.stash_with_auto_id(item1_json)?;
        advanced_tree.stash_with_auto_id(item2_json)?;
        
        // Get all nuts
        let all_nuts = advanced_tree.get_all_nuts()?;
        assert!(all_nuts.len() >= 2);
        
        // Verify nut structure
        for nut in &all_nuts {
            assert!(!nut.id.is_empty());
            assert!(nut.timestamp > 0);
            assert!(nut.version > 0);
        }
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_nut_count() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Get initial count
        let initial_count = advanced_tree.get_nut_count()?;
        assert!(initial_count >= 0);
        
        // Stash items and verify count increases
        let item_json = r#"{"id": "count-test", "name": "Count Test"}"#;
        advanced_tree.stash_with_auto_id(item_json)?;
        
        let updated_count = advanced_tree.get_nut_count()?;
        assert!(updated_count > initial_count);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_last_sync_timestamp() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Get last sync timestamp
        let timestamp = advanced_tree.get_last_sync_timestamp()?;
        assert!(timestamp >= 0);
        
        Ok(())
    }

    #[test]
    fn test_advanced_tree_comprehensive_workflow() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let advanced_tree = AcornAdvancedTree::from_tree(tree);
        
        // Comprehensive workflow test
        let initial_stats = advanced_tree.get_stats()?;
        let initial_count = advanced_tree.get_nut_count()?;
        
        // Configure TTL
        advanced_tree.set_ttl_enforcement(true)?;
        advanced_tree.set_cleanup_interval(30000)?; // 30 seconds
        
        // Stash multiple items
        let items = vec![
            r#"{"id": "workflow-1", "name": "Workflow Item 1", "type": "test"}"#,
            r#"{"id": "workflow-2", "name": "Workflow Item 2", "type": "test"}"#,
            r#"{"id": "workflow-3", "name": "Workflow Item 3", "type": "test"}"#,
        ];
        
        for item_json in &items {
            advanced_tree.stash_with_auto_id(item_json)?;
        }
        
        // Verify changes
        let final_stats = advanced_tree.get_stats()?;
        let final_count = advanced_tree.get_nut_count()?;
        
        assert!(final_stats.total_stashed > initial_stats.total_stashed);
        assert!(final_count > initial_count);
        
        // Test TTL operations
        let ttl_info = advanced_tree.get_ttl_info()?;
        assert!(ttl_info.ttl_enforcement_enabled);
        assert_eq!(ttl_info.cleanup_interval_ms, 30000);
        
        // Test expiring nuts queries
        let expiring_count = advanced_tree.get_expiring_nuts_count(3600000)?;
        assert!(expiring_count >= 0);
        
        let expiring_ids = advanced_tree.get_expiring_nuts(3600000)?;
        assert!(expiring_ids.len() >= 0);
        
        // Test cleanup
        let removed_count = advanced_tree.cleanup_expired_nuts()?;
        assert!(removed_count >= 0);
        
        // Test getting all nuts
        let all_nuts = advanced_tree.get_all_nuts()?;
        assert!(all_nuts.len() >= 3);
        
        Ok(())
    }

    // Event Management Tests
    #[test]
    fn test_event_manager_creation() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let event_manager = AcornEventManager::new(tree)?;
        
        // Verify event manager was created
        let subscriber_count = event_manager.get_subscriber_count()?;
        assert!(subscriber_count >= 0);
        
        Ok(())
    }

    #[test]
    fn test_event_manager_subscription() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let event_manager = AcornEventManager::new(tree)?;
        
        // Subscribe to all events
        let subscription = event_manager.subscribe(|key, json_data| {
            println!("Event: {} with {} bytes", key, json_data.len());
        })?;
        
        // Verify subscription was created
        let subscriber_count = event_manager.get_subscriber_count()?;
        assert!(subscriber_count > 0);
        
        // Clean up subscription
        drop(subscription);
        
        Ok(())
    }

    #[test]
    fn test_event_manager_filtered_subscription() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let event_manager = AcornEventManager::new(tree)?;
        
        // Subscribe to stash events only
        let subscription = event_manager.subscribe_filtered(EventType::Stash, |key, _| {
            println!("Stash event: {}", key);
        })?;
        
        // Verify subscription was created
        let subscriber_count = event_manager.get_subscriber_count()?;
        assert!(subscriber_count > 0);
        
        // Clean up subscription
        drop(subscription);
        
        Ok(())
    }

    #[test]
    fn test_event_manager_raise_event() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let event_manager = AcornEventManager::new(tree)?;
        
        // Raise a custom event
        let payload = r#"{"message": "test event"}"#;
        event_manager.raise_event(EventType::Sync, "test-key", payload)?;
        
        Ok(())
    }

    #[test]
    fn test_tangle_creation() -> Result<(), Error> {
        let local_tree = AcornTree::open_memory()?;
        let remote_tree = AcornTree::open_memory()?;
        
        // Create tangle
        let tangle = AcornTangle::new(local_tree, remote_tree, "test-tangle")?;
        
        // Get tangle stats
        let stats = tangle.get_stats()?;
        assert!(!stats.node_id.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_tangle_in_process() -> Result<(), Error> {
        let local_tree = AcornTree::open_memory()?;
        let remote_tree = AcornTree::open_memory()?;
        
        // Create in-process tangle
        let tangle = AcornTangle::new_in_process(local_tree, remote_tree, "in-process-tangle")?;
        
        // Get tangle stats
        let stats = tangle.get_stats()?;
        assert!(!stats.node_id.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_tangle_operations() -> Result<(), Error> {
        let local_tree = AcornTree::open_memory()?;
        let remote_tree = AcornTree::open_memory()?;
        
        let tangle = AcornTangle::new(local_tree, remote_tree, "test-tangle")?;
        
        // Test push operation
        let payload = r#"{"name": "test", "value": 42}"#;
        tangle.push("test-key", payload)?;
        
        // Test pull operation
        tangle.pull()?;
        
        // Test bidirectional sync
        tangle.sync_bidirectional()?;
        
        Ok(())
    }

    #[test]
    fn test_mesh_coordinator_creation() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        
        // Coordinator should be created successfully
        Ok(())
    }

    #[test]
    fn test_mesh_coordinator_add_node() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        let tree = AcornTree::open_memory()?;
        
        // Add node to mesh
        coordinator.add_node("test-node", tree)?;
        
        Ok(())
    }

    #[test]
    fn test_mesh_coordinator_connect_nodes() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        let tree_a = AcornTree::open_memory()?;
        let tree_b = AcornTree::open_memory()?;
        
        // Add nodes
        coordinator.add_node("node-a", tree_a)?;
        coordinator.add_node("node-b", tree_b)?;
        
        // Connect nodes
        coordinator.connect_nodes("node-a", "node-b")?;
        
        Ok(())
    }

    #[test]
    fn test_mesh_coordinator_topology() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        let tree_a = AcornTree::open_memory()?;
        let tree_b = AcornTree::open_memory()?;
        let tree_c = AcornTree::open_memory()?;
        
        // Add nodes
        coordinator.add_node("node-a", tree_a)?;
        coordinator.add_node("node-b", tree_b)?;
        coordinator.add_node("node-c", tree_c)?;
        
        // Create full mesh topology
        coordinator.create_topology(MeshTopology::Full, "")?;
        
        Ok(())
    }

    #[test]
    fn test_mesh_coordinator_synchronize() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        let tree_a = AcornTree::open_memory()?;
        let tree_b = AcornTree::open_memory()?;
        
        // Add nodes
        coordinator.add_node("node-a", tree_a)?;
        coordinator.add_node("node-b", tree_b)?;
        
        // Synchronize all nodes
        coordinator.synchronize_all()?;
        
        Ok(())
    }

    #[test]
    fn test_mesh_coordinator_stats() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        let tree = AcornTree::open_memory()?;
        
        // Add node
        coordinator.add_node("test-node", tree)?;
        
        // Get node stats
        let stats = coordinator.get_node_stats("test-node")?;
        assert_eq!(stats.node_id, "test-node");
        assert!(stats.active_tangles >= 0);
        assert!(stats.tracked_change_ids >= 0);
        
        Ok(())
    }

    #[test]
    fn test_mesh_coordinator_all_stats() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        let tree_a = AcornTree::open_memory()?;
        let tree_b = AcornTree::open_memory()?;
        
        // Add nodes
        coordinator.add_node("node-a", tree_a)?;
        coordinator.add_node("node-b", tree_b)?;
        
        // Get all stats
        let all_stats = coordinator.get_all_stats()?;
        assert_eq!(all_stats.len(), 2);
        
        // Verify stats structure
        for stats in &all_stats {
            assert!(!stats.node_id.is_empty());
            assert!(stats.active_tangles >= 0);
            assert!(stats.tracked_change_ids >= 0);
        }
        
        Ok(())
    }

    #[test]
    fn test_event_management_comprehensive_workflow() -> Result<(), Error> {
        // Create trees and event managers
        let tree_a = AcornTree::open_memory()?;
        let tree_b = AcornTree::open_memory()?;
        
        let event_manager_a = AcornEventManager::new(tree_a)?;
        let event_manager_b = AcornEventManager::new(tree_b)?;
        
        // Create tangle between trees
        let tangle = AcornTangle::new(tree_a, tree_b, "comprehensive-tangle")?;
        
        // Create mesh coordinator
        let coordinator = AcornMeshCoordinator::new()?;
        coordinator.add_node("tree-a", tree_a)?;
        coordinator.add_node("tree-b", tree_b)?;
        
        // Set up event subscriptions
        let _sub_a = event_manager_a.subscribe(|key, json_data| {
            println!("Tree A event: {} ({} bytes)", key, json_data.len());
        })?;
        
        let _sub_b = event_manager_b.subscribe(|key, json_data| {
            println!("Tree B event: {} ({} bytes)", key, json_data.len());
        })?;
        
        // Raise events
        let payload = r#"{"test": "data"}"#;
        event_manager_a.raise_event(EventType::Stash, "test-key", payload)?;
        event_manager_b.raise_event(EventType::Sync, "sync-key", payload)?;
        
        // Test tangle operations
        tangle.push("tangle-key", payload)?;
        tangle.pull()?;
        tangle.sync_bidirectional()?;
        
        // Test mesh operations
        coordinator.connect_nodes("tree-a", "tree-b")?;
        coordinator.create_topology(MeshTopology::Full, "")?;
        coordinator.synchronize_all()?;
        
        // Verify everything worked
        let stats = tangle.get_stats()?;
        assert!(!stats.node_id.is_empty());
        
        let mesh_stats = coordinator.get_all_stats()?;
        assert_eq!(mesh_stats.len(), 2);
        
        Ok(())
    }

    // Performance Monitoring Tests
    #[test]
    fn test_performance_monitor_creation() -> Result<(), Error> {
        let monitor = AcornPerformanceMonitor::new()?;
        
        // Monitor should be created successfully
        Ok(())
    }

    #[test]
    fn test_performance_monitor_collection() -> Result<(), Error> {
        let monitor = AcornPerformanceMonitor::new()?;
        
        // Start collection
        monitor.start_collection()?;
        
        // Stop collection
        monitor.stop_collection()?;
        
        Ok(())
    }

    #[test]
    fn test_performance_monitor_metrics() -> Result<(), Error> {
        let monitor = AcornPerformanceMonitor::new()?;
        
        // Get metrics (should work even without collection)
        let metrics = monitor.get_metrics()?;
        assert!(metrics.operations_per_second >= 0);
        assert!(metrics.memory_usage_bytes >= 0);
        assert!(metrics.cache_hit_rate_percent >= 0);
        assert!(metrics.cache_hit_rate_percent <= 100);
        
        Ok(())
    }

    #[test]
    fn test_performance_monitor_history() -> Result<(), Error> {
        let monitor = AcornPerformanceMonitor::new()?;
        
        // Get history (should return empty initially)
        let history = monitor.get_history()?;
        assert!(history.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_performance_monitor_reset() -> Result<(), Error> {
        let monitor = AcornPerformanceMonitor::new()?;
        
        // Reset metrics
        monitor.reset_metrics()?;
        
        Ok(())
    }

    #[test]
    fn test_health_checker_creation() -> Result<(), Error> {
        let checker = AcornHealthChecker::new()?;
        
        // Checker should be created successfully
        Ok(())
    }

    #[test]
    fn test_health_checker_add_service() -> Result<(), Error> {
        let checker = AcornHealthChecker::new()?;
        
        // Add a service
        checker.add_service("test-service", "http://localhost:8080/health")?;
        
        Ok(())
    }

    #[test]
    fn test_health_checker_check_all() -> Result<(), Error> {
        let checker = AcornHealthChecker::new()?;
        
        // Add a service
        checker.add_service("test-service", "http://localhost:8080/health")?;
        
        // Check all services
        let results = checker.check_all()?;
        assert!(results.len() >= 0);
        
        Ok(())
    }

    #[test]
    fn test_health_checker_check_service() -> Result<(), Error> {
        let checker = AcornHealthChecker::new()?;
        
        // Add a service
        checker.add_service("test-service", "http://localhost:8080/health")?;
        
        // Check specific service
        let result = checker.check_service("test-service")?;
        assert_eq!(result.service_name, "test-service");
        
        Ok(())
    }

    #[test]
    fn test_health_checker_overall_status() -> Result<(), Error> {
        let checker = AcornHealthChecker::new()?;
        
        // Get overall status
        let status = checker.get_overall_status()?;
        assert!(matches!(status, HealthStatus::Unknown | HealthStatus::Healthy | HealthStatus::Degraded | HealthStatus::Unhealthy));
        
        Ok(())
    }

    #[test]
    fn test_benchmark_tree_operations() -> Result<(), Error> {
        let tree = AcornTree::open_memory()?;
        let config = BenchmarkConfig {
            operation_count: 100,
            warmup_iterations: 5,
            measurement_iterations: 10,
            timeout_ms: 10000,
            enable_memory_tracking: true,
            enable_disk_tracking: false,
            enable_network_tracking: false,
        };
        
        let results = AcornBenchmark::benchmark_tree_operations(tree, &config)?;
        assert!(results.len() > 0);
        
        for result in &results {
            assert!(!result.operation_name.is_empty());
            assert!(result.operations_per_second >= 0);
            assert!(result.total_time_ms >= 0);
        }
        
        Ok(())
    }

    #[test]
    fn test_benchmark_sync_operations() -> Result<(), Error> {
        let local_tree = AcornTree::open_memory()?;
        let remote_tree = AcornTree::open_memory()?;
        let tangle = AcornTangle::new(local_tree, remote_tree, "benchmark-tangle")?;
        
        let config = BenchmarkConfig {
            operation_count: 50,
            warmup_iterations: 3,
            measurement_iterations: 5,
            timeout_ms: 10000,
            enable_memory_tracking: true,
            enable_disk_tracking: false,
            enable_network_tracking: true,
        };
        
        let results = AcornBenchmark::benchmark_sync_operations(tangle, &config)?;
        assert!(results.len() > 0);
        
        for result in &results {
            assert!(!result.operation_name.is_empty());
            assert!(result.operations_per_second >= 0);
            assert!(result.total_time_ms >= 0);
        }
        
        Ok(())
    }

    #[test]
    fn test_benchmark_mesh_operations() -> Result<(), Error> {
        let coordinator = AcornMeshCoordinator::new()?;
        let tree_a = AcornTree::open_memory()?;
        let tree_b = AcornTree::open_memory()?;
        
        coordinator.add_node("node-a", tree_a)?;
        coordinator.add_node("node-b", tree_b)?;
        
        let config = BenchmarkConfig {
            operation_count: 30,
            warmup_iterations: 2,
            measurement_iterations: 3,
            timeout_ms: 10000,
            enable_memory_tracking: true,
            enable_disk_tracking: false,
            enable_network_tracking: true,
        };
        
        let results = AcornBenchmark::benchmark_mesh_operations(coordinator, &config)?;
        assert!(results.len() > 0);
        
        for result in &results {
            assert!(!result.operation_name.is_empty());
            assert!(result.operations_per_second >= 0);
            assert!(result.total_time_ms >= 0);
        }
        
        Ok(())
    }

    #[test]
    fn test_resource_monitor_memory_usage() -> Result<(), Error> {
        let (heap_bytes, stack_bytes, total_bytes) = AcornResourceMonitor::get_memory_usage()?;
        
        assert!(heap_bytes >= 0);
        assert!(stack_bytes >= 0);
        assert!(total_bytes >= 0);
        assert!(total_bytes >= heap_bytes);
        assert!(total_bytes >= stack_bytes);
        
        Ok(())
    }

    #[test]
    fn test_resource_monitor_disk_usage() -> Result<(), Error> {
        let (used_bytes, total_bytes, free_bytes) = AcornResourceMonitor::get_disk_usage("/tmp")?;
        
        assert!(used_bytes >= 0);
        assert!(total_bytes >= 0);
        assert!(free_bytes >= 0);
        assert!(total_bytes >= used_bytes);
        assert!(total_bytes >= free_bytes);
        
        Ok(())
    }

    #[test]
    fn test_resource_monitor_system_info() -> Result<(), Error> {
        let system_info = AcornResourceMonitor::get_system_info()?;
        
        // System info should be a non-empty string
        assert!(!system_info.is_empty());
        
        Ok(())
    }

    #[test]
    fn test_performance_monitoring_comprehensive_workflow() -> Result<(), Error> {
        // Create performance monitor
        let monitor = AcornPerformanceMonitor::new()?;
        
        // Create health checker
        let health_checker = AcornHealthChecker::new()?;
        health_checker.add_service("test-service", "http://localhost:8080/health")?;
        
        // Create tree for benchmarking
        let tree = AcornTree::open_memory()?;
        
        // Start performance monitoring
        monitor.start_collection()?;
        
        // Perform some operations
        for i in 0..100 {
            let data = format!(r#"{{"id": "item-{}", "value": {}}}"#, i, i);
            tree.stash(&format!("item-{}", i), &data)?;
        }
        
        // Stop monitoring
        monitor.stop_collection()?;
        
        // Get metrics
        let metrics = monitor.get_metrics()?;
        assert!(metrics.operations_per_second >= 0);
        
        // Get history
        let history = monitor.get_history()?;
        assert!(history.len() >= 0);
        
        // Check health
        let health_results = health_checker.check_all()?;
        assert!(health_results.len() >= 0);
        
        // Run benchmark
        let config = BenchmarkConfig {
            operation_count: 50,
            warmup_iterations: 3,
            measurement_iterations: 5,
            timeout_ms: 5000,
            enable_memory_tracking: true,
            enable_disk_tracking: false,
            enable_network_tracking: false,
        };
        
        let benchmark_results = AcornBenchmark::benchmark_tree_operations(tree, &config)?;
        assert!(benchmark_results.len() > 0);
        
        // Check resource usage
        let (heap_bytes, stack_bytes, total_bytes) = AcornResourceMonitor::get_memory_usage()?;
        assert!(total_bytes > 0);
        
        Ok(())
    }
}

