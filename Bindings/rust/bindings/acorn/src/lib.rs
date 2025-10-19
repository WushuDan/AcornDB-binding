use acorn_sys::*;
use serde::{de::DeserializeOwned, Serialize};
use std::{ffi::CString, ptr};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Acorn error: {0}")]
    Acorn(String),
    #[error("Not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;

pub struct AcornTree { h: acorn_tree_handle }

/// Encryption provider for AcornDB
pub struct AcornEncryption { h: acorn_encryption_handle }

impl AcornEncryption {
    /// Create an encryption provider from a password and salt using PBKDF2 key derivation
    /// 
    /// # Arguments
    /// * `password` - The password to derive the encryption key from
    /// * `salt` - The salt to use for key derivation (should be unique per database)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_password("my-secret-password", "my-unique-salt")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_password(password: &str, salt: &str) -> Result<Self> {
        let password_c = CString::new(password).map_err(|e| Error::Acorn(format!("Invalid password: {}", e)))?;
        let salt_c = CString::new(salt).map_err(|e| Error::Acorn(format!("Invalid salt: {}", e)))?;
        let mut h: acorn_encryption_handle = 0;
        let rc = unsafe { acorn_encryption_from_password(password_c.as_ptr(), salt_c.as_ptr(), &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create an encryption provider from explicit key and IV (base64 encoded)
    /// 
    /// # Arguments
    /// * `key_base64` - The encryption key encoded as base64 (must be 32 bytes)
    /// * `iv_base64` - The initialization vector encoded as base64 (must be 16 bytes)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_key_iv("base64-key", "base64-iv")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_key_iv(key_base64: &str, iv_base64: &str) -> Result<Self> {
        let key_c = CString::new(key_base64).map_err(|e| Error::Acorn(format!("Invalid key: {}", e)))?;
        let iv_c = CString::new(iv_base64).map_err(|e| Error::Acorn(format!("Invalid IV: {}", e)))?;
        let mut h: acorn_encryption_handle = 0;
        let rc = unsafe { acorn_encryption_from_key_iv(key_c.as_ptr(), iv_c.as_ptr(), &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Generate a random key and IV for testing or new deployments
    /// 
    /// # Returns
    /// A tuple of (key_base64, iv_base64) strings
    /// 
    /// # Warning
    /// Store the returned key and IV securely - data cannot be decrypted without them!
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let (key, iv) = AcornEncryption::generate_key_iv()?;
    /// println!("Key: {}", key);
    /// println!("IV: {}", iv);
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_key_iv() -> Result<(String, String)> {
        let mut key_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let mut iv_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_generate_key_iv(&mut key_buf as *mut _, &mut iv_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        // Convert buffers to strings
        let key_slice = unsafe { std::slice::from_raw_parts(key_buf.data, key_buf.len) };
        let iv_slice = unsafe { std::slice::from_raw_parts(iv_buf.data, iv_buf.len) };
        
        let key = String::from_utf8(key_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid key UTF-8: {}", e)))?;
        let iv = String::from_utf8(iv_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid IV UTF-8: {}", e)))?;

        // Free the buffers
        unsafe { 
            acorn_free_buf(&mut key_buf as *mut _);
            acorn_free_buf(&mut iv_buf as *mut _);
        }

        Ok((key, iv))
    }

    /// Export the encryption key as a base64 string (for backup/storage)
    pub fn export_key(&self) -> Result<String> {
        let mut key_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_export_key(self.h, &mut key_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let key_slice = unsafe { std::slice::from_raw_parts(key_buf.data, key_buf.len) };
        let key = String::from_utf8(key_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid key UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut key_buf as *mut _) };
        Ok(key)
    }

    /// Export the initialization vector as a base64 string (for backup/storage)
    pub fn export_iv(&self) -> Result<String> {
        let mut iv_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_export_iv(self.h, &mut iv_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let iv_slice = unsafe { std::slice::from_raw_parts(iv_buf.data, iv_buf.len) };
        let iv = String::from_utf8(iv_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid IV UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut iv_buf as *mut _) };
        Ok(iv)
    }

    /// Encrypt plaintext data
    pub fn encrypt(&self, plaintext: &str) -> Result<String> {
        let plaintext_c = CString::new(plaintext).map_err(|e| Error::Acorn(format!("Invalid plaintext: {}", e)))?;
        let mut ciphertext_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_encrypt(self.h, plaintext_c.as_ptr(), &mut ciphertext_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let ciphertext_slice = unsafe { std::slice::from_raw_parts(ciphertext_buf.data, ciphertext_buf.len) };
        let ciphertext = String::from_utf8(ciphertext_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid ciphertext UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut ciphertext_buf as *mut _) };
        Ok(ciphertext)
    }

    /// Decrypt ciphertext data
    pub fn decrypt(&self, ciphertext: &str) -> Result<String> {
        let ciphertext_c = CString::new(ciphertext).map_err(|e| Error::Acorn(format!("Invalid ciphertext: {}", e)))?;
        let mut plaintext_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_encryption_decrypt(self.h, ciphertext_c.as_ptr(), &mut plaintext_buf as *mut _) };
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        let plaintext_slice = unsafe { std::slice::from_raw_parts(plaintext_buf.data, plaintext_buf.len) };
        let plaintext = String::from_utf8(plaintext_slice.to_vec()).map_err(|e| Error::Acorn(format!("Invalid plaintext UTF-8: {}", e)))?;

        unsafe { acorn_free_buf(&mut plaintext_buf as *mut _) };
        Ok(plaintext)
    }

    /// Check if encryption is enabled
    pub fn is_enabled(&self) -> Result<bool> {
        let rc = unsafe { acorn_encryption_is_enabled(self.h) };
        if rc == -1 {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        } else {
            Ok(rc == 1)
        }
    }
}

impl Drop for AcornEncryption {
    fn drop(&mut self) {
        unsafe { acorn_encryption_close(self.h); }
    }
}

impl AcornTree {
    pub fn open(uri: &str) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree(c.as_ptr(), &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with encryption enabled
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./encrypted_db" or "memory://")
    /// * `encryption` - The encryption provider to use
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_password("my-password", "my-salt")?;
    /// let tree = AcornTree::open_encrypted("file://./secure_db", &encryption)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_encrypted(uri: &str, encryption: &AcornEncryption) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_encrypted(c.as_ptr(), encryption.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with both encryption and compression enabled
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./secure_db" or "memory://")
    /// * `encryption` - The encryption provider to use
    /// * `compression_level` - Compression level (0=Fastest, 1=Optimal, 2=SmallestSize)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornEncryption, Error};
    /// # fn main() -> Result<(), Error> {
    /// let encryption = AcornEncryption::from_password("my-password", "my-salt")?;
    /// let tree = AcornTree::open_encrypted_compressed("file://./secure_db", &encryption, 1)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_encrypted_compressed(uri: &str, encryption: &AcornEncryption, compression_level: i32) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_encrypted_compressed(c.as_ptr(), encryption.h, compression_level, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    pub fn stash<T: Serialize>(&mut self, id: &str, value: &T) -> Result<()> {
        let json = serde_json::to_vec(value).map_err(|e| Error::Acorn(format!("Serialization error: {}", e)))?;
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let rc = unsafe { acorn_stash_json(self.h, idc.as_ptr(), json.as_ptr(), json.len()) };
        if rc == 0 { 
            Ok(()) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    pub fn crack<T: DeserializeOwned>(&self, id: &str) -> Result<T> {
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_crack_json(self.h, idc.as_ptr(), &mut buf as *mut _) };
        if rc == 1 {
            return Err(Error::NotFound);
        }
        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        // Safety: We trust the shim to return valid data
        let slice = unsafe { std::slice::from_raw_parts(buf.data, buf.len) };
        let out = serde_json::from_slice(slice).map_err(|e| Error::Acorn(e.to_string()))?;
        unsafe { acorn_free_buf(&mut buf as *mut _) };
        Ok(out)
    }

    /// Create an iterator over key-value pairs with the given prefix.
    /// Pass an empty string to iterate over all keys.
    pub fn iter(&self, prefix: &str) -> Result<AcornIterator> {
        let prefix_c = CString::new(prefix).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        let mut iter_h: acorn_iter_handle = 0;
        let rc = unsafe { acorn_iter_start(self.h, prefix_c.as_ptr(), &mut iter_h as *mut _) };
        if rc == 0 {
            Ok(AcornIterator { h: iter_h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Subscribe to changes in the tree. The callback will be invoked whenever
    /// an item is added or modified. The callback is called from a background thread.
    ///
    /// Returns an `AcornSubscription` that will automatically unsubscribe when dropped.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe(|key: &str, value: &serde_json::Value| {
    ///     println!("Changed: {} = {:?}", key, value);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe<F>(&self, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        AcornSubscription::new(self.h, callback)
    }

    /// Synchronize this tree with a remote HTTP endpoint.
    /// This pulls data from the remote server and merges it into the local tree.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("file://./db")?;
    /// tree.sync_http("http://example.com/api/acorn")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sync_http(&self, url: &str) -> Result<()> {
        let url_c = CString::new(url).map_err(|e| Error::Acorn(format!("Invalid URL: {}", e)))?;
        let rc = unsafe { acorn_sync_http(self.h, url_c.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Store multiple key-value pairs in a single operation.
    /// This is more efficient than calling stash() multiple times.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Data { value: i32 }
    /// # fn main() -> Result<(), Error> {
    /// let mut tree = AcornTree::open("memory://")?;
    /// let items = vec![
    ///     ("key1", Data { value: 1 }),
    ///     ("key2", Data { value: 2 }),
    ///     ("key3", Data { value: 3 }),
    /// ];
    /// tree.batch_stash(&items)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_stash<T: Serialize>(&mut self, items: &[(&str, T)]) -> Result<()> {
        if items.is_empty() {
            return Ok(());
        }

        // Prepare C-compatible arrays
        let ids: Vec<CString> = items
            .iter()
            .map(|(id, _)| CString::new(*id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let jsons: Vec<Vec<u8>> = items
            .iter()
            .map(|(_, value)| serde_json::to_vec(value).map_err(|e| Error::Acorn(format!("Serialization error: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let id_ptrs: Vec<*const i8> = ids.iter().map(|s| s.as_ptr()).collect();
        let json_ptrs: Vec<*const u8> = jsons.iter().map(|v| v.as_ptr()).collect();
        let json_lens: Vec<usize> = jsons.iter().map(|v| v.len()).collect();

        let rc = unsafe {
            acorn_batch_stash(
                self.h,
                id_ptrs.as_ptr() as *mut *const i8,
                json_ptrs.as_ptr() as *mut *const u8,
                json_lens.as_ptr(),
                items.len(),
            )
        };

        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Retrieve multiple values by their IDs in a single operation.
    /// Returns a vector of Option<T> where None indicates the key was not found.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Data { value: i32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let keys = vec!["key1", "key2", "key3"];
    /// let results: Vec<Option<Data>> = tree.batch_crack(&keys)?;
    /// for (key, result) in keys.iter().zip(results.iter()) {
    ///     match result {
    ///         Some(data) => println!("{}: {:?}", key, data.value),
    ///         None => println!("{}: not found", key),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_crack<T: DeserializeOwned>(&self, ids: &[&str]) -> Result<Vec<Option<T>>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Prepare C-compatible arrays
        let id_cstrings: Vec<CString> = ids
            .iter()
            .map(|id| CString::new(*id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let id_ptrs: Vec<*const i8> = id_cstrings.iter().map(|s| s.as_ptr()).collect();

        let mut out_jsons: Vec<acorn_buf> = vec![acorn_buf { data: ptr::null_mut(), len: 0 }; ids.len()];
        let mut out_found: Vec<i32> = vec![0; ids.len()];

        let rc = unsafe {
            acorn_batch_crack(
                self.h,
                id_ptrs.as_ptr() as *mut *const i8,
                ids.len(),
                out_jsons.as_mut_ptr(),
                out_found.as_mut_ptr(),
            )
        };

        if rc != 0 {
            // Clean up any allocated buffers
            for buf in &mut out_jsons {
                if !buf.data.is_null() {
                    unsafe { acorn_free_buf(buf as *mut _) };
                }
            }
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        // Convert results to Rust types
        let mut results = Vec::with_capacity(ids.len());
        for i in 0..ids.len() {
            if out_found[i] == 0 {
                results.push(None);
            } else {
                let slice = unsafe { std::slice::from_raw_parts(out_jsons[i].data, out_jsons[i].len) };
                let value = serde_json::from_slice(slice).map_err(|e| Error::Acorn(e.to_string()))?;
                results.push(Some(value));
            }

            // Free the buffer
            if !out_jsons[i].data.is_null() {
                unsafe { acorn_free_buf(&mut out_jsons[i] as *mut _) };
            }
        }

        Ok(results)
    }

    /// Delete multiple items by their IDs in a single operation.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let mut tree = AcornTree::open("memory://")?;
    /// let keys_to_delete = vec!["key1", "key2", "key3"];
    /// tree.batch_delete(&keys_to_delete)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn batch_delete(&mut self, ids: &[&str]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        // Prepare C-compatible arrays
        let id_cstrings: Vec<CString> = ids
            .iter()
            .map(|id| CString::new(*id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e))))
            .collect::<Result<Vec<_>>>()?;

        let id_ptrs: Vec<*const i8> = id_cstrings.iter().map(|s| s.as_ptr()).collect();

        let rc = unsafe {
            acorn_batch_delete(
                self.h,
                id_ptrs.as_ptr() as *mut *const i8,
                ids.len(),
            )
        };

        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Start a LINQ-style query on this tree.
    /// Returns a query builder that supports filtering, ordering, and projection.
    ///
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// 
    /// // Query users older than 18, ordered by name
    /// let adults: Vec<User> = tree.query()
    ///     .where_condition(|user| user["age"].as_u64().unwrap_or(0) >= 18)
    ///     .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn query(&self) -> AcornQuery {
        AcornQuery::new(self.h)
    }

    /// Begin a new transaction for atomic multi-operation changes.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// 
    /// tx.stash("user1", &serde_json::json!({"name": "Alice", "age": 30}))?;
    /// tx.stash("user2", &serde_json::json!({"name": "Bob", "age": 25}))?;
    /// 
    /// if tx.commit()? {
    ///     println!("Transaction committed successfully");
    /// } else {
    ///     println!("Transaction failed to commit");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn begin_transaction(&self) -> Result<AcornTransaction> {
        let mut h: acorn_transaction_handle = 0;
        let rc = unsafe { acorn_begin_transaction(self.h, &mut h as *mut _) };
        if rc == 0 {
            Ok(AcornTransaction { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a new mesh coordinator for advanced synchronization.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_mesh() -> Result<AcornMesh> {
        let mut h: acorn_mesh_handle = 0;
        let rc = unsafe { acorn_mesh_create(&mut h as *mut _) };
        if rc == 0 {
            Ok(AcornMesh { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a peer-to-peer sync connection with another tree.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_p2p(local_tree: &AcornTree, remote_tree: &AcornTree) -> Result<AcornP2P> {
        let mut h: acorn_p2p_handle = 0;
        let rc = unsafe { acorn_p2p_create(local_tree.h, remote_tree.h, &mut h as *mut _) };
        if rc == 0 {
            Ok(AcornP2P { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornTree {
    fn drop(&mut self) { unsafe { acorn_close_tree(self.h); } }
}

/// Iterator over key-value pairs in an AcornTree.
/// The iterator holds a snapshot of the tree at the time it was created.
pub struct AcornIterator {
    h: acorn_iter_handle,
}

/// Transaction for atomic multi-operation changes.
/// Provides snapshot isolation and rollback capabilities.
pub struct AcornTransaction {
    h: acorn_transaction_handle,
}

/// Mesh coordinator for advanced synchronization across multiple trees.
/// Supports various network topologies like full mesh, ring, and star.
pub struct AcornMesh {
    h: acorn_mesh_handle,
}

/// Peer-to-peer synchronization connection between two trees.
/// Supports bidirectional, push-only, and pull-only sync modes.
pub struct AcornP2P {
    h: acorn_p2p_handle,
}

impl AcornTransaction {
    /// Store a value in the transaction.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.stash("user1", &serde_json::json!({"name": "Alice"}))?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn stash<T: Serialize>(&mut self, id: &str, value: &T) -> Result<()> {
        let json = serde_json::to_vec(value).map_err(|e| Error::Acorn(format!("Serialization error: {}", e)))?;
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let rc = unsafe { acorn_transaction_stash(self.h, idc.as_ptr(), json.as_ptr(), json.len()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Delete a value from the transaction.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.delete("user1")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn delete(&mut self, id: &str) -> Result<()> {
        let idc = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        let rc = unsafe { acorn_transaction_delete(self.h, idc.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Commit the transaction, applying all changes atomically.
    /// Returns true if the commit was successful, false if it failed.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.stash("user1", &serde_json::json!({"name": "Alice"}))?;
    /// 
    /// if tx.commit()? {
    ///     println!("Transaction committed successfully");
    /// } else {
    ///     println!("Transaction failed to commit");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn commit(&mut self) -> Result<bool> {
        let rc = unsafe { acorn_transaction_commit(self.h) };
        if rc == 0 {
            Ok(true)
        } else if rc == 1 {
            Ok(false) // Transaction failed to commit
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Rollback the transaction, discarding all changes.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mut tree = AcornTree::open("memory://")?;
    /// let mut tx = tree.begin_transaction()?;
    /// tx.stash("user1", &serde_json::json!({"name": "Alice"}))?;
    /// tx.rollback()?; // All changes are discarded
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn rollback(&mut self) -> Result<()> {
        let rc = unsafe { acorn_transaction_rollback(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornTransaction {
    fn drop(&mut self) {
        unsafe { acorn_transaction_close(self.h); }
    }
}

impl AcornMesh {
    /// Add a tree node to the mesh with the given ID.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// let tree = AcornTree::open("memory://")?;
    /// mesh.add_node("node1", &tree)?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn add_node(&self, node_id: &str, tree: &AcornTree) -> Result<()> {
        let idc = CString::new(node_id).map_err(|e| Error::Acorn(format!("Invalid node ID: {}", e)))?;
        let rc = unsafe { acorn_mesh_add_node(self.h, idc.as_ptr(), tree.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Connect two nodes in the mesh for bidirectional synchronization.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// mesh.add_node("node1", &tree1)?;
    /// mesh.add_node("node2", &tree2)?;
    /// mesh.connect_nodes("node1", "node2")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn connect_nodes(&self, node_a: &str, node_b: &str) -> Result<()> {
        let node_ac = CString::new(node_a).map_err(|e| Error::Acorn(format!("Invalid node A ID: {}", e)))?;
        let node_bc = CString::new(node_b).map_err(|e| Error::Acorn(format!("Invalid node B ID: {}", e)))?;
        let rc = unsafe { acorn_mesh_connect_nodes(self.h, node_ac.as_ptr(), node_bc.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a full mesh topology where every node connects to every other node.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Add nodes first...
    /// mesh.create_full_mesh()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_full_mesh(&self) -> Result<()> {
        let rc = unsafe { acorn_mesh_create_full_mesh(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a ring topology where each node connects to the next, and the last connects to the first.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Add nodes first...
    /// mesh.create_ring()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_ring(&self) -> Result<()> {
        let rc = unsafe { acorn_mesh_create_ring(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Create a star topology where all nodes connect to a central hub.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Add nodes first...
    /// mesh.create_star("hub")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn create_star(&self, hub_node_id: &str) -> Result<()> {
        let hubc = CString::new(hub_node_id).map_err(|e| Error::Acorn(format!("Invalid hub node ID: {}", e)))?;
        let rc = unsafe { acorn_mesh_create_star(self.h, hubc.as_ptr()) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize all nodes in the mesh.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let mesh = AcornTree::create_mesh()?;
    /// // Setup mesh topology...
    /// mesh.synchronize_all()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn synchronize_all(&self) -> Result<()> {
        let rc = unsafe { acorn_mesh_synchronize_all(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornMesh {
    fn drop(&mut self) {
        unsafe { acorn_mesh_close(self.h); }
    }
}

impl AcornP2P {
    /// Synchronize bidirectionally between the local and remote trees.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.sync_bidirectional()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sync_bidirectional(&self) -> Result<()> {
        let rc = unsafe { acorn_p2p_sync_bidirectional(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize by pushing changes from local to remote tree only.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.sync_push_only()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sync_push_only(&self) -> Result<()> {
        let rc = unsafe { acorn_p2p_sync_push_only(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Synchronize by pulling changes from remote to local tree only.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.sync_pull_only()?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn sync_pull_only(&self) -> Result<()> {
        let rc = unsafe { acorn_p2p_sync_pull_only(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set the synchronization mode.
    /// 
    /// # Arguments
    /// * `sync_mode` - 0=Bidirectional, 1=PushOnly, 2=PullOnly, 3=Disabled
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.set_sync_mode(1)?; // PushOnly
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_sync_mode(&self, sync_mode: i32) -> Result<()> {
        let rc = unsafe { acorn_p2p_set_sync_mode(self.h, sync_mode) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set the conflict resolution direction.
    /// 
    /// # Arguments
    /// * `conflict_direction` - 0=UseJudge, 1=PreferLocal, 2=PreferRemote
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use acorn::AcornTree;
    /// 
    /// let tree1 = AcornTree::open("memory://")?;
    /// let tree2 = AcornTree::open("memory://")?;
    /// let p2p = AcornTree::create_p2p(&tree1, &tree2)?;
    /// p2p.set_conflict_direction(1)?; // PreferLocal
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn set_conflict_direction(&self, conflict_direction: i32) -> Result<()> {
        let rc = unsafe { acorn_p2p_set_conflict_direction(self.h, conflict_direction) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornP2P {
    fn drop(&mut self) {
        unsafe { acorn_p2p_close(self.h); }
    }
}

impl AcornIterator {
    /// Get the next key-value pair. Returns None when iteration is complete.
    pub fn next<T: DeserializeOwned>(&mut self) -> Result<Option<(String, T)>> {
        let mut key_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let mut json_buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let mut done: i32 = 0;

        let rc = unsafe {
            acorn_iter_next(
                self.h,
                &mut key_buf as *mut _,
                &mut json_buf as *mut _,
                &mut done as *mut _,
            )
        };

        if rc != 0 {
            return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
        }

        if done != 0 {
            return Ok(None);
        }

        // Extract key
        let key_slice = unsafe { std::slice::from_raw_parts(key_buf.data, key_buf.len) };
        let key = String::from_utf8_lossy(key_slice).into_owned();

        // Extract and deserialize value
        let json_slice = unsafe { std::slice::from_raw_parts(json_buf.data, json_buf.len) };
        let value = serde_json::from_slice(json_slice).map_err(|e| Error::Acorn(e.to_string()))?;

        // Free buffers
        unsafe {
            acorn_free_buf(&mut key_buf as *mut _);
            acorn_free_buf(&mut json_buf as *mut _);
        }

        Ok(Some((key, value)))
    }

    /// Collect all remaining items into a Vec. This consumes the iterator.
    pub fn collect<T: DeserializeOwned>(&mut self) -> Result<Vec<(String, T)>> {
        let mut items = Vec::new();
        while let Some(item) = self.next()? {
            items.push(item);
        }
        Ok(items)
    }
}

impl Drop for AcornIterator {
    fn drop(&mut self) {
        unsafe { acorn_iter_close(self.h); }
    }
}

/// Subscription to tree changes. Automatically unsubscribes when dropped.
pub struct AcornSubscription {
    h: acorn_sub_handle,
    // Keep the callback alive by storing the boxed callback
    // The user_data pointer in the C subscription points to this
    _callback: Box<Box<dyn Fn(&str, &serde_json::Value) + Send>>,
}

impl AcornSubscription {
    fn new<F>(tree_h: acorn_tree_handle, callback: F) -> Result<Self>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        // Box the callback - this will be passed as user data to C
        let callback_box: Box<dyn Fn(&str, &serde_json::Value) + Send> = Box::new(callback);
        let user_data = Box::into_raw(Box::new(callback_box)) as *mut std::ffi::c_void;

        // Define the C callback wrapper
        unsafe extern "C" fn c_callback(
            key: *const std::os::raw::c_char,
            json: *const u8,
            len: usize,
            user: *mut std::ffi::c_void,
        ) {
            if user.is_null() {
                return;
            }

            // Reconstruct the callback from user data
            let callback_ptr = user as *const Box<dyn Fn(&str, &serde_json::Value) + Send>;
            let callback = &**callback_ptr;

            // Convert key to str
            let key_str = if key.is_null() {
                ""
            } else {
                std::ffi::CStr::from_ptr(key)
                    .to_str()
                    .unwrap_or("")
            };

            // Convert JSON bytes to serde_json::Value
            if !json.is_null() && len > 0 {
                let json_slice = std::slice::from_raw_parts(json, len);
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(json_slice) {
                    // Invoke the user callback
                    callback(key_str, &value);
                }
            }
        }

        let mut sub_h: acorn_sub_handle = 0;
        let rc = unsafe {
            acorn_subscribe(
                tree_h,
                Some(c_callback),
                user_data,
                &mut sub_h as *mut _,
            )
        };

        if rc == 0 {
            // Reconstruct the Box to store in Self (we own it now)
            // user_data points to Box<Box<dyn Fn...>>
            let callback_box = unsafe { Box::from_raw(user_data as *mut Box<dyn Fn(&str, &serde_json::Value) + Send>) };
            Ok(Self {
                h: sub_h,
                _callback: callback_box,
            })
        } else {
            // Clean up on error
            unsafe {
                let _ = Box::from_raw(user_data as *mut Box<dyn Fn(&str, &serde_json::Value) + Send>);
            }
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornSubscription {
    fn drop(&mut self) {
        unsafe {
            acorn_unsubscribe(self.h);
        }
    }
}

/// LINQ-style query builder for AcornTree.
/// Provides fluent API for filtering, ordering, and projecting tree data.
pub struct AcornQuery {
    tree_h: acorn_tree_handle,
}

impl AcornQuery {
    fn new(tree_h: acorn_tree_handle) -> Self {
        Self { tree_h }
    }

    /// Filter items by a condition on the payload.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let adults: Vec<User> = tree.query()
    ///     .where_condition(|user| user["age"].as_u64().unwrap_or(0) >= 18)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn where_condition<F>(self, predicate: F) -> AcornQueryWhere<F>
    where
        F: Fn(&serde_json::Value) -> bool,
    {
        AcornQueryWhere {
            tree_h: self.tree_h,
            predicate,
        }
    }

    /// Order items by a key selector.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let users: Vec<User> = tree.query()
    ///     .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn order_by<F>(self, key_selector: F) -> AcornQueryOrderBy<F>
    where
        F: Fn(&serde_json::Value) -> String,
    {
        AcornQueryOrderBy {
            tree_h: self.tree_h,
            key_selector,
            descending: false,
        }
    }

    /// Order items by a key selector (descending).
    pub fn order_by_descending<F>(self, key_selector: F) -> AcornQueryOrderBy<F>
    where
        F: Fn(&serde_json::Value) -> String,
    {
        AcornQueryOrderBy {
            tree_h: self.tree_h,
            key_selector,
            descending: true,
        }
    }

    /// Take only the first N items.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let top_users: Vec<User> = tree.query()
    ///     .order_by(|user| user["name"].as_str().unwrap_or("").to_string())
    ///     .take(10)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn take(self, count: usize) -> AcornQueryTake {
        AcornQueryTake {
            tree_h: self.tree_h,
            count,
        }
    }

    /// Skip the first N items.
    pub fn skip(self, count: usize) -> AcornQuerySkip {
        AcornQuerySkip {
            tree_h: self.tree_h,
            count,
        }
    }

    /// Execute the query and collect all results into a Vec.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { name: String, age: u32 }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let users: Vec<User> = tree.query().collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, we'll implement a simple version that gets all items
        // In a full implementation, we'd need to implement the query execution
        // through the FFI layer
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            results.push(value);
        }
        Ok(results)
    }

    /// Execute the query and return the first result, or None if empty.
    pub fn first<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        if let Some((_, value)) = iter.next()? {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Count the number of items that would be returned by this query.
    pub fn count(&self) -> Result<usize> {
        unsafe {
            let mut count: usize = 0;
            let rc = acorn_count(self.tree_h, &mut count as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            Ok(count)
        }
    }

    /// Check if any items match this query.
    pub fn any(&self) -> Result<bool> {
        Ok(self.count()? > 0)
    }
}

/// Query builder with WHERE condition applied.
pub struct AcornQueryWhere<F> {
    tree_h: acorn_tree_handle,
    predicate: F,
}

impl<F> AcornQueryWhere<F>
where
    F: Fn(&serde_json::Value) -> bool,
{
    /// Apply ordering to the filtered results.
    pub fn order_by<G>(self, key_selector: G) -> AcornQueryWhereOrderBy<F, G>
    where
        G: Fn(&serde_json::Value) -> String,
    {
        AcornQueryWhereOrderBy {
            tree_h: self.tree_h,
            predicate: self.predicate,
            key_selector,
            descending: false,
        }
    }

    /// Apply ordering to the filtered results (descending).
    pub fn order_by_descending<G>(self, key_selector: G) -> AcornQueryWhereOrderBy<F, G>
    where
        G: Fn(&serde_json::Value) -> String,
    {
        AcornQueryWhereOrderBy {
            tree_h: self.tree_h,
            predicate: self.predicate,
            key_selector,
            descending: true,
        }
    }

    /// Take only the first N items from filtered results.
    pub fn take(self, count: usize) -> AcornQueryWhereTake<F> {
        AcornQueryWhereTake {
            tree_h: self.tree_h,
            predicate: self.predicate,
            count,
        }
    }

    /// Skip the first N items from filtered results.
    pub fn skip(self, count: usize) -> AcornQueryWhereSkip<F> {
        AcornQueryWhereSkip {
            tree_h: self.tree_h,
            predicate: self.predicate,
            count,
        }
    }

    /// Execute the query and collect all filtered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items and filtering in Rust
        // In a full implementation, this would be done in the C# layer
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                results.push(typed_value);
            }
        }
        Ok(results)
    }

    /// Execute the query and return the first filtered result.
    pub fn first<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                return Ok(Some(typed_value));
            }
        }
        Ok(None)
    }

    /// Count the number of filtered results.
    pub fn count(&self) -> Result<usize> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut count = 0;
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Check if any items match the filter.
    pub fn any(&self) -> Result<bool> {
        Ok(self.count()? > 0)
    }
}

/// Query builder with WHERE condition and ORDER BY applied.
pub struct AcornQueryWhereOrderBy<F, G> {
    tree_h: acorn_tree_handle,
    predicate: F,
    key_selector: G,
    descending: bool,
}

impl<F, G> AcornQueryWhereOrderBy<F, G>
where
    F: Fn(&serde_json::Value) -> bool,
    G: Fn(&serde_json::Value) -> String,
{
    /// Take only the first N items from filtered, ordered results.
    pub fn take(self, count: usize) -> AcornQueryWhereOrderByTake<F, G> {
        AcornQueryWhereOrderByTake {
            tree_h: self.tree_h,
            predicate: self.predicate,
            key_selector: self.key_selector,
            descending: self.descending,
            count,
        }
    }

    /// Execute the query and collect all filtered, ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // Get all filtered results first
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut filtered_items = Vec::new();
        while let Some((key, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let key_str = (self.key_selector)(&value);
                filtered_items.push((key_str, value));
            }
        }

        // Sort by the key selector
        filtered_items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in filtered_items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with WHERE condition and TAKE applied.
pub struct AcornQueryWhereTake<F> {
    tree_h: acorn_tree_handle,
    predicate: F,
    count: usize,
}

impl<F> AcornQueryWhereTake<F>
where
    F: Fn(&serde_json::Value) -> bool,
{
    /// Execute the query and collect up to N filtered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                results.push(typed_value);
                if results.len() >= self.count {
                    break;
                }
            }
        }
        Ok(results)
    }
}

/// Query builder with WHERE condition and SKIP applied.
pub struct AcornQueryWhereSkip<F> {
    tree_h: acorn_tree_handle,
    predicate: F,
    count: usize,
}

impl<F> AcornQueryWhereSkip<F>
where
    F: Fn(&serde_json::Value) -> bool,
{
    /// Execute the query and collect filtered results after skipping N items.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        let mut skipped = 0;
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                if skipped < self.count {
                    skipped += 1;
                } else {
                    let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
                    results.push(typed_value);
                }
            }
        }
        Ok(results)
    }
}

/// Query builder with ORDER BY applied.
pub struct AcornQueryOrderBy<F> {
    tree_h: acorn_tree_handle,
    key_selector: F,
    descending: bool,
}

impl<F> AcornQueryOrderBy<F>
where
    F: Fn(&serde_json::Value) -> String,
{
    /// Take only the first N items from ordered results.
    pub fn take(self, count: usize) -> AcornQueryOrderByTake<F> {
        AcornQueryOrderByTake {
            tree_h: self.tree_h,
            key_selector: self.key_selector,
            descending: self.descending,
            count,
        }
    }

    /// Skip the first N items from ordered results.
    pub fn skip(self, count: usize) -> AcornQueryOrderBySkip<F> {
        AcornQueryOrderBySkip {
            tree_h: self.tree_h,
            key_selector: self.key_selector,
            descending: self.descending,
            count,
        }
    }

    /// Execute the query and collect all ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            let key_str = (self.key_selector)(&value);
            items.push((key_str, value));
        }

        // Sort by the key selector
        items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with TAKE applied.
pub struct AcornQueryTake {
    tree_h: acorn_tree_handle,
    count: usize,
}

impl AcornQueryTake {
    /// Execute the query and collect up to N results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        while let Some((_, value)) = iter.next()? {
            results.push(value);
            if results.len() >= self.count {
                break;
            }
        }
        Ok(results)
    }
}

/// Query builder with SKIP applied.
pub struct AcornQuerySkip {
    tree_h: acorn_tree_handle,
    count: usize,
}

impl AcornQuerySkip {
    /// Execute the query and collect results after skipping N items.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut results = Vec::new();
        let mut skipped = 0;
        while let Some((_, value)) = iter.next()? {
            if skipped < self.count {
                skipped += 1;
            } else {
                results.push(value);
            }
        }
        Ok(results)
    }
}

/// Query builder with ORDER BY and TAKE applied.
pub struct AcornQueryOrderByTake<F> {
    tree_h: acorn_tree_handle,
    key_selector: F,
    descending: bool,
    count: usize,
}

impl<F> AcornQueryOrderByTake<F>
where
    F: Fn(&serde_json::Value) -> String,
{
    /// Execute the query and collect up to N ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            let key_str = (self.key_selector)(&value);
            items.push((key_str, value));
        }

        // Sort by the key selector
        items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Take only the first N items
        items.truncate(self.count);

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with ORDER BY and SKIP applied.
pub struct AcornQueryOrderBySkip<F> {
    tree_h: acorn_tree_handle,
    key_selector: F,
    descending: bool,
    count: usize,
}

impl<F> AcornQueryOrderBySkip<F>
where
    F: Fn(&serde_json::Value) -> String,
{
    /// Execute the query and collect ordered results after skipping N items.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            let key_str = (self.key_selector)(&value);
            items.push((key_str, value));
        }

        // Sort by the key selector
        items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Skip the first N items
        if self.count < items.len() {
            items.drain(0..self.count);
        } else {
            items.clear();
        }

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

/// Query builder with WHERE condition, ORDER BY, and TAKE applied.
pub struct AcornQueryWhereOrderByTake<F, G> {
    tree_h: acorn_tree_handle,
    predicate: F,
    key_selector: G,
    descending: bool,
    count: usize,
}

impl<F, G> AcornQueryWhereOrderByTake<F, G>
where
    F: Fn(&serde_json::Value) -> bool,
    G: Fn(&serde_json::Value) -> String,
{
    /// Execute the query and collect up to N filtered, ordered results.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut filtered_items = Vec::new();
        while let Some((_, value)) = iter.next::<serde_json::Value>()? {
            if (self.predicate)(&value) {
                let key_str = (self.key_selector)(&value);
                filtered_items.push((key_str, value));
            }
        }

        // Sort by the key selector
        filtered_items.sort_by(|a, b| {
            if self.descending {
                b.0.cmp(&a.0)
            } else {
                a.0.cmp(&b.0)
            }
        });

        // Take only the first N items
        filtered_items.truncate(self.count);

        // Convert to typed results
        let mut results = Vec::new();
        for (_, value) in filtered_items {
            let typed_value: T = serde_json::from_value(value).map_err(|e| Error::Acorn(e.to_string()))?;
            results.push(typed_value);
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct TestData {
        id: String,
        value: i32,
        name: String,
    }

    #[test]
    fn test_error_types() {
        // Test error types exist and can be created
        let _not_found = Error::NotFound;
        let _acorn_error = Error::Acorn("test error".to_string());
    }

    #[test]
    fn test_result_type() {
        // Test Result type works
        let ok_result: Result<String> = Ok("test".to_string());
        let err_result: Result<String> = Err(Error::NotFound);
        
        assert!(ok_result.is_ok());
        assert!(err_result.is_err());
    }

    #[test]
    fn test_serialization() {
        // Test that our test data can be serialized/deserialized
        let data = TestData {
            id: "test-1".to_string(),
            value: 42,
            name: "Test Item".to_string(),
        };

        let json = serde_json::to_string(&data).unwrap();
        let deserialized: TestData = serde_json::from_str(&json).unwrap();
        
        assert_eq!(data, deserialized);
    }

    #[test]
    fn test_cstring_creation() {
        // Test CString creation with various inputs
        let valid_id = "test-id";
        let cstring = CString::new(valid_id).unwrap();
        assert_eq!(cstring.to_string_lossy(), valid_id);

        // Test with null bytes (should fail)
        let invalid_id = "test\0id";
        let result = CString::new(invalid_id);
        assert!(result.is_err());
    }

    // Integration tests would go here, but they require the actual shim to be built
    // and the ACORN_SHIM_DIR environment variable to be set
    #[cfg(feature = "integration-tests")]
    mod integration_tests {
        use super::*;

        #[test]
        fn test_tree_lifecycle() {
            // This test would require the shim to be built and available
            // let mut tree = AcornTree::open("file://./test_db").unwrap();
            // 
            // let test_data = TestData {
            //     id: "test-1".to_string(),
            //     value: 42,
            //     name: "Test Item".to_string(),
            // };
            // 
            // // Test stash
            // tree.stash("test-1", &test_data).unwrap();
            // 
            // // Test crack
            // let retrieved: TestData = tree.crack("test-1").unwrap();
            // assert_eq!(test_data, retrieved);
            // 
            // // Test not found
            // let result: Result<TestData> = tree.crack("nonexistent");
            // assert!(matches!(result, Err(Error::NotFound)));
        }
    }
}
