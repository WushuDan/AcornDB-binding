use acorn::{
    AcornTree, AcornPerformanceMonitor, AcornHealthChecker, AcornBenchmark, AcornResourceMonitor,
    AcornTangle, AcornMeshCoordinator, BenchmarkConfig, HealthStatus, Error
};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use std::thread;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct User {
    id: String,
    name: String,
    email: String,
    role: String,
    last_login: SystemTime,
    is_active: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Product {
    id: String,
    name: String,
    price: f64,
    category: String,
    stock_count: i32,
    created_at: SystemTime,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
struct Order {
    id: String,
    user_id: String,
    product_id: String,
    quantity: i32,
    total_price: f64,
    status: String,
    created_at: SystemTime,
}

fn main() -> Result<(), Error> {
    println!("📊 AcornDB Performance Monitoring Example");
    println!("==========================================");

    // Example 1: Performance Monitoring
    println!("=== Example 1: Performance Monitoring ===");

    let monitor = AcornPerformanceMonitor::new()?;
    
    // Start collecting metrics
    monitor.start_collection()?;
    println!("📈 Started performance monitoring");

    // Create a tree and perform operations
    let tree = AcornTree::open_memory()?;
    
    // Generate some load
    println!("🔄 Generating load for performance monitoring...");
    for i in 0..1000 {
        let user = User {
            id: format!("user-{}", i),
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
            role: if i % 10 == 0 { "admin".to_string() } else { "user".to_string() },
            last_login: SystemTime::now(),
            is_active: i % 2 == 0,
        };
        
        let user_json = serde_json::to_string(&user)?;
        tree.stash(&user.id, &user_json)?;
        
        if i % 100 == 0 {
            println!("   Processed {} users", i);
        }
    }

    // Get current metrics
    let metrics = monitor.get_metrics()?;
    println!("📊 Current Performance Metrics:");
    println!("   Operations per second: {}", metrics.operations_per_second);
    println!("   Memory usage: {} bytes", metrics.memory_usage_bytes);
    println!("   Cache hit rate: {}%", metrics.cache_hit_rate_percent);
    println!("   Sync latency: {} ms", metrics.sync_latency_ms);
    println!("   Disk I/O: {} bytes", metrics.disk_io_bytes);
    println!("   Network: {} bytes", metrics.network_bytes);
    println!("   CPU usage: {}%", metrics.cpu_usage_percent);
    println!("   Timestamp: {}", metrics.timestamp);
    println!();

    // Stop collection and get history
    monitor.stop_collection()?;
    let history = monitor.get_history()?;
    println!("📈 Performance History: {} samples collected", history.len());
    
    if !history.is_empty() {
        let latest = &history[history.len() - 1];
        println!("   Latest sample: {} ops/sec, {} bytes memory", 
            latest.operations_per_second, latest.memory_usage_bytes);
    }
    println!();

    // Example 2: Health Checking
    println!("=== Example 2: Health Checking ===");

    let health_checker = AcornHealthChecker::new()?;
    
    // Add services to monitor (simulated endpoints)
    health_checker.add_service("database", "http://localhost:5432/health")?;
    health_checker.add_service("api", "http://localhost:8080/health")?;
    health_checker.add_service("cache", "http://localhost:6379/health")?;
    health_checker.add_service("storage", "http://localhost:9000/health")?;
    
    println!("🏥 Added 4 services to health monitoring");

    // Check all services
    let health_results = health_checker.check_all()?;
    println!("🏥 Health Check Results:");
    for result in &health_results {
        let status_emoji = match result.status {
            HealthStatus::Healthy => "✅",
            HealthStatus::Degraded => "⚠️",
            HealthStatus::Unhealthy => "❌",
            HealthStatus::Unknown => "❓",
        };
        
        println!("   {} {}: {} ({}ms)", 
            status_emoji, result.service_name, result.message, result.response_time_ms);
        
        if let Some(details) = &result.details {
            println!("      Details: {}", details);
        }
    }

    // Get overall health status
    let overall_status = health_checker.get_overall_status()?;
    let overall_emoji = match overall_status {
        HealthStatus::Healthy => "✅",
        HealthStatus::Degraded => "⚠️",
        HealthStatus::Unhealthy => "❌",
        HealthStatus::Unknown => "❓",
    };
    println!("🏥 Overall Health Status: {} {:?}", overall_emoji, overall_status);
    println!();

    // Example 3: Benchmarking
    println!("=== Example 3: Benchmarking ===");

    let benchmark_tree = AcornTree::open_memory()?;
    
    // Configure benchmark
    let config = BenchmarkConfig {
        operation_count: 1000,
        warmup_iterations: 10,
        measurement_iterations: 100,
        timeout_ms: 30000,
        enable_memory_tracking: true,
        enable_disk_tracking: false,
        enable_network_tracking: false,
    };

    println!("⚡ Running tree operations benchmark...");
    let tree_results = AcornBenchmark::benchmark_tree_operations(benchmark_tree, &config)?;
    
    println!("⚡ Tree Operations Benchmark Results:");
    for result in &tree_results {
        println!("   {}: {} ops/sec (avg: {:.2}ms, p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms)", 
            result.operation_name, 
            result.operations_per_second,
            result.average_latency_ms,
            result.p50_latency_ms,
            result.p95_latency_ms,
            result.p99_latency_ms);
        println!("      Memory: {} bytes, Disk I/O: {} bytes, Network: {} bytes",
            result.memory_allocated_bytes, result.disk_io_bytes, result.network_bytes);
    }
    println!();

    // Example 4: Sync Operations Benchmarking
    println!("=== Example 4: Sync Operations Benchmarking ===");

    let local_tree = AcornTree::open_memory()?;
    let remote_tree = AcornTree::open_memory()?;
    let tangle = AcornTangle::new(local_tree, remote_tree, "benchmark-tangle")?;

    let sync_config = BenchmarkConfig {
        operation_count: 500,
        warmup_iterations: 5,
        measurement_iterations: 50,
        timeout_ms: 30000,
        enable_memory_tracking: true,
        enable_disk_tracking: false,
        enable_network_tracking: true,
    };

    println!("🔄 Running sync operations benchmark...");
    let sync_results = AcornBenchmark::benchmark_sync_operations(tangle, &sync_config)?;
    
    println!("🔄 Sync Operations Benchmark Results:");
    for result in &sync_results {
        println!("   {}: {} ops/sec (avg: {:.2}ms, p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms)", 
            result.operation_name, 
            result.operations_per_second,
            result.average_latency_ms,
            result.p50_latency_ms,
            result.p95_latency_ms,
            result.p99_latency_ms);
        println!("      Memory: {} bytes, Network: {} bytes",
            result.memory_allocated_bytes, result.network_bytes);
    }
    println!();

    // Example 5: Mesh Operations Benchmarking
    println!("=== Example 5: Mesh Operations Benchmarking ===");

    let coordinator = AcornMeshCoordinator::new()?;
    let tree_a = AcornTree::open_memory()?;
    let tree_b = AcornTree::open_memory()?;
    let tree_c = AcornTree::open_memory()?;
    
    coordinator.add_node("node-a", tree_a)?;
    coordinator.add_node("node-b", tree_b)?;
    coordinator.add_node("node-c", tree_c)?;
    coordinator.create_topology(acorn::MeshTopology::Full, "")?;

    let mesh_config = BenchmarkConfig {
        operation_count: 300,
        warmup_iterations: 5,
        measurement_iterations: 30,
        timeout_ms: 30000,
        enable_memory_tracking: true,
        enable_disk_tracking: false,
        enable_network_tracking: true,
    };

    println!("🕸️  Running mesh operations benchmark...");
    let mesh_results = AcornBenchmark::benchmark_mesh_operations(coordinator, &mesh_config)?;
    
    println!("🕸️  Mesh Operations Benchmark Results:");
    for result in &mesh_results {
        println!("   {}: {} ops/sec (avg: {:.2}ms, p50: {:.2}ms, p95: {:.2}ms, p99: {:.2}ms)", 
            result.operation_name, 
            result.operations_per_second,
            result.average_latency_ms,
            result.p50_latency_ms,
            result.p95_latency_ms,
            result.p99_latency_ms);
        println!("      Memory: {} bytes, Network: {} bytes",
            result.memory_allocated_bytes, result.network_bytes);
    }
    println!();

    // Example 6: Resource Monitoring
    println!("=== Example 6: Resource Monitoring ===");

    // Get memory usage
    let (heap_bytes, stack_bytes, total_bytes) = AcornResourceMonitor::get_memory_usage()?;
    println!("💾 Memory Usage:");
    println!("   Heap: {} bytes ({:.2} MB)", heap_bytes, heap_bytes as f64 / 1_048_576.0);
    println!("   Stack: {} bytes ({:.2} MB)", stack_bytes, stack_bytes as f64 / 1_048_576.0);
    println!("   Total: {} bytes ({:.2} MB)", total_bytes, total_bytes as f64 / 1_048_576.0);

    // Get disk usage
    let (used_bytes, total_disk_bytes, free_bytes) = AcornResourceMonitor::get_disk_usage("/tmp")?;
    println!("💽 Disk Usage (/tmp):");
    println!("   Used: {} bytes ({:.2} MB)", used_bytes, used_bytes as f64 / 1_048_576.0);
    println!("   Free: {} bytes ({:.2} MB)", free_bytes, free_bytes as f64 / 1_048_576.0);
    println!("   Total: {} bytes ({:.2} MB)", total_disk_bytes, total_disk_bytes as f64 / 1_048_576.0);

    // Get system information
    let system_info = AcornResourceMonitor::get_system_info()?;
    println!("🖥️  System Information:");
    println!("   {}", system_info);
    println!();

    // Example 7: Continuous Performance Monitoring
    println!("=== Example 7: Continuous Performance Monitoring ===");

    let continuous_monitor = AcornPerformanceMonitor::new()?;
    continuous_monitor.start_collection()?;

    // Simulate continuous workload
    let workload_tree = AcornTree::open_memory()?;
    
    println!("🔄 Running continuous workload for 5 seconds...");
    let start_time = SystemTime::now();
    let mut operation_count = 0;
    
    while start_time.elapsed().unwrap() < Duration::from_secs(5) {
        // Generate mixed workload
        let product = Product {
            id: format!("product-{}", operation_count),
            name: format!("Product {}", operation_count),
            price: (operation_count as f64) * 10.0,
            category: match operation_count % 4 {
                0 => "Electronics".to_string(),
                1 => "Clothing".to_string(),
                2 => "Books".to_string(),
                _ => "Home".to_string(),
            },
            stock_count: operation_count % 100,
            created_at: SystemTime::now(),
        };
        
        let product_json = serde_json::to_string(&product)?;
        workload_tree.stash(&product.id, &product_json)?;
        
        // Occasionally read data
        if operation_count % 10 == 0 {
            let _ = workload_tree.crack(&product.id);
        }
        
        operation_count += 1;
        
        // Brief pause to avoid overwhelming the system
        if operation_count % 100 == 0 {
            thread::sleep(Duration::from_millis(1));
        }
    }

    continuous_monitor.stop_collection()?;
    
    // Get final metrics
    let final_metrics = continuous_monitor.get_metrics()?;
    let duration = start_time.elapsed().unwrap();
    let ops_per_sec = operation_count as f64 / duration.as_secs_f64();
    
    println!("📊 Continuous Workload Results:");
    println!("   Total operations: {}", operation_count);
    println!("   Duration: {:.2} seconds", duration.as_secs_f64());
    println!("   Operations per second: {:.2}", ops_per_sec);
    println!("   Final metrics - OPS: {}, Memory: {} bytes, CPU: {}%", 
        final_metrics.operations_per_second, 
        final_metrics.memory_usage_bytes, 
        final_metrics.cpu_usage_percent);
    
    // Get metrics history
    let continuous_history = continuous_monitor.get_history()?;
    println!("   Metrics samples collected: {}", continuous_history.len());
    
    if continuous_history.len() > 1 {
        let first = &continuous_history[0];
        let last = &continuous_history[continuous_history.len() - 1];
        println!("   Memory growth: {} bytes", last.memory_usage_bytes - first.memory_usage_bytes);
        println!("   CPU trend: {}% -> {}%", first.cpu_usage_percent, last.cpu_usage_percent);
    }
    println!();

    // Example 8: Performance Analysis
    println!("=== Example 8: Performance Analysis ===");

    let analysis_tree = AcornTree::open_memory()?;
    let analysis_monitor = AcornPerformanceMonitor::new()?;
    
    // Test different operation patterns
    let patterns = vec![
        ("Sequential Writes", 1000),
        ("Random Reads", 1000),
        ("Mixed Operations", 1000),
        ("Batch Operations", 1000),
    ];
    
    for (pattern_name, count) in patterns {
        analysis_monitor.start_collection()?;
        
        match pattern_name {
            "Sequential Writes" => {
                for i in 0..count {
                    let order = Order {
                        id: format!("order-{}", i),
                        user_id: format!("user-{}", i % 100),
                        product_id: format!("product-{}", i % 50),
                        quantity: (i % 10) + 1,
                        total_price: (i as f64) * 15.0,
                        status: "pending".to_string(),
                        created_at: SystemTime::now(),
                    };
                    let order_json = serde_json::to_string(&order)?;
                    analysis_tree.stash(&order.id, &order_json)?;
                }
            },
            "Random Reads" => {
                for i in 0..count {
                    let key = format!("order-{}", i % 1000);
                    let _ = analysis_tree.crack(&key);
                }
            },
            "Mixed Operations" => {
                for i in 0..count {
                    if i % 3 == 0 {
                        // Write
                        let order = Order {
                            id: format!("mixed-{}", i),
                            user_id: format!("user-{}", i % 100),
                            product_id: format!("product-{}", i % 50),
                            quantity: (i % 10) + 1,
                            total_price: (i as f64) * 15.0,
                            status: "pending".to_string(),
                            created_at: SystemTime::now(),
                        };
                        let order_json = serde_json::to_string(&order)?;
                        analysis_tree.stash(&order.id, &order_json)?;
                    } else {
                        // Read
                        let key = format!("mixed-{}", i % 1000);
                        let _ = analysis_tree.crack(&key);
                    }
                }
            },
            "Batch Operations" => {
                let batch_size = 10;
                for batch in 0..(count / batch_size) {
                    for i in 0..batch_size {
                        let order = Order {
                            id: format!("batch-{}-{}", batch, i),
                            user_id: format!("user-{}", (batch * batch_size + i) % 100),
                            product_id: format!("product-{}", (batch * batch_size + i) % 50),
                            quantity: ((batch * batch_size + i) % 10) + 1,
                            total_price: ((batch * batch_size + i) as f64) * 15.0,
                            status: "pending".to_string(),
                            created_at: SystemTime::now(),
                        };
                        let order_json = serde_json::to_string(&order)?;
                        analysis_tree.stash(&order.id, &order_json)?;
                    }
                    // Brief pause between batches
                    thread::sleep(Duration::from_millis(1));
                }
            },
            _ => {}
        }
        
        analysis_monitor.stop_collection()?;
        
        let metrics = analysis_monitor.get_metrics()?;
        println!("📈 {} Performance Analysis:", pattern_name);
        println!("   Operations: {}, OPS: {}, Memory: {} bytes, CPU: {}%", 
            count, metrics.operations_per_second, metrics.memory_usage_bytes, metrics.cpu_usage_percent);
        
        analysis_monitor.reset_metrics()?;
    }
    println!();

    // Example 9: Health Monitoring Dashboard
    println!("=== Example 9: Health Monitoring Dashboard ===");

    let dashboard_checker = AcornHealthChecker::new()?;
    
    // Add various services
    let services = vec![
        ("acorn-db", "http://localhost:5000/health"),
        ("user-service", "http://localhost:5001/health"),
        ("product-service", "http://localhost:5002/health"),
        ("order-service", "http://localhost:5003/health"),
        ("payment-service", "http://localhost:5004/health"),
        ("notification-service", "http://localhost:5005/health"),
    ];
    
    for (service_name, endpoint) in &services {
        dashboard_checker.add_service(service_name, endpoint)?;
    }
    
    println!("🏥 Health Monitoring Dashboard");
    println!("==============================");
    
    // Simulate health checks
    for round in 1..=3 {
        println!("Round {} Health Check:", round);
        
        let results = dashboard_checker.check_all()?;
        let mut healthy_count = 0;
        let mut degraded_count = 0;
        let mut unhealthy_count = 0;
        
        for result in &results {
            match result.status {
                HealthStatus::Healthy => healthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Unknown => {}
            }
        }
        
        println!("   Status Summary: {} healthy, {} degraded, {} unhealthy", 
            healthy_count, degraded_count, unhealthy_count);
        
        for result in &results {
            let status_emoji = match result.status {
                HealthStatus::Healthy => "🟢",
                HealthStatus::Degraded => "🟡",
                HealthStatus::Unhealthy => "🔴",
                HealthStatus::Unknown => "⚪",
            };
            
            println!("   {} {}: {} ({}ms)", 
                status_emoji, result.service_name, result.message, result.response_time_ms);
        }
        
        let overall_status = dashboard_checker.get_overall_status()?;
        let overall_emoji = match overall_status {
            HealthStatus::Healthy => "🟢",
            HealthStatus::Degraded => "🟡",
            HealthStatus::Unhealthy => "🔴",
            HealthStatus::Unknown => "⚪",
        };
        println!("   Overall Status: {} {:?}", overall_emoji, overall_status);
        println!();
        
        if round < 3 {
            thread::sleep(Duration::from_millis(500));
        }
    }
    println!();

    // Example 10: Comprehensive Performance Report
    println!("=== Example 10: Comprehensive Performance Report ===");

    let report_monitor = AcornPerformanceMonitor::new()?;
    let report_tree = AcornTree::open_memory()?;
    
    // Generate comprehensive workload
    report_monitor.start_collection()?;
    
    println!("📊 Generating comprehensive workload...");
    
    // Create users
    for i in 0..500 {
        let user = User {
            id: format!("user-{}", i),
            name: format!("User {}", i),
            email: format!("user{}@example.com", i),
            role: if i % 20 == 0 { "admin".to_string() } else { "user".to_string() },
            last_login: SystemTime::now(),
            is_active: i % 3 != 0,
        };
        let user_json = serde_json::to_string(&user)?;
        report_tree.stash(&user.id, &user_json)?;
    }
    
    // Create products
    for i in 0..200 {
        let product = Product {
            id: format!("product-{}", i),
            name: format!("Product {}", i),
            price: (i as f64) * 25.0,
            category: match i % 5 {
                0 => "Electronics".to_string(),
                1 => "Clothing".to_string(),
                2 => "Books".to_string(),
                3 => "Home".to_string(),
                _ => "Sports".to_string(),
            },
            stock_count: (i % 100) + 10,
            created_at: SystemTime::now(),
        };
        let product_json = serde_json::to_string(&product)?;
        report_tree.stash(&product.id, &product_json)?;
    }
    
    // Create orders
    for i in 0..1000 {
        let order = Order {
            id: format!("order-{}", i),
            user_id: format!("user-{}", i % 500),
            product_id: format!("product-{}", i % 200),
            quantity: (i % 5) + 1,
            total_price: ((i % 200) as f64) * 25.0 * ((i % 5) + 1) as f64,
            status: match i % 4 {
                0 => "pending".to_string(),
                1 => "processing".to_string(),
                2 => "shipped".to_string(),
                _ => "delivered".to_string(),
            },
            created_at: SystemTime::now(),
        };
        let order_json = serde_json::to_string(&order)?;
        report_tree.stash(&order.id, &order_json)?;
    }
    
    // Perform reads
    for i in 0..500 {
        let user_key = format!("user-{}", i % 500);
        let _ = report_tree.crack(&user_key);
        
        let product_key = format!("product-{}", i % 200);
        let _ = report_tree.crack(&product_key);
        
        let order_key = format!("order-{}", i % 1000);
        let _ = report_tree.crack(&order_key);
    }
    
    report_monitor.stop_collection()?;
    
    // Generate comprehensive report
    let final_metrics = report_monitor.get_metrics()?;
    let history = report_monitor.get_history()?;
    let (heap_bytes, stack_bytes, total_bytes) = AcornResourceMonitor::get_memory_usage()?;
    
    println!("📊 Comprehensive Performance Report");
    println!("==================================");
    println!("📈 Performance Metrics:");
    println!("   Operations per second: {}", final_metrics.operations_per_second);
    println!("   Memory usage: {} bytes ({:.2} MB)", final_metrics.memory_usage_bytes, final_metrics.memory_usage_bytes as f64 / 1_048_576.0);
    println!("   Cache hit rate: {}%", final_metrics.cache_hit_rate_percent);
    println!("   Sync latency: {} ms", final_metrics.sync_latency_ms);
    println!("   Disk I/O: {} bytes ({:.2} MB)", final_metrics.disk_io_bytes, final_metrics.disk_io_bytes as f64 / 1_048_576.0);
    println!("   Network: {} bytes ({:.2} MB)", final_metrics.network_bytes, final_metrics.network_bytes as f64 / 1_048_576.0);
    println!("   CPU usage: {}%", final_metrics.cpu_usage_percent);
    println!();
    
    println!("💾 Resource Usage:");
    println!("   Heap memory: {} bytes ({:.2} MB)", heap_bytes, heap_bytes as f64 / 1_048_576.0);
    println!("   Stack memory: {} bytes ({:.2} MB)", stack_bytes, stack_bytes as f64 / 1_048_576.0);
    println!("   Total memory: {} bytes ({:.2} MB)", total_bytes, total_bytes as f64 / 1_048_576.0);
    println!();
    
    println!("📊 Data Statistics:");
    println!("   Users created: 500");
    println!("   Products created: 200");
    println!("   Orders created: 1000");
    println!("   Read operations: 1500");
    println!("   Total operations: 3200");
    println!();
    
    println!("📈 Monitoring Statistics:");
    println!("   Metrics samples collected: {}", history.len());
    if history.len() > 1 {
        let first = &history[0];
        let last = &history[history.len() - 1];
        println!("   Memory growth: {} bytes ({:.2} MB)", 
            last.memory_usage_bytes - first.memory_usage_bytes,
            (last.memory_usage_bytes - first.memory_usage_bytes) as f64 / 1_048_576.0);
        println!("   Performance trend: {} -> {} ops/sec", 
            first.operations_per_second, last.operations_per_second);
    }
    println!();

    println!("🎉 Performance Monitoring example completed successfully!");
    println!();
    println!("Key Features Demonstrated:");
    println!("✅ Performance Monitoring: Real-time metrics collection and analysis");
    println!("✅ Health Checking: Service health monitoring with status tracking");
    println!("✅ Benchmarking: Comprehensive performance testing for trees, sync, and mesh");
    println!("✅ Resource Monitoring: Memory, disk, and system resource tracking");
    println!("✅ Continuous Monitoring: Long-running performance observation");
    println!("✅ Performance Analysis: Pattern-based performance analysis");
    println!("✅ Health Dashboard: Multi-service health monitoring dashboard");
    println!("✅ Comprehensive Reporting: Detailed performance and resource reports");
    println!("✅ Metrics History: Historical performance data collection");
    println!("✅ System Integration: Integration with system-level monitoring");

    Ok(())
}
