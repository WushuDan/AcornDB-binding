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
