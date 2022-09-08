// Copyright 2022, 贺梦杰 (njtech_hemengjie@qq.com)
// SPDX-License-Identifier: Apache-2.0

use std::{collections::HashMap, sync::Mutex};

use super::kv_transaction::KVTransaction;
use async_trait::async_trait;
use consensus::ConsensusOutput;
use executor::{ExecutionIndices, ExecutionState, ExecutionStateError};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MemoryExecutionEngineError {}
impl ExecutionStateError for MemoryExecutionEngineError {
    fn node_error(&self) -> bool {
        true
    }
}
pub struct MemoryKVExecutionEngine {
    inner: Mutex<HashMap<String, String>>,
}
impl Default for MemoryKVExecutionEngine {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}
#[async_trait]
impl ExecutionState for MemoryKVExecutionEngine {
    type Transaction = KVTransaction<String, String>;
    type Error = MemoryExecutionEngineError;
    type Outcome = Option<String>;

    async fn handle_consensus_transaction(
        &self,
        _consensus_output: &ConsensusOutput,
        _execution_indices: ExecutionIndices,
        transaction: Self::Transaction,
    ) -> Result<Self::Outcome, Self::Error> {
        let mut inner = self.inner.lock().unwrap();
        let outcome = match transaction.clone() {
            KVTransaction::Insert(k, v) => inner.insert(k, v),
            KVTransaction::Get(k) => inner.get(&k).cloned(),
            KVTransaction::Remove(k) => inner.remove(&k),
        };
        Ok(outcome)
    }

    fn deserialize(bytes: &[u8]) -> Result<Self::Transaction, bincode::Error> {
        bincode::deserialize(&bytes[9..])
    }

    fn ask_consensus_write_lock(&self) -> bool {
        true
    }

    fn release_consensus_write_lock(&self) {}

    async fn load_execution_indices(&self) -> Result<ExecutionIndices, Self::Error> {
        Ok(ExecutionIndices::default())
    }
}
