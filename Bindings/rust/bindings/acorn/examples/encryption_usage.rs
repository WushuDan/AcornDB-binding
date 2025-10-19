use acorn::{AcornTree, AcornEncryption, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
    password_hash: String,
}

fn main() -> Result<(), Error> {
    println!("🔐 AcornDB Encryption Example");
    println!("=============================");

    // Example 1: Password-based encryption
    println!("\n1. Password-based encryption:");
    let encryption = AcornEncryption::from_password("my-secret-password", "my-unique-salt")?;
    println!("✓ Created encryption provider from password");

    // Open an encrypted tree
    let mut encrypted_tree = AcornTree::open_encrypted("file://./encrypted_db", &encryption)?;
    println!("✓ Opened encrypted tree");

    // Store sensitive data
    let user = User {
        id: "user-1".to_string(),
        name: "Alice Johnson".to_string(),
        email: "alice@example.com".to_string(),
        password_hash: "hashed_password_123".to_string(),
    };

    encrypted_tree.stash("user-1", &user)?;
    println!("✓ Stored encrypted user data");

    // Retrieve and verify data
    let retrieved_user: User = encrypted_tree.crack("user-1")?;
    assert_eq!(user, retrieved_user);
    println!("✓ Retrieved and verified encrypted data: {:?}", retrieved_user);

    // Example 2: Key/IV-based encryption
    println!("\n2. Key/IV-based encryption:");
    let (key, iv) = AcornEncryption::generate_key_iv()?;
    println!("✓ Generated random key and IV");
    println!("  Key: {}", key);
    println!("  IV:  {}", iv);

    let key_encryption = AcornEncryption::from_key_iv(&key, &iv)?;
    println!("✓ Created encryption provider from key/IV");

    // Export key/IV for backup
    let exported_key = key_encryption.export_key()?;
    let exported_iv = key_encryption.export_iv()?;
    println!("✓ Exported key/IV for backup");
    println!("  Exported Key: {}", exported_key);
    println!("  Exported IV:  {}", exported_iv);

    // Example 3: Direct encryption/decryption
    println!("\n3. Direct encryption/decryption:");
    let plaintext = "This is sensitive data that needs to be encrypted!";
    let ciphertext = key_encryption.encrypt(plaintext)?;
    println!("✓ Encrypted plaintext");
    println!("  Plaintext:  {}", plaintext);
    println!("  Ciphertext: {}", ciphertext);

    let decrypted = key_encryption.decrypt(&ciphertext)?;
    println!("✓ Decrypted ciphertext");
    println!("  Decrypted: {}", decrypted);
    assert_eq!(plaintext, decrypted);

    // Example 4: Encrypted + Compressed tree
    println!("\n4. Encrypted + Compressed tree:");
    let mut compressed_tree = AcornTree::open_encrypted_compressed("file://./compressed_encrypted_db", &encryption, 1)?;
    println!("✓ Opened encrypted + compressed tree (Optimal compression)");

    // Store large data to demonstrate compression
    let large_data = User {
        id: "user-2".to_string(),
        name: "Bob Smith".to_string(),
        email: "bob@example.com".to_string(),
        password_hash: "very_long_hashed_password_with_lots_of_characters_to_demonstrate_compression".to_string(),
    };

    compressed_tree.stash("user-2", &large_data)?;
    println!("✓ Stored large data in encrypted + compressed tree");

    let retrieved_large: User = compressed_tree.crack("user-2")?;
    assert_eq!(large_data, retrieved_large);
    println!("✓ Retrieved and verified compressed encrypted data");

    // Example 5: Encryption status check
    println!("\n5. Encryption status:");
    let is_enabled = encryption.is_enabled()?;
    println!("✓ Encryption enabled: {}", is_enabled);

    // Example 6: Memory-based encrypted tree
    println!("\n6. Memory-based encrypted tree:");
    let mut memory_tree = AcornTree::open_encrypted("memory://", &encryption)?;
    println!("✓ Opened encrypted memory tree");

    let temp_user = User {
        id: "temp-1".to_string(),
        name: "Temporary User".to_string(),
        email: "temp@example.com".to_string(),
        password_hash: "temp_hash".to_string(),
    };

    memory_tree.stash("temp-1", &temp_user)?;
    let retrieved_temp: User = memory_tree.crack("temp-1")?;
    assert_eq!(temp_user, retrieved_temp);
    println!("✓ Stored and retrieved from encrypted memory tree");

    println!("\n🎉 All encryption examples completed successfully!");
    println!("\nSecurity Notes:");
    println!("- Store encryption keys securely (not in code!)");
    println!("- Use unique salts for each database");
    println!("- Consider using key management services for production");
    println!("- Encryption adds overhead but provides data protection");

    Ok(())
}
