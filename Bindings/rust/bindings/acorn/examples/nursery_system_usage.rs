use acorn::{AcornNursery, AcornTree, AcornStorage, Error};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Product {
    id: String,
    name: String,
    price: f64,
    category: String,
    in_stock: bool,
    created_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Order {
    id: String,
    customer_id: String,
    product_ids: Vec<String>,
    total_amount: f64,
    status: String,
    created_at: SystemTime,
}

fn main() -> Result<(), Error> {
    println!("🌱 AcornDB Nursery System Example");
    println!("=================================");

    // Example 1: Discover available trunk types
    println!("=== Example 1: Discover Available Trunk Types ===");

    let nursery = AcornNursery::new()?;
    println!("✅ Created Nursery instance");

    let available_types = nursery.get_available_types()?;
    println!("📦 Available trunk types:");
    for trunk_type in &available_types {
        println!("   - {}", trunk_type);
    }
    println!();

    // Example 2: Get detailed metadata for trunk types
    println!("=== Example 2: Trunk Metadata Discovery ===");

    for trunk_type in &available_types {
        if let Ok(metadata) = nursery.get_metadata(trunk_type) {
            println!("📋 {} Trunk:", metadata.display_name);
            println!("   Description: {}", metadata.description);
            println!("   Category: {}", metadata.category);
            println!("   Durable: {}, History: {}, Sync: {}, Async: {}", 
                metadata.is_durable, metadata.supports_history, 
                metadata.supports_sync, metadata.supports_async);
            println!("   Required config: {:?}", metadata.required_config_keys);
            println!("   Optional config: {:?}", metadata.optional_config_keys);
            println!("   Built-in: {}", metadata.is_built_in);
            println!();
        }
    }

    // Example 3: Get formatted catalog
    println!("=== Example 3: Nursery Catalog ===");

    let catalog = nursery.get_catalog()?;
    println!("📚 Nursery Catalog:");
    println!("{}", catalog);
    println!();

    // Example 4: Validate configurations before creating trunks
    println!("=== Example 4: Configuration Validation ===");

    let configs = vec![
        ("file", r#"{"path": "./products"}"#),
        ("memory", r#"{}"#),
        ("git", r#"{"repo_path": "./git-products", "author_name": "AcornDB", "author_email": "acorn@acorndb.dev", "auto_push": false}"#),
        ("invalid", r#"{"invalid": "config"}"#),
    ];

    for (trunk_type, config) in &configs {
        let is_valid = nursery.validate_config(trunk_type, config)?;
        println!("✅ {} trunk config validation: {}", trunk_type, if is_valid { "VALID" } else { "INVALID" });
    }
    println!();

    // Example 5: Grow trunks dynamically from configuration
    println!("=== Example 5: Dynamic Trunk Creation ===");

    // Create different storage backends dynamically
    let mut trees = Vec::new();

    // File storage
    if nursery.has_trunk("file")? {
        let file_config = r#"{"path": "./nursery-products"}"#;
        if nursery.validate_config("file", file_config)? {
            let file_storage = nursery.grow_trunk("file", file_config)?;
            let mut file_tree = AcornTree::open_with_storage(file_storage)?;
            trees.push(("file", file_tree));
            println!("🌳 Grew file trunk successfully");
        }
    }

    // Memory storage
    if nursery.has_trunk("memory")? {
        let memory_config = r#"{}"#;
        if nursery.validate_config("memory", memory_config)? {
            let memory_storage = nursery.grow_trunk("memory", memory_config)?;
            let mut memory_tree = AcornTree::open_with_storage(memory_storage)?;
            trees.push(("memory", memory_tree));
            println!("🌳 Grew memory trunk successfully");
        }
    }

    // Git storage (if available)
    if nursery.has_trunk("git")? {
        let git_config = r#"{"repo_path": "./nursery-git-products", "author_name": "AcornDB", "author_email": "acorn@acorndb.dev", "auto_push": false}"#;
        if nursery.validate_config("git", git_config)? {
            let git_storage = nursery.grow_trunk("git", git_config)?;
            let mut git_tree = AcornTree::open_with_storage(git_storage)?;
            trees.push(("git", git_tree));
            println!("🌳 Grew git trunk successfully");
        }
    }
    println!();

    // Example 6: Use dynamically created trees
    println!("=== Example 6: Using Dynamic Trees ===");

    let products = vec![
        Product {
            id: "prod-1".to_string(),
            name: "AcornDB Pro".to_string(),
            price: 99.99,
            category: "Software".to_string(),
            in_stock: true,
            created_at: SystemTime::now(),
        },
        Product {
            id: "prod-2".to_string(),
            name: "Rust Binding Kit".to_string(),
            price: 49.99,
            category: "Development".to_string(),
            in_stock: true,
            created_at: SystemTime::now(),
        },
        Product {
            id: "prod-3".to_string(),
            name: "Nursery System".to_string(),
            price: 29.99,
            category: "Tools".to_string(),
            in_stock: false,
            created_at: SystemTime::now(),
        },
    ];

    let orders = vec![
        Order {
            id: "order-1".to_string(),
            customer_id: "customer-1".to_string(),
            product_ids: vec!["prod-1".to_string(), "prod-2".to_string()],
            total_amount: 149.98,
            status: "completed".to_string(),
            created_at: SystemTime::now(),
        },
        Order {
            id: "order-2".to_string(),
            customer_id: "customer-2".to_string(),
            product_ids: vec!["prod-3".to_string()],
            total_amount: 29.99,
            status: "pending".to_string(),
            created_at: SystemTime::now(),
        },
    ];

    // Store products in all trees
    for (trunk_name, tree) in &mut trees {
        for product in &products {
            tree.stash(&product.id, product)?;
        }
        println!("📦 Stored {} products in {} trunk", products.len(), trunk_name);
    }
    println!();

    // Example 7: Query across different storage backends
    println!("=== Example 7: Cross-Storage Queries ===");

    for (trunk_name, tree) in &trees {
        // Query in-stock products
        let in_stock_products: Vec<Product> = tree.query()
            .where_condition(|item| item["in_stock"].as_bool() == Some(true))
            .collect()?;
        
        println!("🛒 {} trunk - In-stock products: {}", trunk_name, in_stock_products.len());
        for product in &in_stock_products {
            println!("   - {}: ${:.2}", product.name, product.price);
        }

        // Query by category
        let software_products: Vec<Product> = tree.query()
            .where_condition(|item| item["category"].as_str() == Some("Software"))
            .collect()?;
        
        println!("💻 {} trunk - Software products: {}", trunk_name, software_products.len());
        for product in &software_products {
            println!("   - {}: ${:.2}", product.name, product.price);
        }
        println!();
    }

    // Example 8: Configuration-driven tree creation
    println!("=== Example 8: Configuration-Driven Creation ===");

    // Simulate reading configuration from a file or environment
    let configs = vec![
        ("products", "file", r#"{"path": "./config-products"}"#),
        ("orders", "memory", r#"{}"#),
        ("analytics", "git", r#"{"repo_path": "./analytics-git", "author_name": "Analytics", "author_email": "analytics@acorndb.dev", "auto_push": false}"#),
    ];

    let mut config_trees = Vec::new();

    for (name, trunk_type, config) in &configs {
        if nursery.has_trunk(trunk_type)? && nursery.validate_config(trunk_type, config)? {
            let storage = nursery.grow_trunk(trunk_type, config)?;
            let mut tree = AcornTree::open_with_storage(storage)?;
            config_trees.push((name, tree));
            println!("🔧 Created {} tree with {} trunk", name, trunk_type);
        } else {
            println!("❌ Failed to create {} tree with {} trunk", name, trunk_type);
        }
    }
    println!();

    // Example 9: Store different data types in different trees
    println!("=== Example 9: Specialized Data Storage ===");

    // Store products in products tree
    if let Some((_, products_tree)) = config_trees.iter_mut().find(|(name, _)| *name == "products") {
        for product in &products {
            products_tree.stash(&product.id, product)?;
        }
        println!("📦 Stored {} products in products tree", products.len());
    }

    // Store orders in orders tree
    if let Some((_, orders_tree)) = config_trees.iter_mut().find(|(name, _)| *name == "orders") {
        for order in &orders {
            orders_tree.stash(&order.id, order)?;
        }
        println!("📋 Stored {} orders in orders tree", orders.len());
    }

    // Store analytics data in analytics tree (Git-backed for version history)
    if let Some((_, analytics_tree)) = config_trees.iter_mut().find(|(name, _)| *name == "analytics") {
        let analytics_data = serde_json::json!({
            "timestamp": SystemTime::now(),
            "total_products": products.len(),
            "total_orders": orders.len(),
            "total_revenue": orders.iter().map(|o| o.total_amount).sum::<f64>(),
            "categories": ["Software", "Development", "Tools"]
        });
        
        analytics_tree.stash("daily-stats", &analytics_data)?;
        println!("📊 Stored analytics data in analytics tree (Git-backed)");
    }
    println!();

    // Example 10: Performance comparison across storage types
    println!("=== Example 10: Performance Comparison ===");

    for (trunk_name, tree) in &trees {
        use std::time::Instant;
        
        // Test query performance
        let start = Instant::now();
        let all_products: Vec<Product> = tree.query().collect()?;
        let query_time = start.elapsed();
        
        // Test count performance
        let start = Instant::now();
        let count = tree.query().count()?;
        let count_time = start.elapsed();
        
        println!("⚡ {} trunk performance:", trunk_name);
        println!("   Query time: {:?} ({} items)", query_time, all_products.len());
        println!("   Count time: {:?} ({} items)", count_time, count);
    }
    println!();

    // Example 11: Error handling and validation
    println!("=== Example 11: Error Handling ===");

    // Test invalid trunk type
    match nursery.has_trunk("nonexistent") {
        Ok(has_trunk) => println!("❌ Unexpected: nonexistent trunk exists: {}", has_trunk),
        Err(e) => println!("✅ Expected error for nonexistent trunk: {}", e),
    }

    // Test invalid configuration
    match nursery.validate_config("file", r#"{"invalid": "config"}"#) {
        Ok(is_valid) => println!("❌ Unexpected: invalid config is valid: {}", is_valid),
        Err(e) => println!("✅ Expected error for invalid config: {}", e),
    }

    // Test growing invalid trunk
    match nursery.grow_trunk("nonexistent", r#"{}"#) {
        Ok(_) => println!("❌ Unexpected: successfully grew nonexistent trunk"),
        Err(e) => println!("✅ Expected error growing nonexistent trunk: {}", e),
    }
    println!();

    println!("🎉 Nursery System example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("✅ Dynamic trunk discovery: Browse available trunk types at runtime");
    println!("✅ Trunk metadata: Get detailed information about trunk capabilities");
    println!("✅ Configuration validation: Validate configs before creating trunks");
    println!("✅ Dynamic trunk creation: Create trunks by type ID from configuration");
    println!("✅ Cross-storage operations: Use multiple storage backends simultaneously");
    println!("✅ Configuration-driven setup: Create trees from configuration files");
    println!("✅ Specialized storage: Use different trunks for different data types");
    println!("✅ Performance comparison: Compare performance across storage types");
    println!("✅ Error handling: Proper error handling for invalid configurations");
    println!("✅ Nursery catalog: Formatted catalog of all available trunks");

    Ok(())
}
