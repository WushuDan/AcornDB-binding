#![allow(dead_code)]

use acorn_core::{
    AcornResult, BranchId, CapabilityAdvertiser, HistoryEvent, HistoryProvider, Nut, Trunk, TrunkCapability,
    Ttl, TtlProvider,
};
use std::time::{Duration, SystemTime};

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
        S: Trunk<Vec<u8>> + CapabilityAdvertiser,
    {
        let caps = CapabilityAdvertiser::capabilities(trunk);
        for cap in expected {
            assert!(
                caps.contains(cap),
                "expected capability {:?} but it was missing",
                cap
            );
        }
    }

    pub fn ttl_expiry<S>(trunk: &S) -> AcornResult<()>
    where
        S: Trunk<Vec<u8>> + TtlProvider<Vec<u8>>,
    {
        let branch = BranchId::new("ttl-contract");
        let key = "ttl-key";
        let ttl = Ttl {
            expires_at: SystemTime::now() + Duration::from_millis(5),
        };
        trunk.put_with_ttl(
            &branch,
            key,
            Nut {
                value: b"live".to_vec(),
            },
            ttl,
        )?;

        if trunk.get(&branch, key)?.is_none() {
            return Err(harness_err("value missing before ttl expiry"));
        }

        std::thread::sleep(Duration::from_millis(10));
        if trunk.get(&branch, key)?.is_some() {
            return Err(harness_err("value still present after ttl expiry"));
        }

        Ok(())
    }

    pub fn assert_history(events: &[HistoryEvent<Vec<u8>>], key: &str) -> AcornResult<()> {
        let mut saw_put = false;
        let mut saw_delete = false;
        for event in events {
            match event {
                HistoryEvent::Put { key: k, .. } if k == key => saw_put = true,
                HistoryEvent::Delete { key: k } if k == key => saw_delete = true,
                _ => {}
            }
        }

        if !saw_put {
            return Err(harness_err("missing put history event"));
        }
        if !saw_delete {
            return Err(harness_err("missing delete history event"));
        }
        Ok(())
    }

    pub fn history_put_delete<S>(trunk: &S) -> AcornResult<()>
    where
        S: Trunk<Vec<u8>> + HistoryProvider<Vec<u8>>,
    {
        let branch = BranchId::new("history-contract");
        let key = "history-key";
        let payload = b"history".to_vec();

        trunk.put(
            &branch,
            key,
            Nut {
                value: payload.clone(),
            },
        )?;
        trunk.delete(&branch, key)?;

        let events = trunk.history(&branch)?;
        Self::assert_history(&events, key)
    }
}

fn harness_err(msg: &str) -> acorn_core::AcornError {
    acorn_core::AcornError::Trunk(msg.to_string())
}
