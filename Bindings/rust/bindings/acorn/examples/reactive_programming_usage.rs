use acorn::{AcornTree, Error, ChangeType};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    role: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Order {
    id: String,
    user_id: String,
    amount: f64,
    status: String,
}

fn main() -> Result<(), Error> {
    println!("🌰 AcornDB Reactive Programming Example");
    println!("=====================================");

    // Open a tree with memory storage
    let mut tree = AcornTree::open("memory://")?;
    println!("✅ Opened database");
    println!();

    // Track different types of notifications
    let stash_notifications = Arc::new(Mutex::new(Vec::new()));
    let toss_notifications = Arc::new(Mutex::new(Vec::new()));
    let filtered_notifications = Arc::new(Mutex::new(Vec::new()));

    // Example 1: Subscribe to all changes
    println!("=== Example 1: Subscribe to All Changes ===");
    let all_notifications = Arc::new(Mutex::new(Vec::new()));
    let all_notifications_clone = all_notifications.clone();
    
    let _all_sub = tree.subscribe(move |key: &str, value: &serde_json::Value| {
        println!("📢 All Changes - Key: {}, Value: {}", key, value);
        let mut n = all_notifications_clone.lock().unwrap();
        n.push((key.to_string(), value.clone()));
    })?;
    println!("✅ Subscribed to all changes");
    println!();

    // Example 2: Subscribe to only stash operations
    println!("=== Example 2: Subscribe to Stash Operations Only ===");
    let stash_notifications_clone = stash_notifications.clone();
    
    let _stash_sub = tree.subscribe_stash(move |key: &str, value: &serde_json::Value| {
        println!("📦 Stash - Key: {}, Value: {}", key, value);
        let mut n = stash_notifications_clone.lock().unwrap();
        n.push((key.to_string(), value.clone()));
    })?;
    println!("✅ Subscribed to stash operations");
    println!();

    // Example 3: Subscribe to only toss operations
    println!("=== Example 3: Subscribe to Toss Operations Only ===");
    let toss_notifications_clone = toss_notifications.clone();
    
    let _toss_sub = tree.subscribe_toss(move |key: &str| {
        println!("🗑️  Toss - Key: {}", key);
        let mut n = toss_notifications_clone.lock().unwrap();
        n.push(key.to_string());
    })?;
    println!("✅ Subscribed to toss operations");
    println!();

    // Example 4: Subscribe with filtering predicate
    println!("=== Example 4: Subscribe with Filtering ===");
    let filtered_notifications_clone = filtered_notifications.clone();
    
    let _filtered_sub = tree.subscribe_where(
        |key: &str, value: &serde_json::Value| {
            // Only notify for user-related keys
            key.starts_with("user-") || 
            (value.is_object() && 
             value.get("role").and_then(|r| r.as_str()) == Some("admin"))
        },
        move |key: &str, value: &serde_json::Value| {
            println!("🔍 Filtered - Key: {}, Value: {}", key, value);
            let mut n = filtered_notifications_clone.lock().unwrap();
            n.push((key.to_string(), value.clone()));
        }
    )?;
    println!("✅ Subscribed with filtering (user-* keys and admin roles)");
    println!();

    // Give subscriptions time to initialize
    thread::sleep(Duration::from_millis(100));

    // Example 5: Perform various operations to trigger notifications
    println!("=== Example 5: Performing Operations ===");

    // Create some users
    let users = vec![
        ("user-1", User {
            id: "user-1".to_string(),
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            role: "user".to_string(),
        }),
        ("user-2", User {
            id: "user-2".to_string(),
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            role: "admin".to_string(),
        }),
        ("user-3", User {
            id: "user-3".to_string(),
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            role: "user".to_string(),
        }),
    ];

    for (id, user) in users {
        tree.stash(id, &user)?;
        println!("✅ Stashed user: {}", id);
        thread::sleep(Duration::from_millis(200));
    }
    println!();

    // Create some orders (these won't trigger filtered notifications)
    let orders = vec![
        ("order-1", Order {
            id: "order-1".to_string(),
            user_id: "user-1".to_string(),
            amount: 99.99,
            status: "pending".to_string(),
        }),
        ("order-2", Order {
            id: "order-2".to_string(),
            user_id: "user-2".to_string(),
            amount: 149.99,
            status: "completed".to_string(),
        }),
    ];

    for (id, order) in orders {
        tree.stash(id, &order)?;
        println!("✅ Stashed order: {}", id);
        thread::sleep(Duration::from_millis(200));
    }
    println!();

    // Update a user (this will trigger stash notifications)
    let updated_user = User {
        id: "user-1".to_string(),
        name: "Alice Johnson-Smith".to_string(), // Changed name
        email: "alice.smith@example.com".to_string(), // Changed email
        role: "user".to_string(),
    };
    tree.stash("user-1", &updated_user)?;
    println!("✅ Updated user-1");
    thread::sleep(Duration::from_millis(200));
    println!();

    // Delete some items (this will trigger toss notifications)
    tree.toss("order-1")?;
    println!("✅ Tossed order-1");
    thread::sleep(Duration::from_millis(200));

    tree.toss("user-3")?;
    println!("✅ Tossed user-3");
    thread::sleep(Duration::from_millis(200));
    println!();

    // Give time for all notifications to be processed
    thread::sleep(Duration::from_millis(500));

    // Display summary
    println!("=== Summary ===");
    
    let all_n = all_notifications.lock().unwrap();
    println!("📊 All Changes Notifications: {}", all_n.len());
    for (idx, (key, _value)) in all_n.iter().enumerate() {
        println!("   {}. Key: {}", idx + 1, key);
    }
    println!();

    let stash_n = stash_notifications.lock().unwrap();
    println!("📦 Stash Notifications: {}", stash_n.len());
    for (idx, (key, _value)) in stash_n.iter().enumerate() {
        println!("   {}. Key: {}", idx + 1, key);
    }
    println!();

    let toss_n = toss_notifications.lock().unwrap();
    println!("🗑️  Toss Notifications: {}", toss_n.len());
    for (idx, key) in toss_n.iter().enumerate() {
        println!("   {}. Key: {}", idx + 1, key);
    }
    println!();

    let filtered_n = filtered_notifications.lock().unwrap();
    println!("🔍 Filtered Notifications: {}", filtered_n.len());
    for (idx, (key, _value)) in filtered_n.iter().enumerate() {
        println!("   {}. Key: {}", idx + 1, key);
    }
    println!();

    println!("🎉 Reactive Programming example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("✅ Basic subscription to all changes");
    println!("✅ Filtered subscription to stash operations only");
    println!("✅ Filtered subscription to toss operations only");
    println!("✅ Predicate-based filtering with custom logic");
    println!("✅ Automatic subscription cleanup on drop");
    println!("✅ Thread-safe notification handling");

    Ok(())
}
