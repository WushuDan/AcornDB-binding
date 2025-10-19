use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    age: u32,
    department: String,
    salary: f64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Product {
    id: String,
    name: String,
    category: String,
    price: f64,
    stock: i32,
    rating: f64,
}

fn main() -> Result<(), Error> {
    println!("AcornDB LINQ-Style Query Example");
    println!();

    // Open a tree with memory storage
    let mut tree = AcornTree::open("memory://")?;
    println!("✓ Opened database");
    println!();

    // Example 1: Basic Query Operations
    println!("=== Example 1: Basic Query Operations ===");
    
    // Store some users
    let users = vec![
        ("user-001", User {
            id: "user-001".to_string(),
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            age: 28,
            department: "Engineering".to_string(),
            salary: 75000.0,
        }),
        ("user-002", User {
            id: "user-002".to_string(),
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            age: 35,
            department: "Marketing".to_string(),
            salary: 65000.0,
        }),
        ("user-003", User {
            id: "user-003".to_string(),
            name: "Charlie Brown".to_string(),
            email: "charlie@example.com".to_string(),
            age: 42,
            department: "Engineering".to_string(),
            salary: 85000.0,
        }),
        ("user-004", User {
            id: "user-004".to_string(),
            name: "Diana Prince".to_string(),
            email: "diana@example.com".to_string(),
            age: 31,
            department: "Sales".to_string(),
            salary: 70000.0,
        }),
        ("user-005", User {
            id: "user-005".to_string(),
            name: "Eve Wilson".to_string(),
            email: "eve@example.com".to_string(),
            age: 26,
            department: "Engineering".to_string(),
            salary: 72000.0,
        }),
    ];

    tree.batch_stash(&users)?;
    println!("✓ Stored {} users", users.len());

    // Query 1: Get all users
    let all_users: Vec<User> = tree.query().collect()?;
    println!("✓ Query 1 - All users: {} found", all_users.len());

    // Query 2: Get first user
    let first_user: Option<User> = tree.query().first()?;
    if let Some(user) = first_user {
        println!("✓ Query 2 - First user: {}", user.name);
    }

    // Query 3: Count users
    let user_count = tree.query().count()?;
    println!("✓ Query 3 - Total users: {}", user_count);

    // Query 4: Check if any users exist
    let has_users = tree.query().any()?;
    println!("✓ Query 4 - Has users: {}", has_users);
    println!();

    // Example 2: Filtering with WHERE conditions
    println!("=== Example 2: Filtering with WHERE Conditions ===");

    // Query 5: Users in Engineering department
    let engineers: Vec<User> = tree.query()
        .where_condition(|user| {
            user["department"].as_str() == Some("Engineering")
        })
        .collect()?;
    println!("✓ Query 5 - Engineers: {} found", engineers.len());
    for user in &engineers {
        println!("  - {} ({})", user.name, user.department);
    }

    // Query 6: Users older than 30
    let seniors: Vec<User> = tree.query()
        .where_condition(|user| {
            user["age"].as_u64().unwrap_or(0) > 30
        })
        .collect()?;
    println!("✓ Query 6 - Senior users (>30): {} found", seniors.len());
    for user in &seniors {
        println!("  - {} (age: {})", user.name, user.age);
    }

    // Query 7: High earners (salary > 70k)
    let high_earners: Vec<User> = tree.query()
        .where_condition(|user| {
            user["salary"].as_f64().unwrap_or(0.0) > 70000.0
        })
        .collect()?;
    println!("✓ Query 7 - High earners (>70k): {} found", high_earners.len());
    for user in &high_earners {
        println!("  - {} (${:.0})", user.name, user.salary);
    }
    println!();

    // Example 3: Ordering with ORDER BY
    println!("=== Example 3: Ordering with ORDER BY ===");

    // Query 8: Users ordered by name
    let users_by_name: Vec<User> = tree.query()
        .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
        .collect()?;
    println!("✓ Query 8 - Users by name:");
    for user in &users_by_name {
        println!("  - {}", user.name);
    }

    // Query 9: Users ordered by salary (descending)
    let users_by_salary: Vec<User> = tree.query()
        .order_by_descending(|user| user["salary"].as_f64().unwrap_or(0.0).to_string())
        .collect()?;
    println!("✓ Query 9 - Users by salary (highest first):");
    for user in &users_by_salary {
        println!("  - {} (${:.0})", user.name, user.salary);
    }
    println!();

    // Example 4: Combining WHERE and ORDER BY
    println!("=== Example 4: Combining WHERE and ORDER BY ===");

    // Query 10: Engineering users ordered by salary
    let engineers_by_salary: Vec<User> = tree.query()
        .where_condition(|user| user["department"].as_str() == Some("Engineering"))
        .order_by_descending(|user| user["salary"].as_f64().unwrap_or(0.0).to_string())
        .collect()?;
    println!("✓ Query 10 - Engineers by salary:");
    for user in &engineers_by_salary {
        println!("  - {} (${:.0})", user.name, user.salary);
    }

    // Query 11: Young users ordered by age
    let young_users: Vec<User> = tree.query()
        .where_condition(|user| user["age"].as_u64().unwrap_or(0) < 35)
        .order_by(|user| user["age"].as_u64().unwrap_or(0).to_string())
        .collect()?;
    println!("✓ Query 11 - Young users (<35) by age:");
    for user in &young_users {
        println!("  - {} (age: {})", user.name, user.age);
    }
    println!();

    // Example 5: Pagination with TAKE and SKIP
    println!("=== Example 5: Pagination with TAKE and SKIP ===");

    // Query 12: First 2 users
    let first_two: Vec<User> = tree.query()
        .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
        .take(2)
        .collect()?;
    println!("✓ Query 12 - First 2 users:");
    for user in &first_two {
        println!("  - {}", user.name);
    }

    // Query 13: Skip first 2, take next 2
    let next_two: Vec<User> = tree.query()
        .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
        .skip(2)
        .collect()?;
    println!("✓ Query 13 - Users after first 2:");
    for user in &next_two {
        println!("  - {}", user.name);
    }
    println!();

    // Example 6: Complex Product Queries
    println!("=== Example 6: Complex Product Queries ===");

    // Store some products
    let products = vec![
        ("product-001", Product {
            id: "product-001".to_string(),
            name: "Laptop Pro".to_string(),
            category: "Electronics".to_string(),
            price: 1299.99,
            stock: 15,
            rating: 4.5,
        }),
        ("product-002", Product {
            id: "product-002".to_string(),
            name: "Wireless Mouse".to_string(),
            category: "Electronics".to_string(),
            price: 29.99,
            stock: 50,
            rating: 4.2,
        }),
        ("product-003", Product {
            id: "product-003".to_string(),
            name: "Office Chair".to_string(),
            category: "Furniture".to_string(),
            price: 199.99,
            stock: 8,
            rating: 4.8,
        }),
        ("product-004", Product {
            id: "product-004".to_string(),
            name: "Coffee Mug".to_string(),
            category: "Accessories".to_string(),
            price: 12.99,
            stock: 100,
            rating: 4.0,
        }),
        ("product-005", Product {
            id: "product-005".to_string(),
            name: "Monitor 4K".to_string(),
            category: "Electronics".to_string(),
            price: 399.99,
            stock: 12,
            rating: 4.7,
        }),
    ];

    tree.batch_stash(&products)?;
    println!("✓ Stored {} products", products.len());

    // Query 14: Electronics products with high rating
    let top_electronics: Vec<Product> = tree.query()
        .where_condition(|product| {
            product["category"].as_str() == Some("Electronics") &&
            product["rating"].as_f64().unwrap_or(0.0) >= 4.5
        })
        .order_by_descending(|product| product["rating"].as_f64().unwrap_or(0.0).to_string())
        .collect()?;
    println!("✓ Query 14 - Top electronics (rating >= 4.5):");
    for product in &top_electronics {
        println!("  - {} (${:.2}, rating: {})", product.name, product.price, product.rating);
    }

    // Query 15: Low stock products
    let low_stock: Vec<Product> = tree.query()
        .where_condition(|product| product["stock"].as_i64().unwrap_or(0) < 20)
        .order_by(|product| product["stock"].as_i64().unwrap_or(0).to_string())
        .collect()?;
    println!("✓ Query 15 - Low stock products (<20):");
    for product in &low_stock {
        println!("  - {} (stock: {})", product.name, product.stock);
    }

    // Query 16: Expensive products
    let expensive: Vec<Product> = tree.query()
        .where_condition(|product| product["price"].as_f64().unwrap_or(0.0) > 100.0)
        .order_by_descending(|product| product["price"].as_f64().unwrap_or(0.0).to_string())
        .take(3)
        .collect()?;
    println!("✓ Query 16 - Top 3 expensive products (>$100):");
    for product in &expensive {
        println!("  - {} (${:.2})", product.name, product.price);
    }
    println!();

    // Example 7: Performance comparison
    println!("=== Example 7: Performance Comparison ===");
    use std::time::Instant;

    // Traditional iteration approach
    let start = Instant::now();
    let mut traditional_count = 0;
    let mut iter = tree.iter("")?;
    while let Some((_, _)) = iter.next::<User>()? {
        traditional_count += 1;
    }
    let traditional_time = start.elapsed();

    // Query approach
    let start = Instant::now();
    let query_count = tree.query().count()?;
    let query_time = start.elapsed();

    println!("✓ Traditional iteration: {} items in {:?}", traditional_count, traditional_time);
    println!("✓ Query count: {} items in {:?}", query_count, query_time);
    println!();

    println!("🎉 LINQ-style query example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("- ✅ Basic queries: collect(), first(), count(), any()");
    println!("- ✅ Filtering: where_condition() with JSON path access");
    println!("- ✅ Ordering: order_by() and order_by_descending()");
    println!("- ✅ Pagination: take() and skip()");
    println!("- ✅ Complex queries: combining multiple operations");
    println!("- ✅ Type safety: automatic deserialization to Rust structs");
    println!("- ✅ Performance: efficient query execution");

    Ok(())
}
