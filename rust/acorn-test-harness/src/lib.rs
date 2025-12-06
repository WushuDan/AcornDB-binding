#![allow(dead_code)]

use acorn_core::{BranchId, Nut, Trunk};

pub struct TrunkContract;

impl TrunkContract {
    pub fn round_trip_bytes<T, S>(trunk: &S) -> bool
    where
        T: Clone + Send + Sync + 'static,
        S: Trunk<T>,
    {
        let branch = BranchId::new("contract");
        let key = "key";
        let nut = Nut {
            value: unsafe { std::mem::zeroed() },
        };

        // This is a placeholder; real contract tests will inject deterministic payloads.
        let _ = trunk.put(&branch, key, nut);
        true
    }
}
