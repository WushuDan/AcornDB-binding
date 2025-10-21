use acorn::{AcornTree, AcornAdvancedTree, Error};
use serde::{Deserialize, Serialize};
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
struct Session {
    id: String,
    user_id: String,
    created_at: SystemTime,
    expires_at: SystemTime,
    ip_address: String,
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

fn main() -> Result<(), Error> {
    println!("🌳 AcornDB Advanced Tree Features Example");
    println!("=========================================");

    // Example 1: Auto-ID Detection
    println!("=== Example 1: Auto-ID Detection ===");

    let mut tree = AcornTree::open_memory()?;
    let advanced_tree = AcornAdvancedTree::from_tree(tree);

    // Create users with auto-ID detection
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

    // Stash users with auto-ID detection
    for user in &users {
        let user_json = serde_json::to_string(user)?;
        advanced_tree.stash_with_auto_id(&user_json)?;
        println!("✅ Stashed user: {} (auto-ID: {})", user.name, user.id);
    }
    println!();

    // Example 2: Tree Statistics
    println!("=== Example 2: Tree Statistics ===");

    let stats = advanced_tree.get_stats()?;
    println!("📊 Tree Statistics:");
    println!("   Total stashed: {}", stats.total_stashed);
    println!("   Total tossed: {}", stats.total_tossed);
    println!("   Squabbles resolved: {}", stats.squabbles_resolved);
    println!("   Smushes performed: {}", stats.smushes_performed);
    println!("   Active tangles: {}", stats.active_tangles);
    println!("   Last sync timestamp: {}", stats.last_sync_timestamp);
    println!();

    // Example 3: TTL Enforcement
    println!("=== Example 3: TTL Enforcement ===");

    // Get current TTL info
    let ttl_info = advanced_tree.get_ttl_info()?;
    println!("⏰ TTL Information:");
    println!("   TTL enforcement enabled: {}", ttl_info.ttl_enforcement_enabled);
    println!("   Cleanup interval: {} ms", ttl_info.cleanup_interval_ms);
    println!("   Expiring nuts count: {}", ttl_info.expiring_nuts_count);

    // Enable TTL enforcement
    advanced_tree.set_ttl_enforcement(true)?;
    println!("✅ TTL enforcement enabled");

    // Set cleanup interval to 30 seconds
    advanced_tree.set_cleanup_interval(30000)?; // 30 seconds
    println!("✅ Cleanup interval set to 30 seconds");

    // Check for expiring nuts in the next hour
    let expiring_count = advanced_tree.get_expiring_nuts_count(3600000)?; // 1 hour
    println!("🔍 Nuts expiring in the next hour: {}", expiring_count);

    // Get IDs of expiring nuts
    let expiring_ids = advanced_tree.get_expiring_nuts(3600000)?;
    if !expiring_ids.is_empty() {
        println!("⚠️  Expiring nuts:");
        for id in &expiring_ids {
            println!("   - {}", id);
        }
    } else {
        println!("✅ No nuts expiring in the next hour");
    }
    println!();

    // Example 4: Manual TTL Cleanup
    println!("=== Example 4: Manual TTL Cleanup ===");

    // Perform manual cleanup
    let removed_count = advanced_tree.cleanup_expired_nuts()?;
    println!("🧹 Manual cleanup removed {} expired nuts", removed_count);
    println!();

    // Example 5: Nut Metadata Access
    println!("=== Example 5: Nut Metadata Access ===");

    // Get all nuts with metadata
    let all_nuts = advanced_tree.get_all_nuts()?;
    println!("🥜 All nuts with metadata:");
    for nut in &all_nuts {
        println!("   ID: {}", nut.id);
        println!("   Timestamp: {}", nut.timestamp);
        println!("   Version: {}", nut.version);
        if let Some(expires_at) = nut.expires_at {
            println!("   Expires at: {}", expires_at);
        } else {
            println!("   Expires at: Never");
        }
        println!("   Payload type: {}", nut.payload.get("name").unwrap_or(&serde_json::Value::Null));
        println!();
    }

    // Get nut count
    let nut_count = advanced_tree.get_nut_count()?;
    println!("📊 Total nuts in tree: {}", nut_count);
    println!();

    // Example 6: Session Management with TTL
    println!("=== Example 6: Session Management with TTL ===");

    // Create sessions with expiration
    let sessions = vec![
        Session {
            id: "session-1".to_string(),
            user_id: "user-1".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + std::time::Duration::from_secs(3600), // 1 hour
            ip_address: "192.168.1.100".to_string(),
        },
        Session {
            id: "session-2".to_string(),
            user_id: "user-2".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + std::time::Duration::from_secs(7200), // 2 hours
            ip_address: "192.168.1.101".to_string(),
        },
        Session {
            id: "session-3".to_string(),
            user_id: "user-3".to_string(),
            created_at: SystemTime::now(),
            expires_at: SystemTime::now() + std::time::Duration::from_secs(1800), // 30 minutes
            ip_address: "192.168.1.102".to_string(),
        },
    ];

    // Stash sessions
    for session in &sessions {
        let session_json = serde_json::to_string(session)?;
        advanced_tree.stash_with_auto_id(&session_json)?;
        println!("🔐 Stashed session: {} (expires in {} minutes)", 
            session.id, 
            session.expires_at.duration_since(SystemTime::now()).unwrap_or_default().as_secs() / 60
        );
    }

    // Check for sessions expiring in the next hour
    let expiring_sessions = advanced_tree.get_expiring_nuts(3600000)?; // 1 hour
    println!("⏰ Sessions expiring in the next hour: {}", expiring_sessions.len());
    for session_id in &expiring_sessions {
        println!("   - {}", session_id);
    }
    println!();

    // Example 7: Product Inventory with Statistics
    println!("=== Example 7: Product Inventory with Statistics ===");

    // Create products
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
        Product {
            id: "prod-3".to_string(),
            name: "Nursery System".to_string(),
            price: 29.99,
            category: "Tools".to_string(),
            stock_count: 25,
            created_at: SystemTime::now(),
        },
    ];

