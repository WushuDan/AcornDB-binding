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
        let c = CString::new(uri).unwrap();
        let mut h: acorn_tree_handle = 0;
        let rc = unsafe { acorn_open_tree(c.as_ptr(), &mut h as *mut _) };
        if rc == 0 { Ok(Self { h }) } else { Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) }
    }

    pub fn stash<T: Serialize>(&mut self, id: &str, value: &T) -> Result<()> {
        let json = serde_json::to_vec(value).unwrap();
        let idc = CString::new(id).unwrap();
        let rc = unsafe { acorn_stash_json(self.h, idc.as_ptr(), json.as_ptr(), json.len()) };
        if rc == 0 { Ok(()) } else { Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })) }
    }

    pub fn crack<T: DeserializeOwned>(&self, id: &str) -> Result<T> {
        let idc = CString::new(id).unwrap();
        let mut buf = acorn_buf { data: ptr::null_mut(), len: 0 };
        let rc = unsafe { acorn_crack_json(self.h, idc.as_ptr(), &mut buf as *mut _) };
        if rc == 1 { return Err(Error::NotFound); }
        if rc != 0 { return Err(Error::Acorn(unsafe { acorn_sys::last_error_string() })); }
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
    #[test]
    fn api_exists() {
        // Smoke test for symbols
        let _ = Error::NotFound;
    }
}
