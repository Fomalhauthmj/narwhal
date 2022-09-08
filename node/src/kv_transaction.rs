// Copyright 2022, 贺梦杰 (njtech_hemengjie@qq.com)
// SPDX-License-Identifier: Apache-2.0
use rand::distributions::DistString;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum KVTransaction<K, V> {
    Insert(K, V),
    Get(K),
    Remove(K),
}
pub fn random_string(len: usize) -> String {
    let mut rng = rand::thread_rng();
    rand::distributions::Alphanumeric.sample_string(&mut rng, len)
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_transaction() {
        let key_len = 10;
        let value_len = 100;
        let insert = KVTransaction::<String, String>::Insert(
            random_string(key_len),
            random_string(value_len),
        );
        let get = KVTransaction::<String, String>::Get(random_string(key_len));
        let remove = KVTransaction::<String, String>::Remove(random_string(key_len));
        assert_eq!(
            20 + key_len + value_len,
            bincode::serialized_size(&insert).unwrap() as usize
        );
        assert_eq!(
            12 + key_len,
            bincode::serialized_size(&get).unwrap() as usize
        );
        assert_eq!(
            12 + key_len,
            bincode::serialized_size(&remove).unwrap() as usize
        );
    }
}
