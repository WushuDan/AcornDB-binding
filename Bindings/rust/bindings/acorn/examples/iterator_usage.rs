use acorn::{AcornTree, Error};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct Product {
    name: String,
    price: f64,
    category: String,
}

fn main() -> Result<(), Error> {
    println!("AcornDB Iterator Example");
    println!();

    // Open a tree with memory storage (avoids file deserialization issues)
    let mut tree = AcornTree::open("memory://")?;
    println!("✓ Opened database");

    // Store some products with category prefixes
    let products = vec![
        ("electronics:laptop", Product { name: "Laptop".to_string(), price: 999.99, category: "electronics".to_string() }),
        ("electronics:phone", Product { name: "Smartphone".to_string(), price: 599.99, category: "electronics".to_string() }),
        ("electronics:tablet", Product { name: "Tablet".to_string(), price: 399.99, category: "electronics".to_string() }),
        ("books:rust", Product { name: "The Rust Book".to_string(), price: 39.99, category: "books".to_string() }),
        ("books:golang", Product { name: "Go in Action".to_string(), price: 44.99, category: "books".to_string() }),
        ("clothing:shirt", Product { name: "T-Shirt".to_string(), price: 19.99, category: "clothing".to_string() }),
        ("clothing:jeans", Product { name: "Jeans".to_string(), price: 59.99, category: "clothing".to_string() }),
    ];

    for (id, product) in &products {
        tree.stash(id, product)?;
    }
    println!("✓ Stored {} products", products.len());
    println!();

    // Example 1: Iterate over all items
    println!("=== All Products ===");
    let mut iter = tree.iter("")?;
    while let Some((key, product)) = iter.next::<Product>()? {
        println!("  {}: {} - ${:.2}", key, product.name, product.price);
    }
    println!();

    // Example 2: Iterate over electronics only
    println!("=== Electronics ===");
    let mut electronics_iter = tree.iter("electronics:")?;
    let electronics: Vec<(String, Product)> = electronics_iter.collect()?;
    for (key, product) in electronics {
        println!("  {}: {} - ${:.2}", key, product.name, product.price);
    }
    println!();

    // Example 3: Iterate over books only
    println!("=== Books ===");
    let mut books_iter = tree.iter("books:")?;
    let books: Vec<(String, Product)> = books_iter.collect()?;
    for (key, product) in books {
        println!("  {}: {} - ${:.2}", key, product.name, product.price);
    }
    println!();

    // Example 4: Calculate total value by category
    println!("=== Total Value by Category ===");

    let mut electronics_iter = tree.iter("electronics:")?;
    let electronics_total: f64 = electronics_iter
        .collect::<Product>()?
        .iter()
        .map(|(_, p)| p.price)
        .sum();
    println!("  Electronics: ${:.2}", electronics_total);

    let mut books_iter = tree.iter("books:")?;
    let books_total: f64 = books_iter
        .collect::<Product>()?
        .iter()
        .map(|(_, p)| p.price)
        .sum();
    println!("  Books: ${:.2}", books_total);

    let mut clothing_iter = tree.iter("clothing:")?;
    let clothing_total: f64 = clothing_iter
        .collect::<Product>()?
        .iter()
        .map(|(_, p)| p.price)
        .sum();
    println!("  Clothing: ${:.2}", clothing_total);
    println!();

    // Example 5: Find expensive items (> $500)
    println!("=== Expensive Items (>$500) ===");
    let mut all_iter = tree.iter("")?;
    while let Some((key, product)) = all_iter.next::<Product>()? {
        if product.price > 500.0 {
            println!("  {}: {} - ${:.2}", key, product.name, product.price);
        }
    }
    println!();

    println!("🎉 Iterator example completed successfully!");
    Ok(())
}