    // Stash products
    for product in &products {
        let product_json = serde_json::to_string(product)?;
        advanced_tree.stash_with_auto_id(&product_json)?;
        println!("📦 Stashed product: {} (${:.2}, stock: {})", 
            product.name, product.price, product.stock_count);
    }

    // Get updated statistics
    let updated_stats = advanced_tree.get_stats()?;
    println!("📊 Updated Statistics:");
    println!("   Total stashed: {}", updated_stats.total_stashed);
    println!("   Total nuts: {}", advanced_tree.get_nut_count()?);
    println!();

    // Example 8: Performance Monitoring
    println!("=== Example 8: Performance Monitoring ===");

    use std::time::Instant;

    // Test auto-ID stash performance
    let start = Instant::now();
    for i in 0..100 {
        let user = User {
            id: format!("perf-user-{}", i),
            name: format!("Performance User {}", i),
            email: format!("perf{}@example.com", i),
            role: "user".to_string(),
            last_login: SystemTime::now(),
            is_active: true,
        };
        let user_json = serde_json::to_string(&user)?;
        advanced_tree.stash_with_auto_id(&user_json)?;
    }
    let stash_time = start.elapsed();

    // Test statistics retrieval performance
    let start = Instant::now();
    let final_stats = advanced_tree.get_stats()?;
    let stats_time = start.elapsed();

    // Test nut count performance
    let start = Instant::now();
    let final_count = advanced_tree.get_nut_count()?;
    let count_time = start.elapsed();

    println!("⚡ Performance Results:");
    println!("   Stashed 100 users in: {:?}", stash_time);
    println!("   Retrieved statistics in: {:?}", stats_time);
    println!("   Retrieved nut count in: {:?}", count_time);
    println!("   Final nut count: {}", final_count);
    println!();

    // Example 9: TTL Configuration Management
    println!("=== Example 9: TTL Configuration Management ===");

