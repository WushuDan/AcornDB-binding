use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Event {
    id: String,
    name: String,
    category: String,
    severity: String,
    timestamp: SystemTime,
    node_id: String,
    description: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    role: String,
    last_login: SystemTime,
    node_id: String,
}

fn main() -> Result<(), Error> {
    println!("🌰 AcornDB Advanced Queries Example");
    println!("==================================");

    // Open a tree with memory storage
    let mut tree = AcornTree::open("memory://")?;
    println!("✅ Opened database");
    println!();

    // Create some test data with timestamps
    let now = SystemTime::now();
    let one_hour_ago = now - Duration::from_secs(3600);
    let two_hours_ago = now - Duration::from_secs(7200);
    let three_hours_ago = now - Duration::from_secs(10800);

    // Example 1: Timestamp-based filtering
    println!("=== Example 1: Timestamp-based Filtering ===");

    let events = vec![
        ("event-1", Event {
            id: "event-1".to_string(),
            name: "System Startup".to_string(),
            category: "System".to_string(),
            severity: "Info".to_string(),
            timestamp: three_hours_ago,
            node_id: "node-1".to_string(),
            description: "System started successfully".to_string(),
        }),
        ("event-2", Event {
            id: "event-2".to_string(),
            name: "User Login".to_string(),
            category: "Security".to_string(),
            severity: "Info".to_string(),
            timestamp: two_hours_ago,
            node_id: "node-2".to_string(),
            description: "User alice logged in".to_string(),
        }),
        ("event-3", Event {
            id: "event-3".to_string(),
            name: "Database Error".to_string(),
            category: "Error".to_string(),
            severity: "Critical".to_string(),
            timestamp: one_hour_ago,
            node_id: "node-1".to_string(),
            description: "Database connection failed".to_string(),
        }),
        ("event-4", Event {
            id: "event-4".to_string(),
            name: "Backup Complete".to_string(),
            category: "System".to_string(),
            severity: "Info".to_string(),
            timestamp: now,
            node_id: "node-3".to_string(),
            description: "Daily backup completed".to_string(),
        }),
    ];

    tree.batch_stash(&events)?;
    println!("✅ Stored {} events", events.len());

    // Query 1: Events in the last hour
    println!("\n📅 Events in the last hour:");
    let recent_events: Vec<Event> = tree.query()
        .after(one_hour_ago)
        .collect()?;
    for event in &recent_events {
        println!("  - {} ({})", event.name, event.category);
    }

    // Query 2: Events before 2 hours ago
    println!("\n📅 Events before 2 hours ago:");
    let old_events: Vec<Event> = tree.query()
        .before(two_hours_ago)
        .collect()?;
    for event in &old_events {
        println!("  - {} ({})", event.name, event.category);
    }

    // Query 3: Events between 3 hours ago and 1 hour ago
    println!("\n📅 Events between 3 hours ago and 1 hour ago:");
    let range_events: Vec<Event> = tree.query()
        .between(three_hours_ago, one_hour_ago)
        .collect()?;
    for event in &range_events {
        println!("  - {} ({})", event.name, event.category);
    }
    println!();

    // Example 2: Node-based filtering
    println!("=== Example 2: Node-based Filtering ===");

    let users = vec![
        ("user-1", User {
            id: "user-1".to_string(),
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            role: "admin".to_string(),
            last_login: now,
            node_id: "node-1".to_string(),
        }),
        ("user-2", User {
            id: "user-2".to_string(),
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            role: "user".to_string(),
            last_login: one_hour_ago,
            node_id: "node-2".to_string(),
        }),
        ("user-3", User {
            id: "user-3".to_string(),
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            role: "user".to_string(),
            last_login: two_hours_ago,
            node_id: "node-1".to_string(),
        }),
        ("user-4", User {
            id: "user-4".to_string(),
            name: "Diana Prince".to_string(),
            email: "diana@example.com".to_string(),
            role: "moderator".to_string(),
            last_login: three_hours_ago,
            node_id: "node-3".to_string(),
        }),
    ];

    tree.batch_stash(&users)?;
    println!("✅ Stored {} users", users.len());

    // Query 4: Users from node-1
    println!("\n🖥️  Users from node-1:");
    let node1_users: Vec<User> = tree.query()
        .from_node("node-1")
        .collect()?;
    for user in &node1_users {
        println!("  - {} ({})", user.name, user.role);
    }

    // Query 5: Users from node-2
    println!("\n🖥️  Users from node-2:");
    let node2_users: Vec<User> = tree.query()
        .from_node("node-2")
        .collect()?;
    for user in &node2_users {
        println!("  - {} ({})", user.name, user.role);
    }
    println!();

    // Example 3: Timestamp-based ordering
    println!("=== Example 3: Timestamp-based Ordering ===");

    // Query 6: Events ordered by newest first
    println!("\n🕒 Events ordered by newest first:");
    let newest_events: Vec<Event> = tree.query()
        .newest()
        .collect()?;
    for event in &newest_events {
        println!("  - {} ({})", event.name, event.category);
    }

    // Query 7: Events ordered by oldest first
    println!("\n🕒 Events ordered by oldest first:");
    let oldest_events: Vec<Event> = tree.query()
        .oldest()
        .collect()?;
    for event in &oldest_events {
        println!("  - {} ({})", event.name, event.category);
    }
    println!();

    // Example 4: Single result queries
    println!("=== Example 4: Single Result Queries ===");

    // Query 8: Find single admin user
    println!("\n👤 Finding single admin user:");
    if let Some(admin) = tree.query()
        .where_condition(|user| user["role"].as_str() == Some("admin"))
        .single::<User>()? {
        println!("  - Found admin: {} ({})", admin.name, admin.email);
    } else {
        println!("  - No admin user found");
    }

    // Query 9: Find single critical event
    println!("\n⚠️  Finding single critical event:");
    if let Some(critical_event) = tree.query()
        .where_condition(|event| event["severity"].as_str() == Some("Critical"))
        .single::<Event>()? {
        println!("  - Found critical event: {} ({})", critical_event.name, critical_event.description);
    } else {
        println!("  - No critical events found");
    }
    println!();

    // Example 5: Complex combined queries
    println!("=== Example 5: Complex Combined Queries ===");

    // Query 10: Recent events from specific node, ordered by newest
    println!("\n🔍 Recent events from node-1, ordered by newest:");
    let recent_node1_events: Vec<Event> = tree.query()
        .where_condition(|event| event["node_id"].as_str() == Some("node-1"))
        .after(two_hours_ago)
        .newest()
        .collect()?;
    for event in &recent_node1_events {
        println!("  - {} ({})", event.name, event.category);
    }

    // Query 11: Users with specific role, ordered by newest login
    println!("\n👥 Users with 'user' role, ordered by newest login:");
    let regular_users: Vec<User> = tree.query()
        .where_condition(|user| user["role"].as_str() == Some("user"))
        .newest()
        .collect()?;
    for user in &regular_users {
        println!("  - {} (last login: {:?})", user.name, user.last_login);
    }

    // Query 12: Critical events from specific time range
    println!("\n🚨 Critical events from last 2 hours:");
    let critical_recent: Vec<Event> = tree.query()
        .where_condition(|event| event["severity"].as_str() == Some("Critical"))
        .after(two_hours_ago)
        .collect()?;
    for event in &critical_recent {
        println!("  - {} ({})", event.name, event.description);
    }
    println!();

    // Example 6: Performance comparison
    println!("=== Example 6: Performance Comparison ===");
    use std::time::Instant;

    // Traditional iteration approach
    let start = Instant::now();
    let mut traditional_count = 0;
    let mut iter = tree.iter("")?;
    while let Some((_, _)) = iter.next::<Event>()? {
        traditional_count += 1;
    }
    let traditional_time = start.elapsed();

    // Advanced query approach
    let start = Instant::now();
    let query_count = tree.query()
        .where_condition(|event| event["category"].as_str() == Some("System"))
        .count()?;
    let query_time = start.elapsed();

    println!("✓ Traditional iteration: {} items in {:?}", traditional_count, traditional_time);
    println!("✓ Advanced query count: {} items in {:?}", query_count, query_time);
    println!();

    println!("🎉 Advanced Queries example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("✅ Timestamp filtering: between(), after(), before()");
    println!("✅ Node filtering: from_node()");
    println!("✅ Timestamp ordering: newest(), oldest()");
    println!("✅ Single result queries: single()");
    println!("✅ Complex combined queries: multiple filters + ordering");
    println!("✅ Performance: efficient query execution");
    println!("✅ Type safety: automatic deserialization to Rust structs");

    Ok(())
}
