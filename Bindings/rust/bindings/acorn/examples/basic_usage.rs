use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Person {
    id: String,
    name: String,
    age: u32,
    email: String,
}

fn main() -> Result<(), Error> {
    println!("AcornDB Rust Bindings Example");
    
    // Open a tree with memory storage (avoids file deserialization issues)
    let mut tree = AcornTree::open("memory://")?;
    println!("✓ Opened database");
    
    // Create some sample data
    let alice = Person {
        id: "alice".to_string(),
        name: "Alice Johnson".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    };
    
    let bob = Person {
        id: "bob".to_string(),
        name: "Bob Smith".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
    };
    
    // Store the data
    tree.stash("alice", &alice)?;
    tree.stash("bob", &bob)?;
    println!("✓ Stored 2 people");
    
    // Retrieve Alice
    let retrieved_alice: Person = tree.crack("alice")?;
    println!("✓ Retrieved Alice: {:?}", retrieved_alice);
    
    // Verify the data matches
    assert_eq!(alice, retrieved_alice);
    println!("✓ Data integrity verified");
    
    // Try to retrieve non-existent person
    match tree.crack::<Person>("charlie") {
        Err(Error::NotFound) => println!("✓ Correctly handled not found case"),
        Ok(_) => println!("❌ Unexpected success"),
        Err(e) => println!("❌ Unexpected error: {}", e),
    }
    
    println!("🎉 Example completed successfully!");
    Ok(())
}
