use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Document {
    doc_id: String,
    title: String,
    content: String,
    version: i32,
}

fn main() -> Result<(), Error> {
    println!("AcornDB HTTP Sync Example");
    println!();

    // Open a local tree with memory storage
    let mut tree = AcornTree::open("memory://")?;
    println!("✓ Opened local database");
    println!();

    // Store some local documents
    println!("=== Creating Local Documents ===");
    let docs = vec![
        Document {
            doc_id: "doc-001".to_string(),
            title: "Introduction".to_string(),
            content: "Welcome to AcornDB".to_string(),
            version: 1,
        },
        Document {
            doc_id: "doc-002".to_string(),
            title: "Getting Started".to_string(),
            content: "First steps with AcornDB".to_string(),
            version: 1,
        },
    ];

    for doc in &docs {
        tree.stash(&doc.doc_id, doc)?;
        println!("✓ Created: {} - {}", doc.doc_id, doc.title);
    }
    println!();

    // Demonstrate sync API usage
    println!("=== HTTP Sync API ===");
    println!("The sync_http() method synchronizes with a remote HTTP endpoint:");
    println!();
    println!("  tree.sync_http(\"http://example.com/api/acorn\")?;");
    println!();
    println!("This would:");
    println!("  1. Connect to the remote AcornDB HTTP endpoint");
    println!("  2. Pull all remote documents");
    println!("  3. Merge them into the local tree using conflict resolution");
    println!();

    // Note: We can't actually test this without a running server
    println!("Note: To test sync functionality, you need:");
    println!("  - A running AcornDB HTTP server (e.g., using the C# AcornDB.Http library)");
    println!("  - The server should expose endpoints like:");
    println!("    GET  /bark/{{treename}}/export  - Export all nuts");
    println!("    POST /bark/{{treename}}/stash/{{id}} - Store a nut");
    println!();

    // Show what the code would look like
    println!("=== Example Usage ===");
    println!("```rust");
    println!("// Connect to remote server and sync");
    println!("match tree.sync_http(\"http://localhost:5000/api/acorn\") {{");
    println!("    Ok(()) => println!(\"Sync successful!\"),");
    println!("    Err(e) => eprintln!(\"Sync failed: {{}}\", e),");
    println!("}}");
    println!("```");
    println!();

    // Demonstrate error handling
    println!("=== Sync Behavior ===");
    println!("Note: sync_http() is fault-tolerant.");
    println!("Network errors are logged but don't cause the operation to fail.");
    println!("This is by design - sync is best-effort.");
    println!();

    let invalid_url = "http://nonexistent.example.com:9999/acorn";
    println!("Attempting to sync with unreachable URL: {}", invalid_url);
    match tree.sync_http(invalid_url) {
        Ok(()) => println!("✓ Method returned Ok (errors logged above)"),
        Err(e) => println!("✗ Method returned Err: {}", e),
    }
    println!();

    println!("🎉 Sync example completed!");
    println!();
    println!("To test with a real server:");
    println!("  1. Set up an AcornDB HTTP server");
    println!("  2. Replace the URL with your server's URL");
    println!("  3. Run: cargo run --example sync_usage");

    Ok(())
}
