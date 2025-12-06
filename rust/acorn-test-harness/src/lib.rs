#![allow(dead_code)]

use acorn_core::{AcornResult, BranchId, Nut, Trunk, TrunkCapability};

#[derive(Debug, Clone)]
pub struct TrunkContract;

impl TrunkContract {
    pub fn round_trip_bytes<S>(trunk: &S) -> AcornResult<()>
    where
        S: Trunk<Vec<u8>>,
    {
        let branch = BranchId::new("contract");
        let key = "key";
        let payload = b"contract-payload".to_vec();

        trunk.put(
            &branch,
            key,
            Nut {
                value: payload.clone(),
            },
        )?;
        let fetched = trunk
            .get(&branch, key)?
            .ok_or_else(|| crate::harness_err("missing payload"))?;
        if fetched.value != payload {
            return Err(crate::harness_err("payload mismatch"));
        }

        trunk.delete(&branch, key)?;
        if trunk.get(&branch, key)?.is_some() {
            return Err(crate::harness_err("delete did not remove payload"));
        }

        Ok(())
    }

    pub fn assert_capabilities<S>(trunk: &S, expected: &[TrunkCapability])
    where
        S: Trunk<Vec<u8>>,
    {
        let caps = trunk.capabilities();
        for cap in expected {
            assert!(
                caps.contains(cap),
                "expected capability {:?} but it was missing",
                cap
            );
        }
    }
}

fn harness_err(msg: &str) -> acorn_core::AcornError {
    acorn_core::AcornError::Trunk(msg.to_string())
}
