use acorn::{AcornConflictJudge, AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
    version: u32,
    last_modified: String,
}

fn main() -> Result<(), Error> {
    println!("⚖️ AcornDB Conflict Resolution Example");
    println!("=====================================");

    // Example 1: Timestamp-based conflict resolution (Last Write Wins)
    println!("\n1. Timestamp-based Conflict Resolution (Last Write Wins)");
    let timestamp_judge = AcornConflictJudge::timestamp()?;
    println!("✅ Created timestamp judge");
    
    let judge_name = timestamp_judge.name()?;
    println!("🔧 Judge name: {}", judge_name);
    assert_eq!(judge_name, "Timestamp");

    // Test conflict resolution with different timestamps
    let local_user = User {
        id: "user1".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        version: 1,
        last_modified: "2023-01-01T10:00:00Z".to_string(),
    };

    let incoming_user = User {
        id: "user1".to_string(),
        name: "Alice Smith".to_string(), // Different name
        email: "alice.smith@example.com".to_string(), // Different email
        version: 1,
        last_modified: "2023-01-01T11:00:00Z".to_string(), // Later timestamp
    };

    let local_json = serde_json::to_string(&local_user).unwrap();
    let incoming_json = serde_json::to_string(&incoming_user).unwrap();
    
    let winner_json = timestamp_judge.resolve_conflict(&local_json, &incoming_json)?;
    let winner: User = serde_json::from_str(&winner_json).unwrap();
    
    println!("📊 Conflict Resolution Result:");
    println!("   Local: {} ({})", local_user.name, local_user.last_modified);
    println!("   Incoming: {} ({})", incoming_user.name, incoming_user.last_modified);
    println!("   Winner: {} ({})", winner.name, winner.last_modified);
    
    // Timestamp judge should pick the incoming user (later timestamp)
    assert_eq!(winner.name, "Alice Smith");
    assert_eq!(winner.last_modified, "2023-01-01T11:00:00Z");

    // Example 2: Version-based conflict resolution
    println!("\n2. Version-based Conflict Resolution");
    let version_judge = AcornConflictJudge::version()?;
    println!("✅ Created version judge");
    
    let version_name = version_judge.name()?;
    println!("🔧 Judge name: {}", version_name);
    assert_eq!(version_name, "Version");

    // Test conflict resolution with different versions
    let local_user_v2 = User {
        id: "user2".to_string(),
        name: "Bob Wilson".to_string(),
        email: "bob@example.com".to_string(),
        version: 2,
        last_modified: "2023-01-01T12:00:00Z".to_string(),
    };

    let incoming_user_v3 = User {
        id: "user2".to_string(),
        name: "Robert Wilson".to_string(), // Different name
        email: "robert@example.com".to_string(), // Different email
        version: 3, // Higher version
        last_modified: "2023-01-01T10:00:00Z".to_string(), // Earlier timestamp
    };

    let local_v2_json = serde_json::to_string(&local_user_v2).unwrap();
    let incoming_v3_json = serde_json::to_string(&incoming_user_v3).unwrap();
    
    let version_winner_json = version_judge.resolve_conflict(&local_v2_json, &incoming_v3_json)?;
    let version_winner: User = serde_json::from_str(&version_winner_json).unwrap();
    
    println!("📊 Version Conflict Resolution Result:");
    println!("   Local: {} (v{})", local_user_v2.name, local_user_v2.version);
    println!("   Incoming: {} (v{})", incoming_user_v3.name, incoming_user_v3.version);
    println!("   Winner: {} (v{})", version_winner.name, version_winner.version);
    
    // Version judge should pick the incoming user (higher version)
    assert_eq!(version_winner.name, "Robert Wilson");
    assert_eq!(version_winner.version, 3);

    // Example 3: Local Wins conflict resolution
    println!("\n3. Local Wins Conflict Resolution");
    let local_wins_judge = AcornConflictJudge::local_wins()?;
    println!("✅ Created local wins judge");
    
    let local_wins_name = local_wins_judge.name()?;
    println!("🔧 Judge name: {}", local_wins_name);
    assert_eq!(local_wins_name, "LocalWins");

    // Test local wins resolution
    let local_wins_winner_json = local_wins_judge.resolve_conflict(&local_json, &incoming_json)?;
    let local_wins_winner: User = serde_json::from_str(&local_wins_winner_json).unwrap();
    
    println!("📊 Local Wins Conflict Resolution Result:");
    println!("   Local: {} ({})", local_user.name, local_user.last_modified);
    println!("   Incoming: {} ({})", incoming_user.name, incoming_user.last_modified);
    println!("   Winner: {} ({})", local_wins_winner.name, local_wins_winner.last_modified);
    
    // Local wins judge should always pick the local user
    assert_eq!(local_wins_winner.name, "Alice Johnson");
    assert_eq!(local_wins_winner.last_modified, "2023-01-01T10:00:00Z");

    // Example 4: Remote Wins conflict resolution
    println!("\n4. Remote Wins Conflict Resolution");
    let remote_wins_judge = AcornConflictJudge::remote_wins()?;
    println!("✅ Created remote wins judge");
    
    let remote_wins_name = remote_wins_judge.name()?;
    println!("🔧 Judge name: {}", remote_wins_name);
    assert_eq!(remote_wins_name, "RemoteWins");

    // Test remote wins resolution
    let remote_wins_winner_json = remote_wins_judge.resolve_conflict(&local_json, &incoming_json)?;
    let remote_wins_winner: User = serde_json::from_str(&remote_wins_winner_json).unwrap();
    
    println!("📊 Remote Wins Conflict Resolution Result:");
    println!("   Local: {} ({})", local_user.name, local_user.last_modified);
    println!("   Incoming: {} ({})", incoming_user.name, incoming_user.last_modified);
    println!("   Winner: {} ({})", remote_wins_winner.name, remote_wins_winner.last_modified);
    
    // Remote wins judge should always pick the incoming user
    assert_eq!(remote_wins_winner.name, "Alice Smith");
    assert_eq!(remote_wins_winner.last_modified, "2023-01-01T11:00:00Z");

    // Example 5: Tree with conflict resolution
    println!("\n5. Tree with Conflict Resolution");
    let judge = AcornConflictJudge::timestamp()?;
    let mut tree = AcornTree::open_with_conflict_judge("memory://conflict_test", &judge)?;
    println!("✅ Opened tree with timestamp conflict judge");
    
    // Store initial data
    let initial_user = User {
        id: "conflict-user".to_string(),
        name: "Initial User".to_string(),
        email: "initial@example.com".to_string(),
        version: 1,
        last_modified: "2023-01-01T09:00:00Z".to_string(),
    };
    
    tree.stash("conflict-user", &initial_user)?;
    println!("📦 Stored initial user");
    
    // Retrieve and verify
    let retrieved: User = tree.crack("conflict-user")?;
    assert_eq!(retrieved.name, "Initial User");
    println!("📤 Retrieved user: {}", retrieved.name);

    // Example 6: Conflict resolution strategies comparison
    println!("\n6. Conflict Resolution Strategies Comparison");
    
    let strategies = vec![
        ("Timestamp", AcornConflictJudge::timestamp()?),
        ("Version", AcornConflictJudge::version()?),
        ("LocalWins", AcornConflictJudge::local_wins()?),
        ("RemoteWins", AcornConflictJudge::remote_wins()?),
    ];
    
    let test_local = User {
        id: "test".to_string(),
        name: "Local User".to_string(),
        email: "local@example.com".to_string(),
        version: 1,
        last_modified: "2023-01-01T10:00:00Z".to_string(),
    };
    
    let test_incoming = User {
        id: "test".to_string(),
        name: "Remote User".to_string(),
        email: "remote@example.com".to_string(),
        version: 2,
        last_modified: "2023-01-01T11:00:00Z".to_string(),
    };
    
    let test_local_json = serde_json::to_string(&test_local).unwrap();
    let test_incoming_json = serde_json::to_string(&test_incoming).unwrap();
    
    println!("📊 Strategy Comparison:");
    println!("   Local: {} (v{}, {})", test_local.name, test_local.version, test_local.last_modified);
    println!("   Incoming: {} (v{}, {})", test_incoming.name, test_incoming.version, test_incoming.last_modified);
    println!();
    
    for (strategy_name, judge) in strategies {
        let winner_json = judge.resolve_conflict(&test_local_json, &test_incoming_json)?;
        let winner: User = serde_json::from_str(&winner_json).unwrap();
        println!("   {}: {} (v{}, {})", strategy_name, winner.name, winner.version, winner.last_modified);
    }

    // Example 7: Error handling
    println!("\n7. Error Handling");
    
    // Test invalid JSON
    match timestamp_judge.resolve_conflict("invalid-json", &incoming_json) {
        Ok(_) => println!("❌ Unexpected success with invalid JSON"),
        Err(e) => println!("✅ Expected error with invalid JSON: {}", e),
    }
    
    // Test mismatched JSON structures
    let mismatched_json = r#"{"different": "structure"}"#;
    match timestamp_judge.resolve_conflict(&local_json, mismatched_json) {
        Ok(_) => println!("❌ Unexpected success with mismatched JSON"),
        Err(e) => println!("✅ Expected error with mismatched JSON: {}", e),
    }

    // Example 8: Conflict resolution best practices
    println!("\n8. Conflict Resolution Best Practices");
    println!("💡 When to use different conflict resolution strategies:");
    println!("   • Timestamp: Simple CRUD apps, single-user scenarios");
    println!("   • Version: Multi-user apps with explicit version tracking");
    println!("   • LocalWins: Read-only replicas, local changes take precedence");
    println!("   • RemoteWins: Remote source is authoritative");
    println!();
    println!("🔧 Tips:");
    println!("   • Use timestamps for simple last-write-wins scenarios");
    println!("   • Use version numbers for collaborative applications");
    println!("   • Consider custom merge strategies for complex data");
    println!("   • Test conflict resolution with realistic data");

    println!("\n🎉 All conflict resolution examples completed successfully!");
    Ok(())
}
