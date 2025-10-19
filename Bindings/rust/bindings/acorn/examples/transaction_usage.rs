use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    name: String,
    age: u32,
    email: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Account {
    user_id: String,
    balance: f64,
    currency: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AcornDB Transaction Usage Example ===\n");

    // Open a tree
    let mut tree = AcornTree::open("memory://")?;
    println!("✓ Opened AcornDB tree");

    // Example 1: Basic transaction with commit
    println!("\n--- Example 1: Basic Transaction with Commit ---");
    let mut tx = tree.begin_transaction()?;
    println!("✓ Started transaction");

    // Add some users
    tx.stash("user1", &User {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
    })?;
    tx.stash("user2", &User {
        name: "Bob".to_string(),
        age: 25,
        email: "bob@example.com".to_string(),
    })?;
    println!("✓ Added users to transaction");

    // Add some accounts
    tx.stash("account1", &Account {
        user_id: "user1".to_string(),
        balance: 1000.0,
        currency: "USD".to_string(),
    })?;
    tx.stash("account2", &Account {
        user_id: "user2".to_string(),
        balance: 500.0,
        currency: "USD".to_string(),
    })?;
    println!("✓ Added accounts to transaction");

    // Commit the transaction
    if tx.commit()? {
        println!("✓ Transaction committed successfully");
    } else {
        println!("✗ Transaction failed to commit");
    }

    // Verify the data was committed
    let user1: User = tree.crack("user1")?;
    let account1: Account = tree.crack("account1")?;
    println!("✓ Verified committed data: {} has ${}", user1.name, account1.balance);

    // Example 2: Transaction with rollback
    println!("\n--- Example 2: Transaction with Rollback ---");
    let mut tx2 = tree.begin_transaction()?;
    println!("✓ Started second transaction");

    // Add a user that we'll rollback
    tx2.stash("user3", &User {
        name: "Charlie".to_string(),
        age: 35,
        email: "charlie@example.com".to_string(),
    })?;
    println!("✓ Added user3 to transaction");

    // Rollback the transaction
    tx2.rollback()?;
    println!("✓ Transaction rolled back");

    // Verify user3 doesn't exist
    match tree.crack::<User>("user3") {
        Ok(_) => println!("✗ user3 should not exist after rollback"),
        Err(Error::NotFound) => println!("✓ user3 correctly does not exist after rollback"),
        Err(e) => return Err(e.into()),
    }

    // Example 3: Complex transaction with conditional logic
    println!("\n--- Example 3: Complex Transaction with Conditional Logic ---");
    let mut tx3 = tree.begin_transaction()?;
    println!("✓ Started third transaction");

    // Transfer money between accounts
    let mut account1: Account = tree.crack("account1")?;
    let mut account2: Account = tree.crack("account2")?;
    
    let transfer_amount = 200.0;
    if account1.balance >= transfer_amount {
        account1.balance -= transfer_amount;
        account2.balance += transfer_amount;
        
        tx3.stash("account1", &account1)?;
        tx3.stash("account2", &account2)?;
        println!("✓ Prepared transfer of ${} from {} to {}", 
                transfer_amount, account1.user_id, account2.user_id);
        
        if tx3.commit()? {
            println!("✓ Transfer transaction committed successfully");
        } else {
            println!("✗ Transfer transaction failed to commit");
        }
    } else {
        tx3.rollback()?;
        println!("✗ Insufficient funds, transaction rolled back");
    }

    // Verify the transfer
    let final_account1: Account = tree.crack("account1")?;
    let final_account2: Account = tree.crack("account2")?;
    println!("✓ Final balances - Account1: ${}, Account2: ${}", 
             final_account1.balance, final_account2.balance);

    // Example 4: Batch operations in transaction
    println!("\n--- Example 4: Batch Operations in Transaction ---");
    let mut tx4 = tree.begin_transaction()?;
    println!("✓ Started fourth transaction");

    // Add multiple users at once
    let users = vec![
        ("user4", User { name: "David".to_string(), age: 28, email: "david@example.com".to_string() }),
        ("user5", User { name: "Eve".to_string(), age: 32, email: "eve@example.com".to_string() }),
        ("user6", User { name: "Frank".to_string(), age: 27, email: "frank@example.com".to_string() }),
    ];

    for (id, user) in &users {
        tx4.stash(id, user)?;
    }
    println!("✓ Added {} users to transaction", users.len());

    // Delete one user
    tx4.delete("user2")?;
    println!("✓ Deleted user2 in transaction");

    if tx4.commit()? {
        println!("✓ Batch transaction committed successfully");
    } else {
        println!("✗ Batch transaction failed to commit");
    }

    // Verify the batch operations
    let user4: User = tree.crack("user4")?;
    println!("✓ Verified user4 exists: {}", user4.name);
    
    match tree.crack::<User>("user2") {
        Ok(_) => println!("✗ user2 should not exist after deletion"),
        Err(Error::NotFound) => println!("✓ user2 correctly deleted"),
        Err(e) => return Err(e.into()),
    }

    // Example 5: Error handling in transactions
    println!("\n--- Example 5: Error Handling in Transactions ---");
    let mut tx5 = tree.begin_transaction()?;
    println!("✓ Started fifth transaction");

    // This will succeed
    tx5.stash("user7", &User {
        name: "Grace".to_string(),
        age: 29,
        email: "grace@example.com".to_string(),
    })?;
    println!("✓ Added user7 to transaction");

    // This will fail due to invalid JSON (empty string as ID)
    match tx5.stash("", &User {
        name: "Invalid".to_string(),
        age: 0,
        email: "invalid@example.com".to_string(),
    }) {
        Ok(_) => println!("✗ Empty ID should have failed"),
        Err(Error::Acorn(_)) => println!("✓ Empty ID correctly failed"),
        Err(e) => return Err(e.into()),
    }

    // Rollback due to error
    tx5.rollback()?;
    println!("✓ Transaction rolled back due to error");

    // Verify user7 doesn't exist
    match tree.crack::<User>("user7") {
        Ok(_) => println!("✗ user7 should not exist after rollback"),
        Err(Error::NotFound) => println!("✓ user7 correctly does not exist after rollback"),
        Err(e) => return Err(e.into()),
    }

    println!("\n=== Transaction Examples Complete ===");
    println!("✓ All transaction operations demonstrated successfully!");

    Ok(())
}
