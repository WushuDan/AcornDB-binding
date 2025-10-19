use acorn::{AcornStorage, AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
    created_at: String,
}

fn main() -> Result<(), Error> {
    println!("🗄️ AcornDB Advanced Storage Backends Example");
    println!("=============================================");

    // Example 1: SQLite Storage Backend
    println!("\n1. SQLite Storage Backend");
    let sqlite_storage = AcornStorage::sqlite("./test_storage.db", Some("users"))?;
    println!("✅ Created SQLite storage backend");
    
    let sqlite_info = sqlite_storage.get_info()?;
    println!("📊 SQLite Storage Info:");
    println!("   Provider: {}", sqlite_info.provider_name);
    println!("   Trunk Type: {}", sqlite_info.trunk_type);
    println!("   Durable: {}", sqlite_info.is_durable);
    println!("   Supports History: {}", sqlite_info.supports_history);
    println!("   Supports Sync: {}", sqlite_info.supports_sync);
    println!("   Supports Async: {}", sqlite_info.supports_async);
    
    let is_connected = sqlite_storage.test_connection()?;
    println!("🔗 Connection Test: {}", if is_connected { "✅ Success" } else { "❌ Failed" });
    
    // Use SQLite storage with tree
    let mut sqlite_tree = AcornTree::open_with_storage(&sqlite_storage)?;
    println!("🌳 Opened tree with SQLite storage");
    
    // Store some test data
    let user1 = User {
        id: "user1".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        created_at: "2023-01-01T10:00:00Z".to_string(),
    };
    
    sqlite_tree.stash("user1", &user1)?;
    println!("📦 Stored user1 in SQLite");
    
    // Retrieve data
    let retrieved_user: User = sqlite_tree.crack("user1")?;
    assert_eq!(user1, retrieved_user);
    println!("📤 Retrieved user1 from SQLite: {}", retrieved_user.name);

    // Example 2: Memory Storage (for comparison)
    println!("\n2. Memory Storage (Default)");
    let mut memory_tree = AcornTree::open("memory://test")?;
    println!("✅ Created memory tree");
    
    let user2 = User {
        id: "user2".to_string(),
        name: "Bob Wilson".to_string(),
        email: "bob@example.com".to_string(),
        created_at: "2023-01-01T11:00:00Z".to_string(),
    };
    
    memory_tree.stash("user2", &user2)?;
    println!("📦 Stored user2 in memory");
    
    let retrieved_user2: User = memory_tree.crack("user2")?;
    assert_eq!(user2, retrieved_user2);
    println!("📤 Retrieved user2 from memory: {}", retrieved_user2.name);

    // Example 3: Storage Backend Comparison
    println!("\n3. Storage Backend Comparison");
    
    let storage_types = vec![
        ("SQLite", AcornStorage::sqlite("./comparison.db", None)?),
    ];
    
    for (name, storage) in storage_types {
        let info = storage.get_info()?;
        let connected = storage.test_connection()?;
        
        println!("📊 {} Storage:");
        println!("   Provider: {}", info.provider_name);
        println!("   Durable: {}", info.is_durable);
        println!("   History: {}", info.supports_history);
        println!("   Sync: {}", info.supports_sync);
        println!("   Async: {}", info.supports_async);
        println!("   Connected: {}", if connected { "✅" } else { "❌" });
        println!();
    }

    // Example 4: Multiple Users in SQLite
    println!("\n4. Multiple Users in SQLite Storage");
    
    let users = vec![
        User {
            id: "user3".to_string(),
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            created_at: "2023-01-01T12:00:00Z".to_string(),
        },
        User {
            id: "user4".to_string(),
            name: "Diana Prince".to_string(),
            email: "diana@example.com".to_string(),
            created_at: "2023-01-01T13:00:00Z".to_string(),
        },
        User {
            id: "user5".to_string(),
            name: "Eve Smith".to_string(),
            email: "eve@example.com".to_string(),
            created_at: "2023-01-01T14:00:00Z".to_string(),
        },
    ];
    
    for user in &users {
        sqlite_tree.stash(&user.id, user)?;
        println!("📦 Stored {} in SQLite", user.name);
    }
    
    // List all users
    let all_keys = sqlite_tree.list()?;
    println!("📋 All users in SQLite: {} items", all_keys.len());
    
    for key in &all_keys {
        let user: User = sqlite_tree.crack(key)?;
        println!("   • {} ({})", user.name, user.email);
    }

    // Example 5: Storage Backend Best Practices
    println!("\n5. Storage Backend Best Practices");
    println!("💡 When to use different storage backends:");
    println!("   • SQLite: Local development, single-user apps, embedded systems");
    println!("   • PostgreSQL: Multi-user apps, complex queries, ACID compliance");
    println!("   • MySQL: Web applications, high concurrency");
    println!("   • SQL Server: Enterprise Windows environments");
    println!("   • S3/Azure: Cloud storage, scalability, backup");
    println!("   • Git: Version control, collaboration, audit trails");
    println!();
    println!("🔧 Tips:");
    println!("   • Use SQLite for development and testing");
    println!("   • Use PostgreSQL for production applications");
    println!("   • Use cloud storage for scalability and backup");
    println!("   • Use Git storage for version-controlled data");
    println!("   • Test connections before using storage backends");
    println!("   • Consider durability vs performance trade-offs");

    // Example 6: Error Handling
    println!("\n6. Error Handling");
    
    // Test invalid SQLite path
    match AcornStorage::sqlite("/invalid/path/that/does/not/exist.db", None) {
        Ok(_) => println!("❌ Unexpected success with invalid path"),
        Err(e) => println!("✅ Expected error with invalid path: {}", e),
    }
    
    // Test invalid table name
    match AcornStorage::sqlite("./test.db", Some("invalid-table-name!")) {
        Ok(_) => println!("❌ Unexpected success with invalid table name"),
        Err(e) => println!("✅ Expected error with invalid table name: {}", e),
    }

    // Example 7: Storage Backend Features
    println!("\n7. Storage Backend Features");
    
    let sqlite_info = sqlite_storage.get_info()?;
    println!("📊 SQLite Features:");
    println!("   • Durable Storage: {}", sqlite_info.is_durable);
    println!("   • Sync Support: {}", sqlite_info.supports_sync);
    println!("   • History Support: {}", sqlite_info.supports_history);
    println!("   • Async Support: {}", sqlite_info.supports_async);
    
    if sqlite_info.supports_history {
        println!("   ✅ Can track data changes over time");
    } else {
        println!("   ❌ No historical data tracking");
    }
    
    if sqlite_info.supports_sync {
        println!("   ✅ Can synchronize with other trees");
    } else {
        println!("   ❌ No synchronization support");
    }

    // Example 8: Performance Considerations
    println!("\n8. Performance Considerations");
    println!("⚡ Storage Backend Performance:");
    println!("   • Memory: Fastest, but not persistent");
    println!("   • SQLite: Fast, good for single-user apps");
    println!("   • PostgreSQL: Good performance, excellent for multi-user");
    println!("   • MySQL: Good performance, web-optimized");
    println!("   • Cloud Storage: Network dependent, highly scalable");
    println!("   • Git: Slower writes, excellent for versioning");

    println!("\n🎉 All storage backend examples completed successfully!");
    Ok(())
}
