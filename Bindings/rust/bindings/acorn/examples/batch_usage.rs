use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Product {
    name: String,
    price: f64,
    category: String,
    stock: i32,
}

fn main() -> Result<(), Error> {
    println!("AcornDB Batch Operations Example");
    println!();

    // Open a tree with memory storage
    let mut tree = AcornTree::open("memory://")?;
    println!("✓ Opened database");
    println!();

    // Example 1: Batch Stash
    println!("=== Batch Stash ===");
    let products = vec![
        ("product-001", Product {
            name: "Laptop".to_string(),
            price: 999.99,
            category: "Electronics".to_string(),
            stock: 15,
        }),
        ("product-002", Product {
            name: "Mouse".to_string(),
            price: 29.99,
            category: "Electronics".to_string(),
            stock: 50,
        }),
        ("product-003", Product {
            name: "Keyboard".to_string(),
            price: 79.99,
            category: "Electronics".to_string(),
            stock: 30,
        }),
        ("product-004", Product {
            name: "Monitor".to_string(),
            price: 299.99,
            category: "Electronics".to_string(),
            stock: 20,
        }),
        ("product-005", Product {
            name: "Webcam".to_string(),
            price: 89.99,
            category: "Electronics".to_string(),
            stock: 40,
        }),
    ];

    tree.batch_stash(&products)?;
    println!("✓ Batch stashed {} products", products.len());
    println!();

    // Example 2: Batch Crack
    println!("=== Batch Crack ===");
    let keys_to_retrieve = vec!["product-001", "product-003", "product-005", "product-999"];
    let results: Vec<Option<Product>> = tree.batch_crack(&keys_to_retrieve)?;

    println!("Retrieved {} items:", keys_to_retrieve.len());
    for (key, result) in keys_to_retrieve.iter().zip(results.iter()) {
        match result {
            Some(product) => println!("  ✓ {}: {} - ${:.2}", key, product.name, product.price),
            None => println!("  ✗ {}: not found", key),
        }
    }
    println!();

    // Example 3: Individual operations for comparison
    println!("=== Individual Stash (for comparison) ===");
    let start = std::time::Instant::now();
    for i in 100..110 {
        tree.stash(
            &format!("item-{}", i),
            &Product {
                name: format!("Item {}", i),
                price: i as f64,
                category: "Test".to_string(),
                stock: i,
            },
        )?;
    }
    let individual_duration = start.elapsed();
    println!("✓ Individual stash of 10 items took: {:?}", individual_duration);
    println!();

    // Example 4: Batch operations for performance
    println!("=== Batch Stash (for comparison) ===");
    let batch_items: Vec<(&str, Product)> = (200..210)
        .map(|i| {
            (
                Box::leak(format!("item-{}", i).into_boxed_str()) as &str,
                Product {
                    name: format!("Item {}", i),
                    price: i as f64,
                    category: "Test".to_string(),
                    stock: i,
                },
            )
        })
        .collect();

    let start = std::time::Instant::now();
    tree.batch_stash(&batch_items)?;
    let batch_duration = start.elapsed();
    println!("✓ Batch stash of 10 items took: {:?}", batch_duration);
    println!("  (Note: Performance benefits are more noticeable with larger batches and network storage)");
    println!();

    // Example 5: Batch Delete
    println!("=== Batch Delete ===");
    let keys_to_delete = vec!["product-002", "product-004"];
    tree.batch_delete(&keys_to_delete)?;
    println!("✓ Batch deleted {} items", keys_to_delete.len());

    // Verify deletion
    let verify_keys = vec!["product-001", "product-002", "product-003", "product-004", "product-005"];
    let verify_results: Vec<Option<Product>> = tree.batch_crack(&verify_keys)?;
    println!("\nVerifying deletions:");
    for (key, result) in verify_keys.iter().zip(verify_results.iter()) {
        match result {
            Some(_) => println!("  ✓ {}: still exists", key),
            None => println!("  ✗ {}: deleted", key),
        }
    }
    println!();

    // Example 6: Mixed operations
    println!("=== Mixed Batch Operations ===");

    // Store a batch
    let new_products = vec![
        ("new-001", Product {
            name: "USB Cable".to_string(),
            price: 9.99,
            category: "Accessories".to_string(),
            stock: 100,
        }),
        ("new-002", Product {
            name: "HDMI Cable".to_string(),
            price: 14.99,
            category: "Accessories".to_string(),
            stock: 75,
        }),
        ("new-003", Product {
            name: "Power Adapter".to_string(),
            price: 19.99,
            category: "Accessories".to_string(),
            stock: 60,
        }),
    ];
    tree.batch_stash(&new_products)?;
    println!("✓ Added {} new products", new_products.len());

    // Retrieve them
    let all_new_keys: Vec<&str> = new_products.iter().map(|(key, _)| *key).collect();
    let retrieved: Vec<Option<Product>> = tree.batch_crack(&all_new_keys)?;
    let found_count = retrieved.iter().filter(|r| r.is_some()).count();
    println!("✓ Retrieved {}/{} products", found_count, all_new_keys.len());

    // Delete them
    tree.batch_delete(&all_new_keys)?;
    println!("✓ Deleted {} products", all_new_keys.len());
    println!();

    println!("🎉 Batch operations example completed successfully!");

    Ok(())
}
