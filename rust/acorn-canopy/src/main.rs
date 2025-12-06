#[cfg(feature = "server")]
fn main() {
    println!("acorn-canopy stub: backend and UI scaffolding will be added in later phases.");
}

#[cfg(not(feature = "server"))]
fn main() {
    println!("acorn-canopy built without `server` feature; nothing to start yet.");
}
