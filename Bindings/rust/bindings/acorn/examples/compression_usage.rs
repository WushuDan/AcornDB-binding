use acorn::{AcornCompression, AcornTree, CompressionLevel, Error};

fn main() -> Result<(), Error> {
    println!("🔧 AcornDB Compression Example");
    println!("================================");

    // Example 1: Gzip compression
    println!("\n1. Gzip Compression");
    let gzip_compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    println!("✅ Created Gzip compression provider");
    
    // Test compression
    let original_text = "Hello, world! This is a test of compression capabilities in AcornDB.";
    let compressed = gzip_compression.compress(original_text)?;
    println!("📦 Compressed data: {}", compressed);
    
    // Test decompression
    let decompressed = gzip_compression.decompress(&compressed)?;
    println!("📤 Decompressed data: {}", decompressed);
    assert_eq!(original_text, decompressed);
    
    // Get compression stats
    let stats = gzip_compression.get_stats(original_text, &compressed)?;
    println!("📊 Compression Stats:");
    println!("   Original size: {} bytes", stats.original_size);
    println!("   Compressed size: {} bytes", stats.compressed_size);
    println!("   Compression ratio: {:.2}", stats.ratio);
    println!("   Space saved: {} bytes", stats.space_saved);
    
    // Check algorithm name
    let algorithm = gzip_compression.algorithm_name()?;
    println!("🔧 Algorithm: {}", algorithm);
    assert_eq!(algorithm, "Gzip");
    
    // Check if enabled
    let is_enabled = gzip_compression.is_enabled()?;
    println!("✅ Compression enabled: {}", is_enabled);
    assert!(is_enabled);

    // Example 2: Brotli compression
    println!("\n2. Brotli Compression");
    let brotli_compression = AcornCompression::brotli(CompressionLevel::SmallestSize)?;
    println!("✅ Created Brotli compression provider");
    
    // Test compression with larger text
    let large_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(10);
    let brotli_compressed = brotli_compression.compress(&large_text)?;
    println!("📦 Brotli compressed data: {}", brotli_compressed);
    
    // Test decompression
    let brotli_decompressed = brotli_compression.decompress(&brotli_compressed)?;
    assert_eq!(large_text, brotli_decompressed);
    
    // Get compression stats
    let brotli_stats = brotli_compression.get_stats(&large_text, &brotli_compressed)?;
    println!("📊 Brotli Compression Stats:");
    println!("   Original size: {} bytes", brotli_stats.original_size);
    println!("   Compressed size: {} bytes", brotli_stats.compressed_size);
    println!("   Compression ratio: {:.2}", brotli_stats.ratio);
    println!("   Space saved: {} bytes", brotli_stats.space_saved);
    
    // Check algorithm name
    let brotli_algorithm = brotli_compression.algorithm_name()?;
    println!("🔧 Algorithm: {}", brotli_algorithm);
    assert_eq!(brotli_algorithm, "Brotli");

    // Example 3: No compression
    println!("\n3. No Compression");
    let no_compression = AcornCompression::none()?;
    println!("✅ Created no compression provider");
    
    // Test with no compression
    let no_compressed = no_compression.compress(original_text)?;
    let no_decompressed = no_compression.decompress(&no_compressed)?;
    assert_eq!(original_text, no_decompressed);
    
    // Check if enabled
    let no_enabled = no_compression.is_enabled()?;
    println!("✅ Compression enabled: {}", no_enabled);
    assert!(!no_enabled);
    
    // Check algorithm name
    let no_algorithm = no_compression.algorithm_name()?;
    println!("🔧 Algorithm: {}", no_algorithm);
    assert_eq!(no_algorithm, "None");

    // Example 4: Compression levels comparison
    println!("\n4. Compression Levels Comparison");
    let fastest = AcornCompression::gzip(CompressionLevel::Fastest)?;
    let optimal = AcornCompression::gzip(CompressionLevel::Optimal)?;
    let smallest = AcornCompression::gzip(CompressionLevel::SmallestSize)?;
    
    let test_data = "This is a test of different compression levels in AcornDB. ".repeat(5);
    
    let fastest_compressed = fastest.compress(&test_data)?;
    let optimal_compressed = optimal.compress(&test_data)?;
    let smallest_compressed = smallest.compress(&test_data)?;
    
    let fastest_stats = fastest.get_stats(&test_data, &fastest_compressed)?;
    let optimal_stats = optimal.get_stats(&test_data, &optimal_compressed)?;
    let smallest_stats = smallest.get_stats(&test_data, &smallest_compressed)?;
    
    println!("📊 Compression Level Comparison:");
    println!("   Fastest - Ratio: {:.2}, Size: {} bytes", fastest_stats.ratio, fastest_stats.compressed_size);
    println!("   Optimal - Ratio: {:.2}, Size: {} bytes", optimal_stats.ratio, optimal_stats.compressed_size);
    println!("   Smallest - Ratio: {:.2}, Size: {} bytes", smallest_stats.ratio, smallest_stats.compressed_size);

    // Example 5: Compressed tree storage
    println!("\n5. Compressed Tree Storage");
    let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    let mut tree = AcornTree::open_compressed("memory://compressed_db", &compression)?;
    println!("✅ Opened compressed tree");
    
    // Store some data
    let data = serde_json::json!({
        "name": "John Doe",
        "age": 30,
        "email": "john.doe@example.com",
        "address": {
            "street": "123 Main St",
            "city": "Anytown",
            "state": "CA",
            "zip": "12345"
        },
        "hobbies": ["reading", "hiking", "cooking"],
        "description": "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(10)
    });
    
    tree.stash("user:1", &data)?;
    println!("📦 Stored user data with compression");
    
    // Retrieve data
    let retrieved: serde_json::Value = tree.crack("user:1")?;
    println!("📤 Retrieved user data");
    assert_eq!(data, retrieved);
    
    // Store more data to show compression benefits
    for i in 2..=5 {
        let user_data = serde_json::json!({
            "id": i,
            "name": format!("User {}", i),
            "description": "This is a long description that will benefit from compression. ".repeat(20),
            "data": vec![i; 100] // Array of 100 integers
        });
        tree.stash(&format!("user:{}", i), &user_data)?;
    }
    println!("📦 Stored additional user data");
    
    // List all users
    let users = tree.list()?;
    println!("👥 Total users stored: {}", users.len());
    assert_eq!(users.len(), 5);

    // Example 6: Error handling
    println!("\n6. Error Handling");
    let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    
    // Try to decompress invalid data
    match compression.decompress("invalid-base64-data") {
        Ok(_) => println!("❌ Unexpected success"),
        Err(e) => println!("✅ Expected error: {}", e),
    }
    
    // Try to get stats with mismatched data
    match compression.get_stats("original", "different-compressed") {
        Ok(_) => println!("❌ Unexpected success"),
        Err(e) => println!("✅ Expected error: {}", e),
    }

    println!("\n🎉 All compression examples completed successfully!");
    Ok(())
}
