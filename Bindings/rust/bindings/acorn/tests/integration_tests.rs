use acorn::{AcornTree, AcornEncryption, AcornCompression, AcornCache, AcornConflictJudge, AcornStorage, CompressionLevel, Error};
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
}

