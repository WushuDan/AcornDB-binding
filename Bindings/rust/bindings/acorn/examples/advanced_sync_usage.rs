use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
    department: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Document {
    id: String,
    title: String,
    content: String,
    author: String,
    version: u32,
}

fn main() -> Result<(), Error> {
    println!("🌐 Advanced Sync Usage Examples");
    println!("================================");

    // Example 1: Mesh Sync with Full Mesh Topology
    println!("\n1. Mesh Sync - Full Mesh Topology");
    mesh_full_mesh_example()?;

    // Example 2: Mesh Sync with Star Topology
    println!("\n2. Mesh Sync - Star Topology");
    mesh_star_example()?;

    // Example 3: Mesh Sync with Ring Topology
    println!("\n3. Mesh Sync - Ring Topology");
    mesh_ring_example()?;

    // Example 4: Peer-to-Peer Bidirectional Sync
    println!("\n4. Peer-to-Peer - Bidirectional Sync");
    p2p_bidirectional_example()?;

    // Example 5: Peer-to-Peer Push-Only Sync
    println!("\n5. Peer-to-Peer - Push-Only Sync");
    p2p_push_only_example()?;

    // Example 6: Peer-to-Peer Pull-Only Sync
    println!("\n6. Peer-to-Peer - Pull-Only Sync");
    p2p_pull_only_example()?;

    // Example 7: Complex Mesh with Mixed Data Types
    println!("\n7. Complex Mesh with Mixed Data Types");
    complex_mesh_example()?;

    println!("\n✅ All Advanced Sync examples completed successfully!");
    Ok(())
}

fn mesh_full_mesh_example() -> Result<(), Error> {
    // Create a mesh coordinator
    let mesh = AcornTree::create_mesh()?;
    
    // Create multiple trees representing different nodes
    let mut node1_tree = AcornTree::open("memory://")?;
    let mut node2_tree = AcornTree::open("memory://")?;
    let mut node3_tree = AcornTree::open("memory://")?;
    
    // Add nodes to the mesh
    mesh.add_node("node1", &node1_tree)?;
    mesh.add_node("node2", &node2_tree)?;
    mesh.add_node("node3", &node3_tree)?;
    
    println!("  Added 3 nodes to mesh");
    
    // Create full mesh topology (every node connects to every other node)
    mesh.create_full_mesh()?;
    println!("  Created full mesh topology");
    
    // Add different data to each node
    node1_tree.stash("user1", &User {
        id: "user1".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@company.com".to_string(),
        department: "Engineering".to_string(),
    })?;
    
    node2_tree.stash("user2", &User {
        id: "user2".to_string(),
        name: "Bob Smith".to_string(),
        email: "bob@company.com".to_string(),
        department: "Marketing".to_string(),
    })?;
    
    node3_tree.stash("user3", &User {
        id: "user3".to_string(),
        name: "Charlie Brown".to_string(),
        email: "charlie@company.com".to_string(),
        department: "Sales".to_string(),
    })?;
    
    println!("  Added users to each node");
    
    // Synchronize the entire mesh
    mesh.synchronize_all()?;
    println!("  Synchronized mesh");
    
    // Verify data propagation - each node should now have all users
    let users_node1: Vec<User> = node1_tree.query().collect()?;
    let users_node2: Vec<User> = node2_tree.query().collect()?;
    let users_node3: Vec<User> = node3_tree.query().collect()?;
    
    println!("  Node1 has {} users", users_node1.len());
    println!("  Node2 has {} users", users_node2.len());
    println!("  Node3 has {} users", users_node3.len());
    
    // Verify specific data
    let alice_node2: User = node2_tree.crack("user1")?;
    let bob_node3: User = node3_tree.crack("user2")?;
    
    assert_eq!(alice_node2.name, "Alice Johnson");
    assert_eq!(bob_node3.department, "Marketing");
    
    println!("  ✅ Full mesh sync verified - all data synchronized");
    Ok(())
}

