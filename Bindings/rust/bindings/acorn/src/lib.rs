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
}

impl Drop for AcornTree {
    fn drop(&mut self) { unsafe { acorn_close_tree(self.h); } }
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
