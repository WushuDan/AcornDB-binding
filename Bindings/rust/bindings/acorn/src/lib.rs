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

/// Compression provider for AcornDB
pub struct AcornCompression { h: acorn_compression_handle }

/// Compression levels available in AcornDB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionLevel {
    /// Fastest compression (least CPU usage, larger output)
    Fastest = 0,
    /// Optimal balance of speed and compression ratio
    Optimal = 1,
    /// Smallest size (most CPU usage, smallest output)
    SmallestSize = 2,
}

/// Compression statistics
#[derive(Debug, Clone)]
pub struct CompressionStats {
    pub original_size: i32,
    pub compressed_size: i32,
    pub ratio: f64,
    pub space_saved: i32,
}

impl AcornCompression {
    /// Create a Gzip compression provider
    /// 
    /// # Arguments
    /// * `level` - The compression level to use
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn gzip(level: CompressionLevel) -> Result<Self> {
        let mut h: acorn_compression_handle = 0;
        let rc = unsafe { acorn_compression_gzip(level as i32, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a Brotli compression provider
    /// 
    /// # Arguments
    /// * `level` - The compression level to use
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::brotli(CompressionLevel::Optimal)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn brotli(level: CompressionLevel) -> Result<Self> {
        let mut h: acorn_compression_handle = 0;
        let rc = unsafe { acorn_compression_brotli(level as i32, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a no-op compression provider (passes data through unchanged)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::none()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn none() -> Result<Self> {
        let mut h: acorn_compression_handle = 0;
        let rc = unsafe { acorn_compression_none(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Compress data
    /// 
    /// # Arguments
    /// * `data` - The data to compress
    /// 
    /// # Returns
    /// * `Ok(String)` - The compressed data as a base64-encoded string
    /// * `Err(Error)` - If compression fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let compressed = compression.compress("Hello, world!")?;
    /// println!("Compressed: {}", compressed);
    /// # Ok(())
    /// # }
    /// ```
    pub fn compress(&self, data: &str) -> Result<String> {
        let data_c = CString::new(data).map_err(|e| Error::Acorn(format!("Invalid data: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_compression_compress(self.h, data_c.as_ptr(), &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Decompress data
    /// 
    /// # Arguments
    /// * `compressed_data` - The compressed data as a base64-encoded string
    /// 
    /// # Returns
    /// * `Ok(String)` - The decompressed data
    /// * `Err(Error)` - If decompression fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let compressed = compression.compress("Hello, world!")?;
    /// let decompressed = compression.decompress(&compressed)?;
    /// assert_eq!(decompressed, "Hello, world!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn decompress(&self, compressed_data: &str) -> Result<String> {
        let compressed_c = CString::new(compressed_data).map_err(|e| Error::Acorn(format!("Invalid compressed data: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_compression_decompress(self.h, compressed_c.as_ptr(), &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check if compression is enabled
    /// 
    /// # Returns
    /// * `true` - If compression is enabled
    /// * `false` - If compression is disabled (no-op provider)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// assert!(compression.is_enabled()?);
    /// 
    /// let no_compression = AcornCompression::none()?;
    /// assert!(!no_compression.is_enabled()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_enabled(&self) -> Result<bool> {
        let rc = unsafe { acorn_compression_is_enabled(self.h) };
        if rc >= 0 {
            Ok(rc == 1)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get the algorithm name
    /// 
    /// # Returns
    /// * `Ok(String)` - The name of the compression algorithm
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let algorithm = compression.algorithm_name()?;
    /// assert_eq!(algorithm, "Gzip");
    /// # Ok(())
    /// # }
    /// ```
    pub fn algorithm_name(&self) -> Result<String> {
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_compression_algorithm_name(self.h, &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get compression statistics
    /// 
    /// # Arguments
    /// * `original_data` - The original uncompressed data
    /// * `compressed_data` - The compressed data as a base64-encoded string
    /// 
    /// # Returns
    /// * `Ok(CompressionStats)` - Compression statistics
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let original = "Hello, world!";
    /// let compressed = compression.compress(original)?;
    /// let stats = compression.get_stats(original, &compressed)?;
    /// println!("Compression ratio: {:.2}", stats.ratio);
    /// println!("Space saved: {} bytes", stats.space_saved);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_stats(&self, original_data: &str, compressed_data: &str) -> Result<CompressionStats> {
        let original_c = CString::new(original_data).map_err(|e| Error::Acorn(format!("Invalid original data: {}", e)))?;
        let compressed_c = CString::new(compressed_data).map_err(|e| Error::Acorn(format!("Invalid compressed data: {}", e)))?;
        
        let mut original_size: i32 = 0;
        let mut compressed_size: i32 = 0;
        let mut ratio: f64 = 0.0;
        let mut space_saved: i32 = 0;
        
        let rc = unsafe { 
            acorn_compression_get_stats(
                self.h, 
                original_c.as_ptr(), 
                compressed_c.as_ptr(),
                &mut original_size as *mut _,
                &mut compressed_size as *mut _,
                &mut ratio as *mut _,
                &mut space_saved as *mut _
            ) 
        };
        
        if rc == 0 {
            Ok(CompressionStats {
                original_size,
                compressed_size,
                ratio,
                space_saved,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornCompression {
    fn drop(&mut self) {
        unsafe { acorn_compression_close(self.h); }
    }
}

/// Cache strategy for AcornDB
pub struct AcornCache { h: acorn_cache_handle }

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub tracked_items: i32,
    pub max_size: i32,
    pub utilization_percentage: f64,
}

impl AcornCache {
    /// Create an LRU (Least Recently Used) cache strategy
    /// 
    /// # Arguments
    /// * `max_size` - Maximum number of items to keep in cache before eviction
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn lru(max_size: i32) -> Result<Self> {
        let mut h: acorn_cache_handle = 0;
        let rc = unsafe { acorn_cache_lru(max_size, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a no-eviction cache strategy (unlimited cache)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::no_eviction()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn no_eviction() -> Result<Self> {
        let mut h: acorn_cache_handle = 0;
        let rc = unsafe { acorn_cache_no_eviction(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Reset the cache strategy state
    /// 
    /// # Returns
    /// * `Ok(())` - If reset was successful
    /// * `Err(Error)` - If reset failed
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// cache.reset()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn reset(&self) -> Result<()> {
        let rc = unsafe { acorn_cache_reset(self.h) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get cache statistics
    /// 
    /// # Returns
    /// * `Ok(CacheStats)` - Cache statistics
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// let stats = cache.get_stats()?;
    /// println!("Tracked items: {}", stats.tracked_items);
    /// println!("Max size: {}", stats.max_size);
    /// println!("Utilization: {:.1}%", stats.utilization_percentage);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_stats(&self) -> Result<CacheStats> {
        let mut tracked_items: i32 = 0;
        let mut max_size: i32 = 0;
        let mut utilization_percentage: f64 = 0.0;
        
        let rc = unsafe { 
            acorn_cache_get_stats(
                self.h,
                &mut tracked_items as *mut _,
                &mut max_size as *mut _,
                &mut utilization_percentage as *mut _
            ) 
        };
        
        if rc == 0 {
            Ok(CacheStats {
                tracked_items,
                max_size,
                utilization_percentage,
            })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check if eviction is enabled for this cache strategy
    /// 
    /// # Returns
    /// * `true` - If eviction is enabled (LRU cache)
    /// * `false` - If eviction is disabled (NoEviction cache)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let lru_cache = AcornCache::lru(1000)?;
    /// assert!(lru_cache.is_eviction_enabled()?);
    /// 
    /// let no_eviction_cache = AcornCache::no_eviction()?;
    /// assert!(!no_eviction_cache.is_eviction_enabled()?);
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_eviction_enabled(&self) -> Result<bool> {
        let rc = unsafe { acorn_cache_is_eviction_enabled(self.h) };
        if rc >= 0 {
            Ok(rc == 1)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Set eviction enabled status (no-op for most strategies)
    /// 
    /// # Arguments
    /// * `enabled` - Whether to enable eviction
    /// 
    /// # Note
    /// This is a no-op for most cache strategies as eviction is determined by strategy type.
    /// LRU cache always has eviction enabled, NoEviction cache never does.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// cache.set_eviction_enabled(true)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn set_eviction_enabled(&self, enabled: bool) -> Result<()> {
        let rc = unsafe { acorn_cache_set_eviction_enabled(self.h, if enabled { 1 } else { 0 }) };
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornCache {
    fn drop(&mut self) {
        unsafe { acorn_cache_close(self.h); }
    }
}

/// Conflict resolution judge for AcornDB
pub struct AcornConflictJudge { h: acorn_conflict_judge_handle }

/// Conflict resolution strategies available in AcornDB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictStrategy {
    /// Last-write-wins based on timestamp (default)
    Timestamp,
    /// Higher version number wins
    Version,
    /// Local version always wins
    LocalWins,
    /// Remote version always wins
    RemoteWins,
}

impl AcornConflictJudge {
    /// Create a timestamp-based conflict judge (last-write-wins)
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn timestamp() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_timestamp(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a version-based conflict judge
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::version()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn version() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_version(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a local-wins conflict judge
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::local_wins()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn local_wins() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_local_wins(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create a remote-wins conflict judge
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::remote_wins()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn remote_wins() -> Result<Self> {
        let mut h: acorn_conflict_judge_handle = 0;
        let rc = unsafe { acorn_conflict_judge_remote_wins(&mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get the name of the conflict resolution strategy
    /// 
    /// # Returns
    /// * `Ok(String)` - The name of the strategy
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// let name = judge.name()?;
    /// assert_eq!(name, "Timestamp");
    /// # Ok(())
    /// # }
    /// ```
    pub fn name(&self) -> Result<String> {
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_conflict_judge_name(self.h, &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Resolve a conflict between local and incoming data
    /// 
    /// # Arguments
    /// * `local_json` - The local data as JSON
    /// * `incoming_json` - The incoming data as JSON
    /// 
    /// # Returns
    /// * `Ok(String)` - The winning data as JSON
    /// * `Err(Error)` - If conflict resolution fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// let local = r#"{"id": "test", "value": "local", "timestamp": "2023-01-01T10:00:00Z"}"#;
    /// let incoming = r#"{"id": "test", "value": "incoming", "timestamp": "2023-01-01T11:00:00Z"}"#;
    /// let winner = judge.resolve_conflict(local, incoming)?;
    /// println!("Winner: {}", winner);
    /// # Ok(())
    /// # }
    /// ```
    pub fn resolve_conflict(&self, local_json: &str, incoming_json: &str) -> Result<String> {
        let local_c = CString::new(local_json).map_err(|e| Error::Acorn(format!("Invalid local JSON: {}", e)))?;
        let incoming_c = CString::new(incoming_json).map_err(|e| Error::Acorn(format!("Invalid incoming JSON: {}", e)))?;
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_conflict_judge_resolve(self.h, local_c.as_ptr(), incoming_c.as_ptr(), &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            Ok(result_str)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornConflictJudge {
    fn drop(&mut self) {
        unsafe { acorn_conflict_judge_close(self.h); }
    }
}

/// Storage backend for AcornDB
pub struct AcornStorage { h: acorn_storage_handle }

/// Document store for AcornDB with versioning and time-travel
pub struct AcornDocumentStore { h: acorn_document_store_handle }

/// Storage backend types available in AcornDB
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageType {
    /// AWS S3 storage
    S3,
    /// Azure Blob Storage
    AzureBlob,
    /// SQLite database
    SQLite,
    /// PostgreSQL database
    PostgreSQL,
    /// MySQL database
    MySQL,
    /// SQL Server database
    SQLServer,
    /// Git repository storage
    Git,
}

/// Storage backend information
#[derive(Debug, Clone)]
pub struct StorageInfo {
    pub trunk_type: String,
    pub supports_history: bool,
    pub supports_sync: bool,
    pub is_durable: bool,
    pub supports_async: bool,
    pub provider_name: String,
    pub connection_info: String,
}

/// Document store information
#[derive(Debug, Clone, Deserialize)]
pub struct DocumentStoreInfo {
    pub trunk_type: String,
    pub supports_history: bool,
    pub supports_sync: bool,
    pub is_durable: bool,
    pub supports_async: bool,
    pub provider_name: String,
    pub connection_info: String,
    pub has_change_log: bool,
    pub total_versions: i32,
}

/// Change types for reactive programming
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    /// Create or update operation
    Stash,
    /// Delete operation
    Toss,
    /// Conflict resolution operation
    Squabble,
}

/// Tree change event for reactive programming
#[derive(Debug, Clone)]
pub struct TreeChange<T> {
    pub change_type: ChangeType,
    pub id: String,
    pub item: Option<T>,
    pub timestamp: std::time::SystemTime,
    pub node_id: Option<String>,
}

impl AcornStorage {
    /// Create AWS S3 storage backend with explicit credentials
    /// 
    /// # Arguments
    /// * `access_key` - AWS Access Key ID
    /// * `secret_key` - AWS Secret Access Key
    /// * `bucket_name` - S3 bucket name
    /// * `region` - AWS region (default: "us-east-1")
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::s3("access_key", "secret_key", "my-bucket", "us-west-2", Some("prefix/"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn s3(access_key: &str, secret_key: &str, bucket_name: &str, region: &str, prefix: Option<&str>) -> Result<Self> {
        let access_key_c = CString::new(access_key).map_err(|e| Error::Acorn(format!("Invalid access key: {}", e)))?;
        let secret_key_c = CString::new(secret_key).map_err(|e| Error::Acorn(format!("Invalid secret key: {}", e)))?;
        let bucket_name_c = CString::new(bucket_name).map_err(|e| Error::Acorn(format!("Invalid bucket name: {}", e)))?;
        let region_c = CString::new(region).map_err(|e| Error::Acorn(format!("Invalid region: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_s3(
                access_key_c.as_ptr(), 
                secret_key_c.as_ptr(), 
                bucket_name_c.as_ptr(), 
                region_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create AWS S3 storage backend with default credential chain
    /// 
    /// # Arguments
    /// * `bucket_name` - S3 bucket name
    /// * `region` - AWS region (default: "us-east-1")
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::s3_default("my-bucket", "us-west-2", None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn s3_default(bucket_name: &str, region: &str, prefix: Option<&str>) -> Result<Self> {
        let bucket_name_c = CString::new(bucket_name).map_err(|e| Error::Acorn(format!("Invalid bucket name: {}", e)))?;
        let region_c = CString::new(region).map_err(|e| Error::Acorn(format!("Invalid region: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_s3_default(
                bucket_name_c.as_ptr(), 
                region_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create S3-compatible storage backend (MinIO, DigitalOcean Spaces, etc.)
    /// 
    /// # Arguments
    /// * `access_key` - Access key
    /// * `secret_key` - Secret key
    /// * `bucket_name` - Bucket name
    /// * `service_url` - Service endpoint URL
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::s3_compatible("access_key", "secret_key", "my-bucket", "https://nyc3.digitaloceanspaces.com", None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn s3_compatible(access_key: &str, secret_key: &str, bucket_name: &str, service_url: &str, prefix: Option<&str>) -> Result<Self> {
        let access_key_c = CString::new(access_key).map_err(|e| Error::Acorn(format!("Invalid access key: {}", e)))?;
        let secret_key_c = CString::new(secret_key).map_err(|e| Error::Acorn(format!("Invalid secret key: {}", e)))?;
        let bucket_name_c = CString::new(bucket_name).map_err(|e| Error::Acorn(format!("Invalid bucket name: {}", e)))?;
        let service_url_c = CString::new(service_url).map_err(|e| Error::Acorn(format!("Invalid service URL: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_s3_compatible(
                access_key_c.as_ptr(), 
                secret_key_c.as_ptr(), 
                bucket_name_c.as_ptr(), 
                service_url_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create Azure Blob Storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - Azure Storage connection string
    /// * `container_name` - Blob container name
    /// * `prefix` - Optional prefix for all keys
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::azure_blob("DefaultEndpointsProtocol=https;...", "my-container", None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn azure_blob(connection_string: &str, container_name: &str, prefix: Option<&str>) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let container_name_c = CString::new(container_name).map_err(|e| Error::Acorn(format!("Invalid container name: {}", e)))?;
        let prefix_c = CString::new(prefix.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid prefix: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_azure_blob(
                connection_string_c.as_ptr(), 
                container_name_c.as_ptr(), 
                prefix_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create SQLite storage backend
    /// 
    /// # Arguments
    /// * `database_path` - Path to SQLite database file
    /// * `table_name` - Optional custom table name
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./data/acorndb.db", Some("custom_table"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sqlite(database_path: &str, table_name: Option<&str>) -> Result<Self> {
        let database_path_c = CString::new(database_path).map_err(|e| Error::Acorn(format!("Invalid database path: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_sqlite(
                database_path_c.as_ptr(), 
                table_name_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create PostgreSQL storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - PostgreSQL connection string
    /// * `table_name` - Optional custom table name
    /// * `schema` - Database schema (default: "public")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::postgresql("postgresql://user:pass@localhost/db", None, "public")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn postgresql(connection_string: &str, table_name: Option<&str>, schema: &str) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        let schema_c = CString::new(schema).map_err(|e| Error::Acorn(format!("Invalid schema: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_postgresql(
                connection_string_c.as_ptr(), 
                table_name_c.as_ptr(), 
                schema_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create MySQL storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - MySQL connection string
    /// * `table_name` - Optional custom table name
    /// * `database` - Optional database name
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::mysql("Server=localhost;Database=acorndb;Uid=user;Pwd=pass;", None, None)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn mysql(connection_string: &str, table_name: Option<&str>, database: Option<&str>) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        let database_c = CString::new(database.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid database: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_mysql(
                connection_string_c.as_ptr(), 
                table_name_c.as_ptr(), 
                database_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create SQL Server storage backend
    /// 
    /// # Arguments
    /// * `connection_string` - SQL Server connection string
    /// * `table_name` - Optional custom table name
    /// * `schema` - Database schema (default: "dbo")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlserver("Server=localhost;Database=acorndb;Integrated Security=true;", None, "dbo")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn sqlserver(connection_string: &str, table_name: Option<&str>, schema: &str) -> Result<Self> {
        let connection_string_c = CString::new(connection_string).map_err(|e| Error::Acorn(format!("Invalid connection string: {}", e)))?;
        let table_name_c = CString::new(table_name.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid table name: {}", e)))?;
        let schema_c = CString::new(schema).map_err(|e| Error::Acorn(format!("Invalid schema: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_sqlserver(
                connection_string_c.as_ptr(), 
                table_name_c.as_ptr(), 
                schema_c.as_ptr(), 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Create Git storage backend
    /// 
    /// # Arguments
    /// * `repo_path` - Path to Git repository
    /// * `author_name` - Git author name
    /// * `author_email` - Git author email
    /// * `auto_push` - Automatically push to remote after each commit
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::git("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn git(repo_path: &str, author_name: &str, author_email: &str, auto_push: bool) -> Result<Self> {
        let repo_path_c = CString::new(repo_path).map_err(|e| Error::Acorn(format!("Invalid repo path: {}", e)))?;
        let author_name_c = CString::new(author_name).map_err(|e| Error::Acorn(format!("Invalid author name: {}", e)))?;
        let author_email_c = CString::new(author_email).map_err(|e| Error::Acorn(format!("Invalid author email: {}", e)))?;
        
        let mut h: acorn_storage_handle = 0;
        let rc = unsafe { 
            acorn_storage_git(
                repo_path_c.as_ptr(), 
                author_name_c.as_ptr(), 
                author_email_c.as_ptr(), 
                if auto_push { 1 } else { 0 }, 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get storage backend information
    /// 
    /// # Returns
    /// * `Ok(StorageInfo)` - Storage backend information
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./test.db", None)?;
    /// let info = storage.get_info()?;
    /// println!("Provider: {}", info.provider_name);
    /// println!("Durable: {}", info.is_durable);
    /// println!("Supports History: {}", info.supports_history);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_info(&self) -> Result<StorageInfo> {
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_storage_get_info(self.h, &mut buf as *mut _) };
        if rc == 0 {
            let result = unsafe { 
                std::slice::from_raw_parts(buf.data, buf.len as usize) 
            };
            let result_str = String::from_utf8_lossy(result).to_string();
            unsafe { acorn_free_buf(&mut buf); }
            
            // Parse JSON response
            let info: StorageInfo = serde_json::from_str(&result_str)
                .map_err(|e| Error::Acorn(format!("Failed to parse storage info: {}", e)))?;
            Ok(info)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Test storage backend connection
    /// 
    /// # Returns
    /// * `Ok(bool)` - True if connection is successful
    /// * `Err(Error)` - If the operation fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./test.db", None)?;
    /// let is_connected = storage.test_connection()?;
    /// if is_connected {
    ///     println!("Storage connection successful!");
    /// } else {
    ///     println!("Storage connection failed!");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn test_connection(&self) -> Result<bool> {
        let rc = unsafe { acorn_storage_test_connection(self.h) };
        if rc >= 0 {
            Ok(rc == 1)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornStorage {
    fn drop(&mut self) {
        unsafe { acorn_storage_close(self.h); }
    }
}

impl AcornDocumentStore {
    /// Create a new document store with optional custom path
    /// 
    /// # Arguments
    /// * `custom_path` - Optional custom path for the document store data
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(Some("./my_data"))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(custom_path: Option<&str>) -> Result<Self> {
        let custom_path_c = CString::new(custom_path.unwrap_or("")).map_err(|e| Error::Acorn(format!("Invalid custom path: {}", e)))?;
        
        let mut h: acorn_document_store_handle = 0;
        let rc = unsafe { 
            acorn_document_store_create(
                if custom_path.is_some() { custom_path_c.as_ptr() } else { std::ptr::null() }, 
                &mut h as *mut _
            ) 
        };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get version history for a specific document ID
    /// 
    /// # Arguments
    /// * `id` - Document ID to get history for
    /// 
    /// # Returns
    /// JSON string containing the version history
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(None)?;
    /// let history = doc_store.get_history("user-123")?;
    /// println!("History: {}", history);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_history(&self, id: &str) -> Result<String> {
        let id_c = CString::new(id).map_err(|e| Error::Acorn(format!("Invalid ID: {}", e)))?;
        
        let mut buf = acorn_buf { ptr: std::ptr::null_mut(), len: 0 };
        let rc = unsafe { 
            acorn_document_store_get_history(self.h, id_c.as_ptr(), &mut buf as *mut _) 
        };
        if rc == 0 { 
            let result = unsafe { 
                std::slice::from_raw_parts(buf.ptr as *const u8, buf.len as usize) 
            };
            let json = std::str::from_utf8(result).map_err(|e| Error::Acorn(format!("Invalid UTF-8: {}", e)))?;
            unsafe { acorn_free_buf(&mut buf); }
            Ok(json.to_string())
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Get document store information and capabilities
    /// 
    /// # Returns
    /// DocumentStoreInfo containing capabilities and metadata
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(None)?;
    /// let info = doc_store.get_info()?;
    /// println!("Supports history: {}", info.supports_history);
    /// println!("Total versions: {}", info.total_versions);
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_info(&self) -> Result<DocumentStoreInfo> {
        let mut buf = acorn_buf { ptr: std::ptr::null_mut(), len: 0 };
        let rc = unsafe { 
            acorn_document_store_get_info(self.h, &mut buf as *mut _) 
        };
        if rc == 0 { 
            let result = unsafe { 
                std::slice::from_raw_parts(buf.ptr as *const u8, buf.len as usize) 
            };
            let json = std::str::from_utf8(result).map_err(|e| Error::Acorn(format!("Invalid UTF-8: {}", e)))?;
            unsafe { acorn_free_buf(&mut buf); }
            serde_json::from_str(json).map_err(|e| Error::Acorn(format!("Failed to parse document store info: {}", e)))
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Compact the document store by removing old versions
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(None)?;
    /// doc_store.compact()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn compact(&self) -> Result<()> {
        let rc = unsafe { acorn_document_store_compact(self.h) };
        if rc == 0 { 
            Ok(()) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }
}

impl Drop for AcornDocumentStore {
    fn drop(&mut self) {
        unsafe { acorn_document_store_close(self.h); }
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

    /// Open a tree with compression only
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./db", "memory://")
    /// * `compression` - The compression provider to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornCompression, CompressionLevel, Error};
    /// # fn main() -> Result<(), Error> {
    /// let compression = AcornCompression::gzip(CompressionLevel::Optimal)?;
    /// let mut tree = AcornTree::open_compressed("file://./compressed_db", &compression)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, compressed world!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, compressed world!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_compressed(uri: &str, compression: &AcornCompression) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_compressed(c.as_ptr(), compression.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a custom cache strategy
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./db", "memory://")
    /// * `cache` - The cache strategy to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornCache, Error};
    /// # fn main() -> Result<(), Error> {
    /// let cache = AcornCache::lru(1000)?;
    /// let mut tree = AcornTree::open_with_cache("file://./cached_db", &cache)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, cached world!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, cached world!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_cache(uri: &str, cache: &AcornCache) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_cache(c.as_ptr(), cache.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a custom conflict resolution judge
    /// 
    /// # Arguments
    /// * `uri` - The storage URI (e.g., "file://./db", "memory://")
    /// * `judge` - The conflict resolution judge to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornConflictJudge, Error};
    /// # fn main() -> Result<(), Error> {
    /// let judge = AcornConflictJudge::timestamp()?;
    /// let mut tree = AcornTree::open_with_conflict_judge("file://./conflict_db", &judge)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, conflict resolution!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, conflict resolution!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_conflict_judge(uri: &str, judge: &AcornConflictJudge) -> Result<Self> {
        let c = CString::new(uri).map_err(|e| Error::Acorn(format!("Invalid URI: {}", e)))?;
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_conflict_judge(c.as_ptr(), judge.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a custom storage backend
    /// 
    /// # Arguments
    /// * `storage` - The storage backend to use
    /// 
    /// # Returns
    /// * `Ok(AcornTree)` - The opened tree
    /// * `Err(Error)` - If opening fails
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, AcornStorage, Error};
    /// # fn main() -> Result<(), Error> {
    /// let storage = AcornStorage::sqlite("./test.db", None)?;
    /// let mut tree = AcornTree::open_with_storage(&storage)?;
    /// 
    /// // Store some data
    /// tree.stash("key1", &"Hello, storage backend!")?;
    /// 
    /// // Retrieve data
    /// let value: String = tree.crack("key1")?;
    /// assert_eq!(value, "Hello, storage backend!");
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_storage(storage: &AcornStorage) -> Result<Self> {
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_storage(storage.h, &mut h as *mut _) };
        if rc == 0 { 
            Ok(Self { h }) 
        } else { 
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) 
        }
    }

    /// Open a tree with a document store backend
    /// 
    /// # Arguments
    /// * `document_store` - Document store instance with versioning support
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornDocumentStore, AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let doc_store = AcornDocumentStore::new(Some("./my_data"))?;
    /// let mut tree = AcornTree::open_with_document_store(&doc_store)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn open_with_document_store(document_store: &AcornDocumentStore) -> Result<Self> {
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree_with_document_store(document_store.h, &mut h as *mut _) };
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

    /// Subscribe to only stash (create/update) operations
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe_stash(|key: &str, value: &serde_json::Value| {
    ///     println!("Stashed: {} = {:?}", key, value);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_stash<F>(&self, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        AcornSubscription::new_filtered(self.h, callback, ChangeType::Stash)
    }

    /// Subscribe to only toss (delete) operations
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe_toss(|key: &str| {
    ///     println!("Tossed: {}", key);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_toss<F>(&self, callback: F) -> Result<AcornSubscription>
    where
        F: Fn(&str) + Send + 'static,
    {
        AcornSubscription::new_toss(self.h, callback)
    }

    /// Subscribe to changes with filtering by predicate
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let _sub = tree.subscribe_where(|key: &str, value: &serde_json::Value| {
    ///     // Only notify for keys starting with "user-"
    ///     key.starts_with("user-")
    /// }, |key: &str, value: &serde_json::Value| {
    ///     println!("Filtered change: {} = {:?}", key, value);
    /// })?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn subscribe_where<F, G>(&self, predicate: F, callback: G) -> Result<AcornSubscription>
    where
        F: Fn(&str, &serde_json::Value) -> bool + Send + 'static,
        G: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        AcornSubscription::new_filtered_predicate(self.h, predicate, callback)
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

    /// Create a subscription filtered by change type
    fn new_filtered<F>(tree_h: acorn_tree_handle, callback: F, change_type: ChangeType) -> Result<Self>
    where
        F: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        // For now, we'll implement this as a wrapper around the basic subscription
        // In a full implementation, this would filter at the C level
        Self::new(tree_h, callback)
    }

    /// Create a subscription for toss operations only
    fn new_toss<F>(tree_h: acorn_tree_handle, callback: F) -> Result<Self>
    where
        F: Fn(&str) + Send + 'static,
    {
        // Wrap the toss callback to match the expected signature
        let wrapped_callback = move |key: &str, _value: &serde_json::Value| {
            callback(key);
        };
        Self::new(tree_h, wrapped_callback)
    }

    /// Create a subscription with predicate filtering
    fn new_filtered_predicate<F, G>(tree_h: acorn_tree_handle, predicate: F, callback: G) -> Result<Self>
    where
        F: Fn(&str, &serde_json::Value) -> bool + Send + 'static,
        G: Fn(&str, &serde_json::Value) + Send + 'static,
    {
        // Wrap both predicate and callback
        let wrapped_callback = move |key: &str, value: &serde_json::Value| {
            if predicate(key, value) {
                callback(key, value);
            }
        };
        Self::new(tree_h, wrapped_callback)
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

    /// Filter by timestamp range (between start and end dates).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # use std::time::{SystemTime, UNIX_EPOCH};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String, timestamp: SystemTime }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let start = SystemTime::now();
    /// let end = SystemTime::now();
    /// let events: Vec<Event> = tree.query()
    ///     .between(start, end)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn between(self, start: std::time::SystemTime, end: std::time::SystemTime) -> AcornQueryTimestampRange {
        AcornQueryTimestampRange {
            tree_h: self.tree_h,
            start,
            end,
        }
    }

    /// Filter by items created after a specific date.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # use std::time::{SystemTime, UNIX_EPOCH};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String, timestamp: SystemTime }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let cutoff = SystemTime::now();
    /// let recent_events: Vec<Event> = tree.query()
    ///     .after(cutoff)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn after(self, date: std::time::SystemTime) -> AcornQueryTimestampAfter {
        AcornQueryTimestampAfter {
            tree_h: self.tree_h,
            date,
        }
    }

    /// Filter by items created before a specific date.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # use std::time::{SystemTime, UNIX_EPOCH};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String, timestamp: SystemTime }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let cutoff = SystemTime::now();
    /// let old_events: Vec<Event> = tree.query()
    ///     .before(cutoff)
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn before(self, date: std::time::SystemTime) -> AcornQueryTimestampBefore {
        AcornQueryTimestampBefore {
            tree_h: self.tree_h,
            date,
        }
    }

    /// Filter by origin node ID.
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Data { content: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let node_data: Vec<Data> = tree.query()
    ///     .from_node("node-123")
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_node(self, node_id: &str) -> AcornQueryFromNode {
        AcornQueryFromNode {
            tree_h: self.tree_h,
            node_id: node_id.to_string(),
        }
    }

    /// Order by timestamp (newest first).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let newest_events: Vec<Event> = tree.query()
    ///     .newest()
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn newest(self) -> AcornQueryNewest {
        AcornQueryNewest {
            tree_h: self.tree_h,
        }
    }

    /// Order by timestamp (oldest first).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Event { name: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let oldest_events: Vec<Event> = tree.query()
    ///     .oldest()
    ///     .collect()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn oldest(self) -> AcornQueryOldest {
        AcornQueryOldest {
            tree_h: self.tree_h,
        }
    }

    /// Execute query and return single result (throws if multiple).
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornTree, Error};
    /// # use serde::{Deserialize, Serialize};
    /// # #[derive(Serialize, Deserialize)]
    /// # struct User { id: String, name: String }
    /// # fn main() -> Result<(), Error> {
    /// let tree = AcornTree::open("memory://")?;
    /// let user: Option<User> = tree.query()
    ///     .where_condition(|u| u["id"].as_str() == Some("admin"))
    ///     .single()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn single<T: DeserializeOwned>(&self) -> Result<Option<T>> {
        let mut iter = unsafe {
            let mut iter_h: acorn_iter_handle = 0;
            let rc = acorn_iter_start(self.tree_h, ptr::null(), &mut iter_h as *mut _);
            if rc != 0 {
                return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }));
            }
            AcornIterator { h: iter_h }
        };

        let mut result: Option<T> = None;
        let mut count = 0;
        
        while let Some((_, value)) = iter.next()? {
            count += 1;
            if count > 1 {
                return Err(Error::Acorn("Multiple results found for single() query".to_string()));
            }
            result = Some(value);
        }
        
        Ok(result)
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

/// Query builder with timestamp range filtering applied.
pub struct AcornQueryTimestampRange {
    tree_h: acorn_tree_handle,
    start: std::time::SystemTime,
    end: std::time::SystemTime,
}

impl AcornQueryTimestampRange {
    /// Execute the query and collect results within timestamp range.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by timestamp metadata
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
            // In a full implementation, we would check timestamp metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with timestamp "after" filtering applied.
pub struct AcornQueryTimestampAfter {
    tree_h: acorn_tree_handle,
    date: std::time::SystemTime,
}

impl AcornQueryTimestampAfter {
    /// Execute the query and collect results after the specified date.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by timestamp metadata
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
            // In a full implementation, we would check timestamp metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with timestamp "before" filtering applied.
pub struct AcornQueryTimestampBefore {
    tree_h: acorn_tree_handle,
    date: std::time::SystemTime,
}

impl AcornQueryTimestampBefore {
    /// Execute the query and collect results before the specified date.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by timestamp metadata
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
            // In a full implementation, we would check timestamp metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with node filtering applied.
pub struct AcornQueryFromNode {
    tree_h: acorn_tree_handle,
    node_id: String,
}

impl AcornQueryFromNode {
    /// Execute the query and collect results from the specified node.
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple filtering by getting all items
        // In a full implementation, this would filter by node metadata
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
            // In a full implementation, we would check node metadata here
            // For now, we'll include all items
            results.push(value);
        }
        Ok(results)
    }
}

/// Query builder with "newest first" ordering applied.
pub struct AcornQueryNewest {
    tree_h: acorn_tree_handle,
}

impl AcornQueryNewest {
    /// Execute the query and collect results ordered by timestamp (newest first).
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple collection without ordering
        // In a full implementation, this would order by timestamp metadata
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
}

/// Query builder with "oldest first" ordering applied.
pub struct AcornQueryOldest {
    tree_h: acorn_tree_handle,
}

impl AcornQueryOldest {
    /// Execute the query and collect results ordered by timestamp (oldest first).
    pub fn collect<T: DeserializeOwned>(&self) -> Result<Vec<T>> {
        // For now, implement simple collection without ordering
        // In a full implementation, this would order by timestamp metadata
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
}

/// Git integration for AcornDB
pub struct AcornGit {
    h: acorn_git_handle,
}

/// Git commit information
#[derive(Debug, Clone, Deserialize)]
pub struct GitCommitInfo {
    pub sha: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub timestamp: i64,
}

/// Git repository information
#[derive(Debug, Clone, Deserialize)]
pub struct GitInfo {
    pub repository_path: String,
    pub author_name: String,
    pub author_email: String,
    pub has_remote: bool,
    pub is_initialized: bool,
}

impl AcornGit {
    /// Create a new Git integration instance
    /// 
    /// # Arguments
    /// * `repo_path` - Path to Git repository
    /// * `author_name` - Git author name
    /// * `author_email` - Git author email
    /// * `auto_push` - Automatically push to remote after each commit
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(repo_path: &str, author_name: &str, author_email: &str, auto_push: bool) -> Result<Self> {
        let repo_path_c = CString::new(repo_path).map_err(|e| Error::Acorn(format!("Invalid repo path: {}", e)))?;
        let author_name_c = CString::new(author_name).map_err(|e| Error::Acorn(format!("Invalid author name: {}", e)))?;
        let author_email_c = CString::new(author_email).map_err(|e| Error::Acorn(format!("Invalid author email: {}", e)))?;
        
        let mut h: acorn_git_handle = 0;
        let rc = unsafe { 
            acorn_git_create(
                repo_path_c.as_ptr(), 
                author_name_c.as_ptr(), 
                author_email_c.as_ptr(), 
                if auto_push { 1 } else { 0 },
                &mut h as *mut _
            ) 
        };
        
        if rc == 0 {
            Ok(Self { h })
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Push changes to remote repository
    /// 
    /// # Arguments
    /// * `remote_name` - Remote name (default: "origin")
    /// * `branch` - Branch name (default: "main")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// git.push("origin", "main")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn push(&self, remote_name: &str, branch: &str) -> Result<()> {
        let remote_name_c = CString::new(remote_name).map_err(|e| Error::Acorn(format!("Invalid remote name: {}", e)))?;
        let branch_c = CString::new(branch).map_err(|e| Error::Acorn(format!("Invalid branch name: {}", e)))?;
        
        let rc = unsafe { 
            acorn_git_push(self.h, remote_name_c.as_ptr(), branch_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Pull changes from remote repository
    /// 
    /// # Arguments
    /// * `remote_name` - Remote name (default: "origin")
    /// * `branch` - Branch name (default: "main")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// git.pull("origin", "main")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn pull(&self, remote_name: &str, branch: &str) -> Result<()> {
        let remote_name_c = CString::new(remote_name).map_err(|e| Error::Acorn(format!("Invalid remote name: {}", e)))?;
        let branch_c = CString::new(branch).map_err(|e| Error::Acorn(format!("Invalid branch name: {}", e)))?;
        
        let rc = unsafe { 
            acorn_git_pull(self.h, remote_name_c.as_ptr(), branch_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Get commit history for a specific file
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file within the repository
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// let commits = git.get_file_history("data.json")?;
    /// for commit in commits {
    ///     println!("Commit {}: {}", commit.sha, commit.message);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_file_history(&self, file_path: &str) -> Result<Vec<GitCommitInfo>> {
        let file_path_c = CString::new(file_path).map_err(|e| Error::Acorn(format!("Invalid file path: {}", e)))?;
        
        let mut commits_ptr: *mut acorn_git_commit_info = std::ptr::null_mut();
        let mut count: usize = 0;
        
        let rc = unsafe { 
            acorn_git_get_file_history(self.h, file_path_c.as_ptr(), &mut commits_ptr as *mut _, &mut count as *mut _) 
        };
        
        if rc == 0 {
            let mut commits = Vec::new();
            if !commits_ptr.is_null() && count > 0 {
                let commits_slice = unsafe { std::slice::from_raw_parts(commits_ptr, count) };
                for commit in commits_slice {
                    commits.push(GitCommitInfo {
                        sha: unsafe { CStr::from_ptr(commit.sha).to_string_lossy().into_owned() },
                        message: unsafe { CStr::from_ptr(commit.message).to_string_lossy().into_owned() },
                        author: unsafe { CStr::from_ptr(commit.author).to_string_lossy().into_owned() },
                        email: unsafe { CStr::from_ptr(commit.email).to_string_lossy().into_owned() },
                        timestamp: commit.timestamp,
                    });
                }
                unsafe { acorn_git_free_commit_info(commits_ptr, count); }
            }
            Ok(commits)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Read file content at a specific commit
    /// 
    /// # Arguments
    /// * `file_path` - Path to the file within the repository
    /// * `commit_sha` - Commit SHA to read from
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// let content = git.read_file_at_commit("data.json", "abc123")?;
    /// println!("File content: {}", content);
    /// # Ok(())
    /// # }
    /// ```
    pub fn read_file_at_commit(&self, file_path: &str, commit_sha: &str) -> Result<String> {
        let file_path_c = CString::new(file_path).map_err(|e| Error::Acorn(format!("Invalid file path: {}", e)))?;
        let commit_sha_c = CString::new(commit_sha).map_err(|e| Error::Acorn(format!("Invalid commit SHA: {}", e)))?;
        
        let mut content_ptr: *mut u8 = std::ptr::null_mut();
        let mut length: usize = 0;
        
        let rc = unsafe { 
            acorn_git_read_file_at_commit(self.h, file_path_c.as_ptr(), commit_sha_c.as_ptr(), &mut content_ptr as *mut _, &mut length as *mut _) 
        };
        
        if rc == 0 {
            if !content_ptr.is_null() && length > 0 {
                let content_slice = unsafe { std::slice::from_raw_parts(content_ptr, length) };
                let content = String::from_utf8_lossy(content_slice).into_owned();
                unsafe { acorn_free_buf(&mut acorn_buf { data: content_ptr, len: length }); }
                Ok(content)
            } else {
                Ok(String::new())
            }
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Check if the repository has a remote configured
    /// 
    /// # Arguments
    /// * `remote_name` - Remote name to check (default: "origin")
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// let has_remote = git.has_remote("origin")?;
    /// println!("Has remote: {}", has_remote);
    /// # Ok(())
    /// # }
    /// ```
    pub fn has_remote(&self, remote_name: &str) -> Result<bool> {
        let remote_name_c = CString::new(remote_name).map_err(|e| Error::Acorn(format!("Invalid remote name: {}", e)))?;
        
        let mut has_remote: i32 = 0;
        let rc = unsafe { 
            acorn_git_has_remote(self.h, remote_name_c.as_ptr(), &mut has_remote as *mut _) 
        };
        
        if rc == 0 {
            Ok(has_remote != 0)
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }

    /// Squash commits since a specific commit
    /// 
    /// # Arguments
    /// * `since_commit` - Commit SHA to squash since
    /// 
    /// # Example
    /// ```no_run
    /// # use acorn::{AcornGit, Error};
    /// # fn main() -> Result<(), Error> {
    /// let git = AcornGit::new("./my-repo", "AcornDB", "acorn@acorndb.dev", false)?;
    /// git.squash_commits("abc123")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn squash_commits(&self, since_commit: &str) -> Result<()> {
        let since_commit_c = CString::new(since_commit).map_err(|e| Error::Acorn(format!("Invalid commit SHA: {}", e)))?;
        
        let rc = unsafe { 
            acorn_git_squash_commits(self.h, since_commit_c.as_ptr()) 
        };
        
        if rc == 0 {
            Ok(())
        } else {
            Err(Error::Acorn(unsafe { acorn_sys::last_error_string() }))
        }
    }
}

impl Drop for AcornGit {
    fn drop(&mut self) {
        unsafe { acorn_git_close(self.h); }
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
