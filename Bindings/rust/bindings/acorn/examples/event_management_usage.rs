use acorn::{AcornTree, AcornEventManager, AcornTangle, AcornMeshCoordinator, EventType, MeshTopology, Error};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    role: String,
    last_login: SystemTime,
    is_active: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Product {
    id: String,
    name: String,
    price: f64,
    category: String,
    stock_count: i32,
    created_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Order {
    id: String,
    user_id: String,
    product_id: String,
    quantity: i32,
    total_price: f64,
    status: String,
    created_at: SystemTime,
}

fn main() -> Result<(), Error> {
    println!("🔔 AcornDB Event Management Example");
    println!("==================================");

    // Example 1: Basic Event Management
    println!("=== Example 1: Basic Event Management ===");

    let tree = AcornTree::open_memory()?;
    let event_manager = AcornEventManager::new(tree)?;

    // Create event counters
    let stash_count = Arc::new(Mutex::new(0));
    let toss_count = Arc::new(Mutex::new(0));
    let sync_count = Arc::new(Mutex::new(0));

    // Subscribe to all events
    let stash_count_clone = stash_count.clone();
    let _subscription_all = event_manager.subscribe(move |key, json_data| {
        println!("📡 Event received: {} (data length: {})", key, json_data.len());
        if let Ok(json_str) = std::str::from_utf8(json_data) {
            println!("   Data: {}", json_str);
        }
    })?;

    // Subscribe to stash events only
    let stash_count_clone2 = stash_count.clone();
    let _subscription_stash = event_manager.subscribe_filtered(EventType::Stash, move |key, _| {
        println!("📥 Stash event: {}", key);
        *stash_count_clone2.lock().unwrap() += 1;
    })?;

    // Subscribe to toss events only
    let toss_count_clone = toss_count.clone();
    let _subscription_toss = event_manager.subscribe_filtered(EventType::Toss, move |key, _| {
        println!("📤 Toss event: {}", key);
        *toss_count_clone.lock().unwrap() += 1;
    })?;

    // Subscribe to sync events only
    let sync_count_clone = sync_count.clone();
    let _subscription_sync = event_manager.subscribe_filtered(EventType::Sync, move |key, _| {
        println!("🔄 Sync event: {}", key);
        *sync_count_clone.lock().unwrap() += 1;
    })?;

    // Get initial subscriber count
    let initial_count = event_manager.get_subscriber_count()?;
    println!("📊 Initial subscriber count: {}", initial_count);

    // Raise custom events
    let user_json = serde_json::to_string(&User {
        id: "user-1".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        role: "admin".to_string(),
        last_login: SystemTime::now(),
        is_active: true,
    })?;

    event_manager.raise_event(EventType::Stash, "user-1", &user_json)?;
    event_manager.raise_event(EventType::Sync, "sync-user-1", &user_json)?;

    // Check event counts
    println!("📈 Event counts:");
    println!("   Stash events: {}", *stash_count.lock().unwrap());
    println!("   Toss events: {}", *toss_count.lock().unwrap());
    println!("   Sync events: {}", *sync_count.lock().unwrap());
    println!();

    // Example 2: Tangle Synchronization
    println!("=== Example 2: Tangle Synchronization ===");

    let local_tree = AcornTree::open_memory()?;
    let remote_tree = AcornTree::open_memory()?;

    // Create in-process tangle
    let tangle = AcornTangle::new_in_process(local_tree, remote_tree, "user-sync")?;

    // Create users
    let users = vec![
        User {
            id: "user-1".to_string(),
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            role: "admin".to_string(),
            last_login: SystemTime::now(),
            is_active: true,
        },
        User {
            id: "user-2".to_string(),
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            role: "user".to_string(),
            last_login: SystemTime::now(),
            is_active: true,
        },
        User {
            id: "user-3".to_string(),
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            role: "moderator".to_string(),
            last_login: SystemTime::now(),
            is_active: false,
        },
    ];

    // Push users through tangle
    for user in &users {
        let user_json = serde_json::to_string(user)?;
        tangle.push(&user.id, &user_json)?;
        println!("🔄 Pushed user: {} to remote tree", user.name);
    }

    // Pull data from remote
    tangle.pull()?;
    println!("📥 Pulled data from remote tree");

    // Synchronize bidirectionally
    tangle.sync_bidirectional()?;
    println!("🔄 Bidirectional sync completed");

    // Get tangle statistics
    let tangle_stats = tangle.get_stats()?;
    println!("📊 Tangle Statistics:");
    println!("   Node ID: {}", tangle_stats.node_id);
    println!("   Tracked change IDs: {}", tangle_stats.tracked_change_ids);
    println!("   Active tangles: {}", tangle_stats.active_tangles);
    println!("   Max hop count: {}", tangle_stats.max_hop_count);
    println!("   Total sync operations: {}", tangle_stats.total_sync_operations);
    println!("   Last sync timestamp: {}", tangle_stats.last_sync_timestamp);
    println!();

    // Example 3: Mesh Coordination
    println!("=== Example 3: Mesh Coordination ===");

    let coordinator = AcornMeshCoordinator::new()?;

    // Create multiple trees for mesh
    let tree_a = AcornTree::open_memory()?;
    let tree_b = AcornTree::open_memory()?;
    let tree_c = AcornTree::open_memory()?;
    let tree_d = AcornTree::open_memory()?;

    // Add nodes to mesh
    coordinator.add_node("node-a", tree_a)?;
    coordinator.add_node("node-b", tree_b)?;
    coordinator.add_node("node-c", tree_c)?;
    coordinator.add_node("node-d", tree_d)?;
    println!("🕸️  Added 4 nodes to mesh");

    // Create full mesh topology
    coordinator.create_topology(MeshTopology::Full, "")?;
    println!("🕸️  Created full mesh topology");

    // Synchronize all nodes
    coordinator.synchronize_all()?;
    println!("🔄 Synchronized all nodes in mesh");

    // Get statistics for specific node
    let node_a_stats = coordinator.get_node_stats("node-a")?;
    println!("📊 Node A Statistics:");
    println!("   Node ID: {}", node_a_stats.node_id);
    println!("   Tracked change IDs: {}", node_a_stats.tracked_change_ids);
    println!("   Active tangles: {}", node_a_stats.active_tangles);
    println!("   Max hop count: {}", node_a_stats.max_hop_count);
    println!("   Total sync operations: {}", node_a_stats.total_sync_operations);

    // Get statistics for all nodes
    let all_stats = coordinator.get_all_stats()?;
    println!("📊 All Mesh Statistics:");
    for stats in &all_stats {
        println!("   Node {}: {} active tangles, {} sync operations", 
            stats.node_id, stats.active_tangles, stats.total_sync_operations);
    }
    println!();

    // Example 4: Different Topology Patterns
    println!("=== Example 4: Different Topology Patterns ===");

    // Create new coordinator for topology examples
    let topology_coordinator = AcornMeshCoordinator::new()?;

    // Add nodes
    let tree_ring_a = AcornTree::open_memory()?;
    let tree_ring_b = AcornTree::open_memory()?;
    let tree_ring_c = AcornTree::open_memory()?;

    topology_coordinator.add_node("ring-a", tree_ring_a)?;
    topology_coordinator.add_node("ring-b", tree_ring_b)?;
    topology_coordinator.add_node("ring-c", tree_ring_c)?;

    // Create ring topology
    topology_coordinator.create_topology(MeshTopology::Ring, "")?;
    println!("🔗 Created ring topology");

    // Create star topology
    let star_coordinator = AcornMeshCoordinator::new()?;
    let tree_star_hub = AcornTree::open_memory()?;
    let tree_star_1 = AcornTree::open_memory()?;
    let tree_star_2 = AcornTree::open_memory()?;

    star_coordinator.add_node("hub", tree_star_hub)?;
    star_coordinator.add_node("spoke-1", tree_star_1)?;
    star_coordinator.add_node("spoke-2", tree_star_2)?;

    star_coordinator.create_topology(MeshTopology::Star, "hub")?;
    println!("⭐ Created star topology with hub node");

    // Custom topology
    let custom_coordinator = AcornMeshCoordinator::new()?;
    let tree_custom_1 = AcornTree::open_memory()?;
    let tree_custom_2 = AcornTree::open_memory()?;
    let tree_custom_3 = AcornTree::open_memory()?;

    custom_coordinator.add_node("custom-1", tree_custom_1)?;
    custom_coordinator.add_node("custom-2", tree_custom_2)?;
    custom_coordinator.add_node("custom-3", tree_custom_3)?;

    // Manual connections
    custom_coordinator.connect_nodes("custom-1", "custom-2")?;
    custom_coordinator.connect_nodes("custom-2", "custom-3")?;
    println!("🔗 Created custom topology: 1 ↔ 2 ↔ 3");
    println!();

    // Example 5: Event-Driven Application
    println!("=== Example 5: Event-Driven Application ===");

    let app_tree = AcornTree::open_memory()?;
    let app_event_manager = AcornEventManager::new(app_tree)?;

    // Create application event handlers
    let audit_log = Arc::new(Mutex::new(Vec::new()));
    let cache_invalidation = Arc::new(Mutex::new(Vec::new()));
    let notification_queue = Arc::new(Mutex::new(Vec::new()));

    // Audit logging handler
    let audit_log_clone = audit_log.clone();
    let _audit_subscription = app_event_manager.subscribe_filtered(EventType::Stash, move |key, json_data| {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let log_entry = format!("[{}] STASH: {} ({} bytes)", timestamp, key, json_data.len());
        audit_log_clone.lock().unwrap().push(log_entry);
    })?;

    // Cache invalidation handler
    let cache_clone = cache_invalidation.clone();
    let _cache_subscription = app_event_manager.subscribe_filtered(EventType::Toss, move |key, _| {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let cache_entry = format!("[{}] CACHE_INVALIDATE: {}", timestamp, key);
        cache_clone.lock().unwrap().push(cache_entry);
    })?;

    // Notification handler
    let notification_clone = notification_queue.clone();
    let _notification_subscription = app_event_manager.subscribe_filtered(EventType::Sync, move |key, json_data| {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let notification = format!("[{}] NOTIFICATION: {} synced ({} bytes)", timestamp, key, json_data.len());
        notification_clone.lock().unwrap().push(notification);
    })?;

    // Simulate application events
    let products = vec![
        Product {
            id: "prod-1".to_string(),
            name: "AcornDB Pro".to_string(),
            price: 99.99,
            category: "Software".to_string(),
            stock_count: 100,
            created_at: SystemTime::now(),
        },
        Product {
            id: "prod-2".to_string(),
            name: "Rust Binding Kit".to_string(),
            price: 49.99,
            category: "Development".to_string(),
            stock_count: 50,
            created_at: SystemTime::now(),
        },
    ];

    for product in &products {
        let product_json = serde_json::to_string(product)?;
        app_event_manager.raise_event(EventType::Stash, &product.id, &product_json)?;
    }

    // Simulate some toss events
    app_event_manager.raise_event(EventType::Toss, "prod-1", r#"{"reason": "discontinued"}"#)?;

    // Simulate sync events
    app_event_manager.raise_event(EventType::Sync, "prod-2", r#"{"sync_source": "inventory_system"}"#)?;

    // Display event logs
    println!("📋 Audit Log:");
    for entry in audit_log.lock().unwrap().iter() {
        println!("   {}", entry);
    }

    println!("🗑️  Cache Invalidation Log:");
    for entry in cache_invalidation.lock().unwrap().iter() {
        println!("   {}", entry);
    }

    println!("📢 Notification Queue:");
    for entry in notification_queue.lock().unwrap().iter() {
        println!("   {}", entry);
    }
    println!();

    // Example 6: Performance Monitoring
    println!("=== Example 6: Performance Monitoring ===");

    use std::time::Instant;

    let perf_tree = AcornTree::open_memory()?;
    let perf_event_manager = AcornEventManager::new(perf_tree)?;

    // Performance monitoring
    let event_times = Arc::new(Mutex::new(Vec::new()));
    let event_times_clone = event_times.clone();

    let _perf_subscription = perf_event_manager.subscribe(move |key, json_data| {
        let start = Instant::now();
        // Simulate some processing
        let _processed = json_data.len() * 2;
        let duration = start.elapsed();
        
        event_times_clone.lock().unwrap().push((key.to_string(), duration));
    })?;

    // Generate events and measure performance
    let start = Instant::now();
    for i in 0..100 {
        let order = Order {
            id: format!("order-{}", i),
            user_id: format!("user-{}", i % 10),
            product_id: format!("prod-{}", i % 5),
            quantity: (i % 10) + 1,
            total_price: (i as f64) * 10.0,
            status: "pending".to_string(),
            created_at: SystemTime::now(),
        };
        
        let order_json = serde_json::to_string(&order)?;
        perf_event_manager.raise_event(EventType::Stash, &order.id, &order_json)?;
    }
    let total_time = start.elapsed();

    // Analyze performance
    let times = event_times.lock().unwrap();
    let total_events = times.len();
    let total_processing_time: std::time::Duration = times.iter().map(|(_, duration)| *duration).sum();
    let avg_processing_time = if total_events > 0 {
        total_processing_time / total_events as u32
    } else {
        std::time::Duration::from_nanos(0)
    };

    println!("⚡ Performance Results:");
    println!("   Total events generated: {}", total_events);
    println!("   Total generation time: {:?}", total_time);
    println!("   Total processing time: {:?}", total_processing_time);
    println!("   Average processing time per event: {:?}", avg_processing_time);
    println!("   Events per second: {:.2}", total_events as f64 / total_time.as_secs_f64());
    println!();

    // Example 7: Event Filtering and Routing
    println!("=== Example 7: Event Filtering and Routing ===");

    let routing_tree = AcornTree::open_memory()?;
    let routing_event_manager = AcornEventManager::new(routing_tree)?;

    // Create specialized handlers for different event types
    let user_events = Arc::new(Mutex::new(Vec::new()));
    let product_events = Arc::new(Mutex::new(Vec::new()));
    let order_events = Arc::new(Mutex::new(Vec::new()));

    // User event handler
    let user_events_clone = user_events.clone();
    let _user_subscription = routing_event_manager.subscribe(move |key, json_data| {
        if key.starts_with("user-") {
            let event = format!("USER_EVENT: {} -> {}", key, json_data.len());
            user_events_clone.lock().unwrap().push(event);
        }
    })?;

    // Product event handler
    let product_events_clone = product_events.clone();
    let _product_subscription = routing_event_manager.subscribe(move |key, json_data| {
        if key.starts_with("prod-") {
            let event = format!("PRODUCT_EVENT: {} -> {}", key, json_data.len());
            product_events_clone.lock().unwrap().push(event);
        }
    })?;

    // Order event handler
    let order_events_clone = order_events.clone();
    let _order_subscription = routing_event_manager.subscribe(move |key, json_data| {
        if key.starts_with("order-") {
            let event = format!("ORDER_EVENT: {} -> {}", key, json_data.len());
            order_events_clone.lock().unwrap().push(event);
        }
    })?;

    // Generate mixed events
    let mixed_events = vec![
        ("user-1", r#"{"name": "Alice", "role": "admin"}"#),
        ("prod-1", r#"{"name": "Product A", "price": 99.99}"#),
        ("order-1", r#"{"user_id": "user-1", "product_id": "prod-1"}"#),
        ("user-2", r#"{"name": "Bob", "role": "user"}"#),
        ("prod-2", r#"{"name": "Product B", "price": 49.99}"#),
        ("order-2", r#"{"user_id": "user-2", "product_id": "prod-2"}"#),
    ];

    for (key, payload) in &mixed_events {
        routing_event_manager.raise_event(EventType::Stash, key, payload)?;
    }

    // Display routed events
    println!("👥 User Events:");
    for event in user_events.lock().unwrap().iter() {
        println!("   {}", event);
    }

    println!("📦 Product Events:");
    for event in product_events.lock().unwrap().iter() {
        println!("   {}", event);
    }

    println!("🛒 Order Events:");
    for event in order_events.lock().unwrap().iter() {
        println!("   {}", event);
    }
    println!();

    // Example 8: Comprehensive Event System
    println!("=== Example 8: Comprehensive Event System ===");

    // Create a comprehensive event system with multiple trees and event managers
    let user_tree = AcornTree::open_memory()?;
    let product_tree = AcornTree::open_memory()?;
    let order_tree = AcornTree::open_memory()?;

    let user_events = AcornEventManager::new(user_tree)?;
    let product_events = AcornEventManager::new(product_tree)?;
    let order_events = AcornEventManager::new(order_tree)?;

    // Create a mesh coordinator for the trees
    let comprehensive_coordinator = AcornMeshCoordinator::new()?;
    comprehensive_coordinator.add_node("user-tree", user_tree)?;
    comprehensive_coordinator.add_node("product-tree", product_tree)?;
    comprehensive_coordinator.add_node("order-tree", order_tree)?;

    // Create full mesh
    comprehensive_coordinator.create_topology(MeshTopology::Full, "")?;

    // Set up cross-tree event handling
    let cross_tree_events = Arc::new(Mutex::new(Vec::new()));

    let cross_tree_clone = cross_tree_events.clone();
    let _user_cross_sub = user_events.subscribe(move |key, json_data| {
        let event = format!("USER_TREE: {} -> {} bytes", key, json_data.len());
        cross_tree_clone.lock().unwrap().push(event);
    })?;

    let cross_tree_clone2 = cross_tree_events.clone();
    let _product_cross_sub = product_events.subscribe(move |key, json_data| {
        let event = format!("PRODUCT_TREE: {} -> {} bytes", key, json_data.len());
        cross_tree_clone2.lock().unwrap().push(event);
    })?;

    let cross_tree_clone3 = cross_tree_events.clone();
    let _order_cross_sub = order_events.subscribe(move |key, json_data| {
        let event = format!("ORDER_TREE: {} -> {} bytes", key, json_data.len());
        cross_tree_clone3.lock().unwrap().push(event);
    })?;

    // Generate events across all trees
    let user_data = r#"{"name": "Alice", "email": "alice@example.com"}"#;
    let product_data = r#"{"name": "AcornDB", "price": 99.99}"#;
    let order_data = r#"{"user_id": "user-1", "product_id": "prod-1"}"#;

    user_events.raise_event(EventType::Stash, "user-1", user_data)?;
    product_events.raise_event(EventType::Stash, "prod-1", product_data)?;
    order_events.raise_event(EventType::Stash, "order-1", order_data)?;

    // Synchronize all trees
    comprehensive_coordinator.synchronize_all()?;

    // Display cross-tree events
    println!("🌐 Cross-Tree Events:");
    for event in cross_tree_events.lock().unwrap().iter() {
        println!("   {}", event);
    }

    // Get comprehensive statistics
    let all_mesh_stats = comprehensive_coordinator.get_all_stats()?;
    println!("📊 Comprehensive Mesh Statistics:");
    for stats in &all_mesh_stats {
        println!("   Tree {}: {} active tangles, {} sync operations", 
            stats.node_id, stats.active_tangles, stats.total_sync_operations);
    }

    println!();
    println!("🎉 Event Management example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("✅ Event Management: Enhanced event system with filtering and routing");
    println!("✅ Tangle Support: Live synchronization between trees");
    println!("✅ Mesh Primitives: Multi-node coordination with different topologies");
    println!("✅ Event Types: Stash, Toss, Squabble, and Sync event handling");
    println!("✅ Event Filtering: Subscribe to specific event types");
    println!("✅ Custom Events: Raise custom events with JSON payloads");
    println!("✅ Topology Management: Full mesh, ring, star, and custom topologies");
    println!("✅ Performance Monitoring: Track event processing performance");
    println!("✅ Event Routing: Route events based on content and type");
    println!("✅ Cross-Tree Events: Coordinate events across multiple trees");
    println!("✅ Statistics Tracking: Comprehensive mesh and tangle statistics");
    println!("✅ Real-time Synchronization: Live sync with loop prevention");

    Ok(())
}