fn mesh_star_example() -> Result<(), Error> {
    // Create a mesh coordinator
    let mesh = AcornTree::create_mesh()?;
    
    // Create trees for hub and spoke nodes
    let mut hub_tree = AcornTree::open("memory://")?;
    let mut spoke1_tree = AcornTree::open("memory://")?;
    let mut spoke2_tree = AcornTree::open("memory://")?;
    let mut spoke3_tree = AcornTree::open("memory://")?;
    
    // Add nodes to the mesh
    mesh.add_node("hub", &hub_tree)?;
    mesh.add_node("spoke1", &spoke1_tree)?;
    mesh.add_node("spoke2", &spoke2_tree)?;
    mesh.add_node("spoke3", &spoke3_tree)?;
    
    println!("  Added hub and 3 spoke nodes to mesh");
    
    // Create star topology with hub as center
    mesh.create_star("hub")?;
    println!("  Created star topology");
    
    // Add documents to hub
    hub_tree.stash("doc1", &Document {
        id: "doc1".to_string(),
        title: "Company Policy".to_string(),
        content: "This is the company policy document...".to_string(),
        author: "HR Department".to_string(),
        version: 1,
    })?;
    
    hub_tree.stash("doc2", &Document {
        id: "doc2".to_string(),
        title: "Technical Guidelines".to_string(),
        content: "Technical guidelines for development...".to_string(),
        author: "Engineering Team".to_string(),
        version: 2,
    })?;
    
    println!("  Added documents to hub");
    
    // Add local documents to spokes
    spoke1_tree.stash("local1", &Document {
        id: "local1".to_string(),
        title: "Spoke1 Local Doc".to_string(),
        content: "Local document from spoke1...".to_string(),
        author: "Spoke1 User".to_string(),
        version: 1,
    })?;
    
    println!("  Added local document to spoke1");
    
    // Synchronize the mesh
    mesh.synchronize_all()?;
    println!("  Synchronized mesh");
    
    // Verify hub documents propagated to spokes
    let doc1_spoke1: Document = spoke1_tree.crack("doc1")?;
    let doc2_spoke2: Document = spoke2_tree.crack("doc2")?;
    
    assert_eq!(doc1_spoke1.title, "Company Policy");
    assert_eq!(doc2_spoke2.author, "Engineering Team");
    
    // Verify spoke documents propagated to hub
    let local1_hub: Document = hub_tree.crack("local1")?;
    assert_eq!(local1_hub.title, "Spoke1 Local Doc");
    
    println!("  ✅ Star topology sync verified - hub-spoke data exchange working");
    Ok(())
}

fn mesh_ring_example() -> Result<(), Error> {
    // Create a mesh coordinator
    let mesh = AcornTree::create_mesh()?;
    
    // Create trees for ring nodes
    let mut node1_tree = AcornTree::open("memory://")?;
    let mut node2_tree = AcornTree::open("memory://")?;
    let mut node3_tree = AcornTree::open("memory://")?;
    let mut node4_tree = AcornTree::open("memory://")?;
    
    // Add nodes to the mesh
    mesh.add_node("node1", &node1_tree)?;
    mesh.add_node("node2", &node2_tree)?;
    mesh.add_node("node3", &node3_tree)?;
    mesh.add_node("node4", &node4_tree)?;
    
    println!("  Added 4 nodes to mesh");
    
    // Create ring topology
    mesh.create_ring()?;
    println!("  Created ring topology");
    
    // Add data to different nodes
    node1_tree.stash("message1", &serde_json::json!({
        "from": "node1",
        "content": "Hello from node1",
        "timestamp": "2024-01-01T10:00:00Z"
    }))?;
    
    node3_tree.stash("message2", &serde_json::json!({
        "from": "node3",
        "content": "Hello from node3",
        "timestamp": "2024-01-01T10:01:00Z"
    }))?;
    
    println!("  Added messages to nodes");
    
    // Synchronize the mesh
    mesh.synchronize_all()?;
    println!("  Synchronized mesh");
    
    // Verify messages propagated through the ring
    let message1_node2: serde_json::Value = node2_tree.crack("message1")?;
    let message2_node4: serde_json::Value = node4_tree.crack("message2")?;
    
    assert_eq!(message1_node2["from"], "node1");
    assert_eq!(message2_node4["from"], "node3");
    
    println!("  ✅ Ring topology sync verified - messages propagated through ring");
    Ok(())
}