    // Test different cleanup intervals
    let intervals = vec![
        (5000, "5 seconds"),
        (30000, "30 seconds"),
        (300000, "5 minutes"),
        (3600000, "1 hour"),
    ];

    for (interval_ms, description) in &intervals {
        advanced_tree.set_cleanup_interval(*interval_ms as i64)?;
        let ttl_info = advanced_tree.get_ttl_info()?;
        println!("⏰ Set cleanup interval to {}: {} ms", description, ttl_info.cleanup_interval_ms);
    }

    // Disable TTL enforcement
    advanced_tree.set_ttl_enforcement(false)?;
    let ttl_info = advanced_tree.get_ttl_info()?;
    println!("🔴 TTL enforcement disabled: {}", !ttl_info.ttl_enforcement_enabled);

    // Re-enable TTL enforcement
    advanced_tree.set_ttl_enforcement(true)?;
    let ttl_info = advanced_tree.get_ttl_info()?;
    println!("🟢 TTL enforcement enabled: {}", ttl_info.ttl_enforcement_enabled);
    println!();

    // Example 10: Comprehensive Tree Analysis
    println!("=== Example 10: Comprehensive Tree Analysis ===");

    // Get comprehensive tree information
    let final_stats = advanced_tree.get_stats()?;
    let final_ttl_info = advanced_tree.get_ttl_info()?;
    let final_count = advanced_tree.get_nut_count()?;
    let last_sync = advanced_tree.get_last_sync_timestamp()?;

    println!("🌳 Comprehensive Tree Analysis:");
    println!("   📊 Statistics:");
    println!("      - Total stashed: {}", final_stats.total_stashed);
    println!("      - Total tossed: {}", final_stats.total_tossed);
    println!("      - Squabbles resolved: {}", final_stats.squabbles_resolved);
    println!("      - Smushes performed: {}", final_stats.smushes_performed);
    println!("      - Active tangles: {}", final_stats.active_tangles);
    println!("   ⏰ TTL Information:");
    println!("      - TTL enforcement: {}", final_ttl_info.ttl_enforcement_enabled);
    println!("      - Cleanup interval: {} ms", final_ttl_info.cleanup_interval_ms);
    println!("      - Expiring nuts: {}", final_ttl_info.expiring_nuts_count);
    println!("   📈 Tree State:");
    println!("      - Total nuts: {}", final_count);
    println!("      - Last sync: {}", last_sync);

    // Get all nuts summary
    let all_nuts = advanced_tree.get_all_nuts()?;
    let user_nuts = all_nuts.iter().filter(|nut| nut.payload.get("email").is_some()).count();
    let session_nuts = all_nuts.iter().filter(|nut| nut.payload.get("ip_address").is_some()).count();
    let product_nuts = all_nuts.iter().filter(|nut| nut.payload.get("price").is_some()).count();

    println!("   🥜 Nut Breakdown:");
    println!("      - Users: {}", user_nuts);
    println!("      - Sessions: {}", session_nuts);
    println!("      - Products: {}", product_nuts);
    println!("      - Total: {}", all_nuts.len());
    println!();

    println!("🎉 Advanced Tree Features example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("✅ Auto-ID Detection: Automatic ID extraction from object properties");
    println!("✅ Tree Statistics: Comprehensive operation tracking and metrics");
    println!("✅ TTL Enforcement: Time-to-live management with automatic cleanup");
    println!("✅ Manual Cleanup: On-demand removal of expired items");
    println!("✅ Expiring Nuts Queries: Find items expiring within timeframes");
    println!("✅ Nut Metadata Access: Full access to nut metadata and payloads");
    println!("✅ Performance Monitoring: Track operation performance and timing");
    println!("✅ TTL Configuration: Dynamic TTL enforcement and cleanup intervals");
    println!("✅ Comprehensive Analysis: Complete tree state and statistics");
    println!("✅ Session Management: Real-world TTL usage for session handling");
    println!("✅ Product Inventory: Business use case with statistics tracking");

    Ok(())
}
