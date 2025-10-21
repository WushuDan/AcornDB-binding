use acorn::{AcornTree, AcornGit, AcornStorage, Error};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Document {
    id: String,
    title: String,
    content: String,
    author: String,
    created_at: SystemTime,
    version: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    role: String,
    last_modified: SystemTime,
}

fn main() -> Result<(), Error> {
    println!("🌰 AcornDB Git Integration Example");
    println!("=================================");

    // Example 1: Git-backed storage with AcornTree
    println!("=== Example 1: Git-backed Storage ===");

    // Create Git storage backend
    let git_storage = AcornStorage::git(
        "./git-database",
        "AcornDB User",
        "user@acorndb.dev",
        false // Don't auto-push
    )?;
    println!("✅ Created Git storage backend");

    // Open tree with Git storage
    let mut tree = AcornTree::open_with_storage(git_storage)?;
    println!("✅ Opened tree with Git storage");
    println!();

    // Example 2: Document versioning with Git history
    println!("=== Example 2: Document Versioning ===");

    let documents = vec![
        ("doc-1", Document {
            id: "doc-1".to_string(),
            title: "Getting Started Guide".to_string(),
            content: "Welcome to AcornDB! This is a comprehensive guide...".to_string(),
            author: "Alice Johnson".to_string(),
            created_at: SystemTime::now(),
            version: 1,
        }),
        ("doc-2", Document {
            id: "doc-2".to_string(),
            title: "API Reference".to_string(),
            content: "Complete API documentation for AcornDB...".to_string(),
            author: "Bob Smith".to_string(),
            created_at: SystemTime::now(),
            version: 1,
        }),
        ("doc-3", Document {
            id: "doc-3".to_string(),
            title: "Best Practices".to_string(),
            content: "Learn the best practices for using AcornDB effectively...".to_string(),
            author: "Charlie Brown".to_string(),
            created_at: SystemTime::now(),
            version: 1,
        }),
    ];

    // Store documents (each stash creates a Git commit)
    for (id, doc) in &documents {
        tree.stash(id, doc)?;
        println!("📝 Stashed document: {} (version {})", doc.title, doc.version);
    }
    println!();

    // Update a document (creates new commit)
    let mut updated_doc = documents[0].1.clone();
    updated_doc.content = "Welcome to AcornDB! This is a comprehensive guide with updated information...".to_string();
    updated_doc.version = 2;
    
    tree.stash("doc-1", &updated_doc)?;
    println!("📝 Updated document: {} (version {})", updated_doc.title, updated_doc.version);
    println!();

    // Example 3: Git operations and history
    println!("=== Example 3: Git Operations ===");

    // Create Git integration instance
    let git = AcornGit::new(
        "./git-database",
        "AcornDB User",
        "user@acorndb.dev",
        false
    )?;
    println!("✅ Created Git integration instance");

    // Check if repository has remote
    let has_remote = git.has_remote("origin")?;
    println!("🔗 Has remote 'origin': {}", has_remote);

    // Get file history for a document
    println!("\n📚 File history for doc-1:");
    let commits = git.get_file_history("doc-1.json")?;
    for (i, commit) in commits.iter().enumerate() {
        println!("  {}. Commit {}: {}", i + 1, &commit.sha[..7], commit.message);
        println!("     Author: {} <{}>", commit.author, commit.email);
        println!("     Timestamp: {}", commit.timestamp);
    }

    // Read file content at specific commit (if we have commits)
    if !commits.is_empty() {
        println!("\n📖 Reading doc-1 at first commit:");
        let content = git.read_file_at_commit("doc-1.json", &commits[0].sha)?;
        if !content.is_empty() {
            println!("   Content preview: {}...", &content[..content.len().min(100)]);
        }
    }
    println!();

    // Example 4: User management with Git tracking
    println!("=== Example 4: User Management ===");

    let users = vec![
        ("user-1", User {
            id: "user-1".to_string(),
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            role: "admin".to_string(),
            last_modified: SystemTime::now(),
        }),
        ("user-2", User {
            id: "user-2".to_string(),
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            role: "user".to_string(),
            last_modified: SystemTime::now(),
        }),
        ("user-3", User {
            id: "user-3".to_string(),
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            role: "moderator".to_string(),
            last_modified: SystemTime::now(),
        }),
    ];

    // Store users (each creates a Git commit)
    for (id, user) in &users {
        tree.stash(id, user)?;
        println!("👤 Stashed user: {} ({})", user.name, user.role);
    }

    // Update user role (creates new commit)
    let mut updated_user = users[1].1.clone();
    updated_user.role = "admin".to_string();
    updated_user.last_modified = SystemTime::now();
    
    tree.stash("user-2", &updated_user)?;
    println!("👤 Updated user: {} (role: {})", updated_user.name, updated_user.role);
    println!();

    // Example 5: Git history analysis
    println!("=== Example 5: Git History Analysis ===");

    // Get history for all files
    let files = vec!["doc-1.json", "doc-2.json", "doc-3.json", "user-1.json", "user-2.json", "user-3.json"];
    
    for file in &files {
        let commits = git.get_file_history(file)?;
        if !commits.is_empty() {
            println!("📄 {}: {} commits", file, commits.len());
            for commit in &commits {
                println!("   - {}: {}", &commit.sha[..7], commit.message);
            }
        }
    }
    println!();

    // Example 6: Advanced Git operations
    println!("=== Example 6: Advanced Git Operations ===");

    // Demonstrate commit squashing (if we have multiple commits)
    if commits.len() > 1 {
        println!("🔄 Squashing commits since first commit...");
        git.squash_commits(&commits[0].sha)?;
        println!("✅ Commits squashed successfully");
    }

    // Demonstrate push/pull operations (if remote exists)
    if has_remote {
        println!("\n⬆️  Pushing to remote...");
        git.push("origin", "main")?;
        println!("✅ Pushed to origin/main");

        println!("\n⬇️  Pulling from remote...");
        git.pull("origin", "main")?;
        println!("✅ Pulled from origin/main");
    } else {
        println!("ℹ️  No remote configured, skipping push/pull operations");
    }
    println!();

    // Example 7: Querying Git-tracked data
    println!("=== Example 7: Querying Git-tracked Data ===");

    // Query all documents
    let all_docs: Vec<Document> = tree.query()
        .where_condition(|item| item["title"].as_str().is_some())
        .collect()?;
    println!("📚 Found {} documents", all_docs.len());

    // Query all users
    let all_users: Vec<User> = tree.query()
        .where_condition(|item| item["email"].as_str().is_some())
        .collect()?;
    println!("👥 Found {} users", all_users.len());

    // Query admin users
    let admin_users: Vec<User> = tree.query()
        .where_condition(|item| item["role"].as_str() == Some("admin"))
        .collect()?;
    println!("👑 Found {} admin users", admin_users.len());

    for user in &admin_users {
        println!("   - {} ({})", user.name, user.email);
    }
    println!();

    // Example 8: Performance comparison
    println!("=== Example 8: Performance Comparison ===");
    use std::time::Instant;

    // Traditional iteration
    let start = Instant::now();
    let mut traditional_count = 0;
    let mut iter = tree.iter("")?;
    while let Some((_, _)) = iter.next::<Document>()? {
        traditional_count += 1;
    }
    let traditional_time = start.elapsed();

    // Git-backed query
    let start = Instant::now();
    let query_count = tree.query()
        .where_condition(|item| item["version"].as_u64().unwrap_or(0) > 0)
        .count()?;
    let query_time = start.elapsed();

    println!("✓ Traditional iteration: {} items in {:?}", traditional_count, traditional_time);
    println!("✓ Git-backed query: {} items in {:?}", query_count, query_time);
    println!();

    println!("🎉 Git Integration example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("✅ Git-backed storage: Every stash creates a Git commit");
    println!("✅ Version history: Track all changes with Git history");
    println!("✅ Git operations: Push, pull, squash commits");
    println!("✅ File history: Get commit history for specific files");
    println!("✅ Time travel: Read file content at specific commits");
    println!("✅ Remote management: Check and manage Git remotes");
    println!("✅ Query integration: Query Git-tracked data efficiently");
    println!("✅ Performance: Fast operations with Git backend");

    Ok(())
}
