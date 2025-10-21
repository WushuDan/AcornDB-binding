use acorn::{AcornDocumentStore, AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct User {
    id: String,
    name: String,
    email: String,
    version: i32,
}

fn main() -> Result<(), Error> {
    println!("🌰 AcornDB Document Store Example");
    println!("================================");

    // Create a document store with custom path
    println!("\n1. Creating document store...");
    let doc_store = AcornDocumentStore::new(Some("./example_data"))?;
    println!("✅ Document store created");

    // Get document store info
    println!("\n2. Getting document store info...");
    let info = doc_store.get_info()?;
    println!("📊 Document Store Info:");
    println!("   - Trunk Type: {}", info.trunk_type);
    println!("   - Supports History: {}", info.supports_history);
    println!("   - Is Durable: {}", info.is_durable);
    println!("   - Provider: {}", info.provider_name);
    println!("   - Has Change Log: {}", info.has_change_log);
    println!("   - Total Versions: {}", info.total_versions);

    // Open a tree with the document store
    println!("\n3. Opening tree with document store...");
    let mut tree = AcornTree::open_with_document_store(&doc_store)?;
    println!("✅ Tree opened with document store");

    // Create some users with versioning
    println!("\n4. Creating users with versioning...");
    
    let user1_v1 = User {
        id: "user-1".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        version: 1,
    };
    
    let user1_v2 = User {
        id: "user-1".to_string(),
        name: "Alice Smith".to_string(), // Changed name
        email: "alice.smith@example.com".to_string(), // Changed email
        version: 2,
    };

    let user2 = User {
        id: "user-2".to_string(),
        name: "Bob Wilson".to_string(),
        email: "bob@example.com".to_string(),
        version: 1,
    };

    // Store initial versions
    tree.stash("user-1", &user1_v1)?;
    tree.stash("user-2", &user2)?;
    println!("✅ Stored user-1 v1 and user-2 v1");

    // Update user-1 to version 2
    tree.stash("user-1", &user1_v2)?;
    println!("✅ Updated user-1 to v2");

    // Get current versions
    println!("\n5. Getting current versions...");
    let current_user1: User = tree.crack("user-1")?;
    let current_user2: User = tree.crack("user-2")?;
    
    println!("📄 Current User 1: {:?}", current_user1);
    println!("📄 Current User 2: {:?}", current_user2);

    // Get version history for user-1
    println!("\n6. Getting version history...");
    let history_json = doc_store.get_history("user-1")?;
    println!("📜 History for user-1:");
    println!("{}", history_json);

    // Parse and display history
    let history: Vec<serde_json::Value> = serde_json::from_str(&history_json)?;
    println!("\n📚 Parsed History:");
    for (i, version) in history.iter().enumerate() {
        println!("   Version {}: {}", i + 1, version);
    }

    // Get updated document store info
    println!("\n7. Updated document store info...");
    let updated_info = doc_store.get_info()?;
    println!("📊 Updated Info:");
    println!("   - Has Change Log: {}", updated_info.has_change_log);
    println!("   - Total Versions: {}", updated_info.total_versions);

    // Demonstrate compaction
    println!("\n8. Compacting document store...");
    doc_store.compact()?;
    println!("✅ Document store compacted");

    // Get final info
    println!("\n9. Final document store info...");
    let final_info = doc_store.get_info()?;
    println!("📊 Final Info:");
    println!("   - Total Versions: {}", final_info.total_versions);

    println!("\n🎉 Document Store example completed successfully!");
    println!("\nKey Features Demonstrated:");
    println!("✅ Document store creation with custom path");
    println!("✅ Tree integration with document store");
    println!("✅ Version history tracking");
    println!("✅ Document store information and capabilities");
    println!("✅ Compaction operation");
    println!("✅ Time-travel through version history");

    Ok(())
}