fn p2p_bidirectional_example() -> Result<(), Error> {
    // Create two trees for P2P sync
    let mut local_tree = AcornTree::open("memory://")?;
    let mut remote_tree = AcornTree::open("memory://")?;
    
    // Create P2P connection
    let p2p = AcornTree::create_p2p(&local_tree, &remote_tree)?;
    println!("  Created P2P connection");
    
    // Add data to local tree
    local_tree.stash("user1", &User {
        id: "user1".to_string(),
        name: "Local User".to_string(),
        email: "local@example.com".to_string(),
        department: "Local Dept".to_string(),
    })?;
    
    println!("  Added user to local tree");
    
    // Sync bidirectionally
    p2p.sync_bidirectional()?;
    println!("  Performed bidirectional sync");
    
    // Verify data synchronized to remote
    let user1_remote: User = remote_tree.crack("user1")?;
    assert_eq!(user1_remote.name, "Local User");
    
    // Add data to remote tree
    remote_tree.stash("user2", &User {
        id: "user2".to_string(),
        name: "Remote User".to_string(),
        email: "remote@example.com".to_string(),
        department: "Remote Dept".to_string(),
    })?;
    
    println!("  Added user to remote tree");
    
    // Sync again
    p2p.sync_bidirectional()?;
    println!("  Performed second bidirectional sync");
    
    // Verify data synchronized to local
    let user2_local: User = local_tree.crack("user2")?;
    assert_eq!(user2_local.name, "Remote User");
    
    println!("  ✅ Bidirectional P2P sync verified - data flows both ways");
    Ok(())
}

fn p2p_push_only_example() -> Result<(), Error> {
    // Create two trees for P2P sync
    let mut source_tree = AcornTree::open("memory://")?;
    let mut target_tree = AcornTree::open("memory://")?;
    
    // Create P2P connection
    let p2p = AcornTree::create_p2p(&source_tree, &target_tree)?;
    
    // Set to push-only mode
    p2p.set_sync_mode(1)?; // PushOnly
    println!("  Set P2P to push-only mode");
    
    // Add data to source tree
    source_tree.stash("doc1", &Document {
        id: "doc1".to_string(),
        title: "Source Document".to_string(),
        content: "This document is pushed from source...".to_string(),
        author: "Source Author".to_string(),
        version: 1,
    })?;
    
    println!("  Added document to source tree");
    
    // Sync push-only
    p2p.sync_push_only()?;
    println!("  Performed push-only sync");
    
    // Verify data synchronized to target
    let doc1_target: Document = target_tree.crack("doc1")?;
    assert_eq!(doc1_target.title, "Source Document");
    
    // Add data to target tree
    target_tree.stash("doc2", &Document {
        id: "doc2".to_string(),
        title: "Target Document".to_string(),
        content: "This document should not be synced back...".to_string(),
        author: "Target Author".to_string(),
        version: 1,
    })?;
    
    println!("  Added document to target tree");
    
    // Sync push-only again
    p2p.sync_push_only()?;
    println!("  Performed second push-only sync");
    
    // Verify target document did NOT sync back to source
    assert!(source_tree.crack::<Document>("doc2").is_err());
    
    println!("  ✅ Push-only P2P sync verified - data flows only from source to target");
    Ok(())
}

