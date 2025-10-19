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
}

impl Drop for AcornTree {
    fn drop(&mut self) { unsafe { acorn_close_tree(self.h); } }
}

/// Iterator over key-value pairs in an AcornTree.
/// The iterator holds a snapshot of the tree at the time it was created.
pub struct AcornIterator {
    h: acorn_iter_handle,
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
