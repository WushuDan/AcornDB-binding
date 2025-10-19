use acorn::{AcornCache, AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
    data: Vec<String>, // Some data to make items larger
}

fn main() -> Result<(), Error> {
    println!("🚀 AcornDB Advanced Caching Example");
    println!("====================================");

    // Example 1: LRU Cache
    println!("\n1. LRU Cache Strategy");
    let lru_cache = AcornCache::lru(5)?; // Small cache for demonstration
    println!("✅ Created LRU cache with max size 5");
    
    // Check cache properties
    let stats = lru_cache.get_stats()?;
    println!("📊 Initial cache stats:");
    println!("   Max size: {}", stats.max_size);
    println!("   Tracked items: {}", stats.tracked_items);
    println!("   Utilization: {:.1}%", stats.utilization_percentage);
    
    let eviction_enabled = lru_cache.is_eviction_enabled()?;
    println!("🔧 Eviction enabled: {}", eviction_enabled);
    assert!(eviction_enabled);

    // Example 2: No Eviction Cache
    println!("\n2. No Eviction Cache Strategy");
    let no_eviction_cache = AcornCache::no_eviction()?;
    println!("✅ Created no eviction cache");
    
    let no_eviction_stats = no_eviction_cache.get_stats()?;
    println!("📊 No eviction cache stats:");
    println!("   Max size: {}", no_eviction_stats.max_size);
    println!("   Tracked items: {}", no_eviction_stats.tracked_items);
    println!("   Utilization: {:.1}%", no_eviction_stats.utilization_percentage);
    
    let no_eviction_enabled = no_eviction_cache.is_eviction_enabled()?;
    println!("🔧 Eviction enabled: {}", no_eviction_enabled);
    assert!(!no_eviction_enabled);

    // Example 3: Tree with LRU Cache
    println!("\n3. Tree with LRU Cache");
    let mut tree = AcornTree::open_with_cache("memory://lru_test", &lru_cache)?;
    println!("✅ Opened tree with LRU cache");
    
    // Create some test users
    let users = vec![
        User {
            id: "user1".to_string(),
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            data: vec!["data1".to_string(), "data2".to_string(), "data3".to_string()],
        },
        User {
            id: "user2".to_string(),
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            data: vec!["data4".to_string(), "data5".to_string(), "data6".to_string()],
        },
        User {
            id: "user3".to_string(),
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            data: vec!["data7".to_string(), "data8".to_string(), "data9".to_string()],
        },
        User {
            id: "user4".to_string(),
            name: "Diana Prince".to_string(),
            email: "diana@example.com".to_string(),
            data: vec!["data10".to_string(), "data11".to_string(), "data12".to_string()],
        },
        User {
            id: "user5".to_string(),
            name: "Eve Wilson".to_string(),
            email: "eve@example.com".to_string(),
            data: vec!["data13".to_string(), "data14".to_string(), "data15".to_string()],
        },
        User {
            id: "user6".to_string(),
            name: "Frank Miller".to_string(),
            email: "frank@example.com".to_string(),
            data: vec!["data16".to_string(), "data17".to_string(), "data18".to_string()],
        },
    ];
    
    // Store users in the tree
    for user in &users {
        tree.stash(&user.id, user)?;
        println!("📦 Stored user: {}", user.name);
    }
    
    // Check cache stats after storing
    let stats_after_store = lru_cache.get_stats()?;
    println!("📊 Cache stats after storing {} users:", users.len());
    println!("   Tracked items: {}", stats_after_store.tracked_items);
    println!("   Utilization: {:.1}%", stats_after_store.utilization_percentage);
    
    // Retrieve users (this will update LRU access times)
    println!("\n📤 Retrieving users:");
    for user in &users {
        let retrieved: User = tree.crack(&user.id)?;
        assert_eq!(user, &retrieved);
        println!("   Retrieved: {}", retrieved.name);
    }
    
    // Check cache stats after retrieval
    let stats_after_retrieve = lru_cache.get_stats()?;
    println!("📊 Cache stats after retrieval:");
    println!("   Tracked items: {}", stats_after_retrieve.tracked_items);
    println!("   Utilization: {:.1}%", stats_after_retrieve.utilization_percentage);

    // Example 4: Tree with No Eviction Cache
    println!("\n4. Tree with No Eviction Cache");
    let mut no_eviction_tree = AcornTree::open_with_cache("memory://no_eviction_test", &no_eviction_cache)?;
    println!("✅ Opened tree with no eviction cache");
    
    // Store the same users
    for user in &users {
        no_eviction_tree.stash(&user.id, user)?;
    }
    
    // Check stats
    let no_eviction_stats_after = no_eviction_cache.get_stats()?;
    println!("📊 No eviction cache stats after storing:");
    println!("   Tracked items: {}", no_eviction_stats_after.tracked_items);
    println!("   Utilization: {:.1}%", no_eviction_stats_after.utilization_percentage);

    // Example 5: Cache Reset
    println!("\n5. Cache Reset");
    println!("🔄 Resetting LRU cache...");
    lru_cache.reset()?;
    
    let stats_after_reset = lru_cache.get_stats()?;
    println!("📊 Cache stats after reset:");
    println!("   Tracked items: {}", stats_after_reset.tracked_items);
    println!("   Utilization: {:.1}%", stats_after_reset.utilization_percentage);
    assert_eq!(stats_after_reset.tracked_items, 0);

    // Example 6: Cache Performance Comparison
    println!("\n6. Cache Performance Comparison");
    
    // Create a large dataset
    let large_dataset: Vec<User> = (1..=100)
        .map(|i| User {
            id: format!("user{}", i),
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
            data: vec![format!("data{}", j); 10], // 10 data items per user
        })
        .collect();
    
    // Test with small LRU cache
    let small_lru = AcornCache::lru(10)?;
    let mut small_tree = AcornTree::open_with_cache("memory://small_lru", &small_lru)?;
    
    println!("📦 Storing {} users in small LRU cache (max 10)...", large_dataset.len());
    for user in &large_dataset {
        small_tree.stash(&user.id, user)?;
    }
    
    let small_stats = small_lru.get_stats()?;
    println!("📊 Small LRU cache stats:");
    println!("   Tracked items: {}", small_stats.tracked_items);
    println!("   Max size: {}", small_stats.max_size);
    println!("   Utilization: {:.1}%", small_stats.utilization_percentage);
    
    // Test with no eviction cache
    let unlimited_cache = AcornCache::no_eviction()?;
    let mut unlimited_tree = AcornTree::open_with_cache("memory://unlimited", &unlimited_cache)?;
    
    println!("📦 Storing {} users in unlimited cache...", large_dataset.len());
    for user in &large_dataset {
        unlimited_tree.stash(&user.id, user)?;
    }
    
    let unlimited_stats = unlimited_cache.get_stats()?;
    println!("📊 Unlimited cache stats:");
    println!("   Tracked items: {}", unlimited_stats.tracked_items);
    println!("   Max size: {}", unlimited_stats.max_size);
    println!("   Utilization: {:.1}%", unlimited_stats.utilization_percentage);

    // Example 7: Cache Strategy Selection
    println!("\n7. Cache Strategy Selection Guide");
    println!("💡 When to use different cache strategies:");
    println!("   • LRU Cache: When you have limited memory and want to keep frequently accessed items");
    println!("   • No Eviction: When you have unlimited memory or want to keep all items");
    println!("   • Small LRU: For testing or when memory is very limited");
    println!("   • Large LRU: For production with moderate memory constraints");

    // Example 8: Error Handling
    println!("\n8. Error Handling");
    
    // Test invalid cache size
    match AcornCache::lru(0) {
        Ok(_) => println!("❌ Unexpected success with invalid cache size"),
        Err(e) => println!("✅ Expected error with invalid cache size: {}", e),
    }
    
    match AcornCache::lru(-1) {
        Ok(_) => println!("❌ Unexpected success with negative cache size"),
        Err(e) => println!("✅ Expected error with negative cache size: {}", e),
    }

    println!("\n🎉 All caching examples completed successfully!");
    Ok(())
}