fn p2p_pull_only_example() -> Result<(), Error> {
    // Create two trees for P2P sync
    let mut local_tree = AcornTree::open("memory://")?;
    let mut remote_tree = AcornTree::open("memory://")?;
    
    // Create P2P connection
    let p2p = AcornTree::create_p2p(&local_tree, &remote_tree)?;
    
    // Set to pull-only mode
    p2p.set_sync_mode(2)?; // PullOnly
    println!("  Set P2P to pull-only mode");
    
    // Add data to remote tree
    remote_tree.stash("config1", &serde_json::json!({
        "setting": "database_url",
        "value": "postgresql://remote-db:5432/app",
        "environment": "production"
    }))?;
    
    remote_tree.stash("config2", &serde_json::json!({
        "setting": "api_key",
        "value": "remote-api-key-123",
        "environment": "production"
    }))?;
    
    println!("  Added configuration to remote tree");
    
    // Sync pull-only
    p2p.sync_pull_only()?;
    println!("  Performed pull-only sync");
    
    // Verify data synchronized to local
    let config1_local: serde_json::Value = local_tree.crack("config1")?;
    let config2_local: serde_json::Value = local_tree.crack("config2")?;
    
    assert_eq!(config1_local["setting"], "database_url");
    assert_eq!(config2_local["value"], "remote-api-key-123");
    
    // Add data to local tree
    local_tree.stash("local_config", &serde_json::json!({
        "setting": "local_setting",
        "value": "local-value",
        "environment": "local"
    }))?;
    
    println!("  Added local configuration");
    
    // Sync pull-only again
    p2p.sync_pull_only()?;
    println!("  Performed second pull-only sync");
    
    // Verify local configuration did NOT sync to remote
    assert!(remote_tree.crack::<serde_json::Value>("local_config").is_err());
    
    println!("  ✅ Pull-only P2P sync verified - data flows only from remote to local");
    Ok(())
}

fn complex_mesh_example() -> Result<(), Error> {
    // Create a complex mesh with mixed data types and topologies
    let mesh = AcornTree::create_mesh()?;
    
    // Create trees for different departments
    let mut hr_tree = AcornTree::open("memory://")?;
    let mut eng_tree = AcornTree::open("memory://")?;
    let mut sales_tree = AcornTree::open("memory://")?;
    let mut marketing_tree = AcornTree::open("memory://")?;
    
    // Add nodes to mesh
    mesh.add_node("hr", &hr_tree)?;
    mesh.add_node("engineering", &eng_tree)?;
    mesh.add_node("sales", &sales_tree)?;
    mesh.add_node("marketing", &marketing_tree)?;
    
    println!("  Added 4 department nodes to mesh");
    
    // Create custom connections (not full mesh)
    mesh.connect_nodes("hr", "engineering")?;
    mesh.connect_nodes("engineering", "sales")?;
    mesh.connect_nodes("sales", "marketing")?;
    mesh.connect_nodes("marketing", "hr")?;
    
    println!("  Created custom mesh connections");
    
    // Add different types of data to each department
    hr_tree.stash("policy1", &Document {
        id: "policy1".to_string(),
        title: "Employee Handbook".to_string(),
        content: "Company policies and procedures...".to_string(),
        author: "HR Team".to_string(),
        version: 3,
    })?;
    
    eng_tree.stash("user1", &User {
        id: "user1".to_string(),
        name: "Alice Engineer".to_string(),
        email: "alice@company.com".to_string(),
        department: "Engineering".to_string(),
    })?;
    
    eng_tree.stash("tech_doc", &Document {
        id: "tech_doc".to_string(),
        title: "API Documentation".to_string(),
        content: "Technical API documentation...".to_string(),
        author: "Engineering Team".to_string(),
        version: 2,
    })?;
    
    sales_tree.stash("user2", &User {
        id: "user2".to_string(),
        name: "Bob Sales".to_string(),
        email: "bob@company.com".to_string(),
        department: "Sales".to_string(),
    })?;
    
    marketing_tree.stash("campaign1", &serde_json::json!({
        "id": "campaign1",
        "name": "Q1 Product Launch",
        "budget": 50000,
        "status": "active",
        "target_audience": "enterprise"
    }))?;
    
    println!("  Added mixed data types to each department");
    
    // Synchronize the mesh
    mesh.synchronize_all()?;
    println!("  Synchronized complex mesh");
    
    // Verify data propagation through the custom mesh
    let policy1_eng: Document = eng_tree.crack("policy1")?;
    let user1_sales: User = sales_tree.crack("user1")?;
    let tech_doc_marketing: Document = marketing_tree.crack("tech_doc")?;
    let campaign1_hr: serde_json::Value = hr_tree.crack("campaign1")?;
    
    assert_eq!(policy1_eng.title, "Employee Handbook");
    assert_eq!(user1_sales.name, "Alice Engineer");
    assert_eq!(tech_doc_marketing.author, "Engineering Team");
    assert_eq!(campaign1_hr["name"], "Q1 Product Launch");
    
    println!("  ✅ Complex mesh sync verified - mixed data types propagated through custom topology");
    Ok(())
}
