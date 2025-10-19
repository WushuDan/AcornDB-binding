use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct SensorReading {
    sensor_id: String,
    temperature: f64,
    humidity: f64,
    timestamp: String,
}

fn main() -> Result<(), Error> {
    println!("AcornDB Subscription Example");
    println!();

    // Open a tree with memory storage
    let mut tree = AcornTree::open("memory://")?;
    println!("✓ Opened database");
    println!();

    // Track notifications received
    let notifications = Arc::new(Mutex::new(Vec::new()));
    let notifications_clone = notifications.clone();

    // Subscribe to changes
    println!("=== Setting up subscription ===");
    let _subscription = tree.subscribe(move |key: &str, value: &serde_json::Value| {
        println!("📢 Change detected:");
        println!("   Key: {}", key);
        println!("   Value: {}", value);

        // Store the notification
        let mut n = notifications_clone.lock().unwrap();
        n.push((key.to_string(), value.clone()));
    })?;
    println!("✓ Subscription active");
    println!();

    // Give the subscription a moment to initialize
    thread::sleep(Duration::from_millis(100));

    // Example 1: Single update
    println!("=== Example 1: Single Update ===");
    let reading1 = SensorReading {
        sensor_id: "sensor-001".to_string(),
        temperature: 22.5,
        humidity: 45.0,
        timestamp: "2025-10-18T10:00:00Z".to_string(),
    };
    tree.stash("sensor-001", &reading1)?;
    println!("✓ Stashed reading for sensor-001");
    thread::sleep(Duration::from_millis(200));
    println!();

    // Example 2: Multiple updates
    println!("=== Example 2: Multiple Updates ===");
    let sensors = vec![
        ("sensor-002", SensorReading {
            sensor_id: "sensor-002".to_string(),
            temperature: 23.1,
            humidity: 50.2,
            timestamp: "2025-10-18T10:01:00Z".to_string(),
        }),
        ("sensor-003", SensorReading {
            sensor_id: "sensor-003".to_string(),
            temperature: 21.8,
            humidity: 48.5,
            timestamp: "2025-10-18T10:02:00Z".to_string(),
        }),
        ("sensor-004", SensorReading {
            sensor_id: "sensor-004".to_string(),
            temperature: 24.2,
            humidity: 52.1,
            timestamp: "2025-10-18T10:03:00Z".to_string(),
        }),
    ];

    for (id, reading) in sensors {
        tree.stash(id, &reading)?;
        println!("✓ Stashed reading for {}", id);
        thread::sleep(Duration::from_millis(150));
    }
    println!();

    // Example 3: Update existing value
    println!("=== Example 3: Update Existing Value ===");
    let updated_reading = SensorReading {
        sensor_id: "sensor-001".to_string(),
        temperature: 23.5,  // Temperature increased
        humidity: 46.5,     // Humidity increased
        timestamp: "2025-10-18T10:04:00Z".to_string(),
    };
    tree.stash("sensor-001", &updated_reading)?;
    println!("✓ Updated reading for sensor-001");
    thread::sleep(Duration::from_millis(200));
    println!();

    // Give some time for all notifications to be processed
    thread::sleep(Duration::from_millis(500));

    // Display summary
    println!("=== Summary ===");
    let n = notifications.lock().unwrap();
    println!("Total notifications received: {}", n.len());

    for (idx, (key, _value)) in n.iter().enumerate() {
        println!("  {}. Key: {}", idx + 1, key);
    }
    println!();

    // The subscription will be automatically cleaned up when dropped
    println!("✓ Subscription will be automatically unsubscribed on drop");
    println!();

    println!("🎉 Subscription example completed successfully!");
    Ok(())
}
