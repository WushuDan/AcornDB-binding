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
}

