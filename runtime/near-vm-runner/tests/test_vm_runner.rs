use near_primitives::hash::hash;
use near_primitives::runtime::fees::RuntimeFeesConfig;
use near_vm_logic::mocks::mock_external::MockedExternal;
use near_vm_logic::{VMConfig, VMContext, VMKind, ProtocolVersion};
use near_vm_runner::{ContractCaller, ContractCallPrepareRequest};

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use near_primitives::types::CompiledContractCache;
use std::ops::Deref;
use near_vm_logic::profile::ProfileData;
use near_vm_errors::VMError;
use near_vm_errors::FunctionCallError::MethodResolveError;
use near_vm_errors::MethodResolveError::MethodNotFound;
use near_vm_errors::VMError::FunctionCallError;

const TEST_CONTRACT_1: &'static [u8] = include_bytes!("../tests/res/test_contract_rs.wasm");
const TEST_CONTRACT_2: &'static [u8] = include_bytes!("../tests/res/test_contract_ts.wasm");

fn default_vm_context() -> VMContext {
    return VMContext {
        current_account_id: "alice".to_string(),
        signer_account_id: "bob".to_string(),
        signer_account_pk: vec![0, 1, 2],
        predecessor_account_id: "carol".to_string(),
        input: vec![],
        block_index: 1,
        block_timestamp: 1586796191203000000,
        account_balance: 10u128.pow(25),
        account_locked_balance: 0,
        storage_usage: 100,
        attached_deposit: 0,
        prepaid_gas: 10u64.pow(18),
        random_seed: vec![0, 1, 2],
        is_view: false,
        output_data_receivers: vec![],
        epoch_height: 1,
    };
}

#[derive(Default, Clone)]
pub struct MockCompiledContractCache {
    store: Arc<Mutex<HashMap<Vec<u8>, Vec<u8>>>>,
}

impl MockCompiledContractCache {
    pub fn len(&self) -> usize {
        self.store.lock().unwrap().len()
    }
}

impl CompiledContractCache for MockCompiledContractCache {
    fn put(&self, key: &[u8], value: &[u8]) -> Result<(), std::io::Error> {
        self.store.lock().unwrap().insert(key.to_vec(), value.to_vec());
        Ok(())
    }

    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, std::io::Error> {
        let res = self.store.lock().unwrap().get(key).cloned();
        Ok(res)
    }
}

#[test]
pub fn test_vm_runner() {
    let code1 = TEST_CONTRACT_1;
    let code2 = TEST_CONTRACT_2;

    let method_name1 = "log_something";

    let mut fake_external = MockedExternal::new();

    let context = default_vm_context();
    let vm_config = VMConfig::default();
    let cache: Option<Arc<dyn CompiledContractCache>> = Some(Arc::new(MockCompiledContractCache::default()));
    let fees = RuntimeFeesConfig::default();
    let promise_results = vec![];
    let mut requests = Vec::new();
    let mut caller= ContractCaller::new(2);
    for _ in 0..3 {
        requests.push(ContractCallPrepareRequest {
            code_hash: hash(code1),
            code: code1.to_vec(),
            vm_config: vm_config.clone(),
            cache: cache.clone(),
        });
       requests.push(ContractCallPrepareRequest {
            code_hash: hash(code2),
            code: code2.to_vec(),
            vm_config: vm_config.clone(),
            cache: cache.clone(),
        });
    }
    let calls = caller.preload(requests,VMKind::Wasmer0);
    let profile_data = ProfileData::new_disabled();
    for prepared in &calls {
        let result = caller.run_preloaded(
             prepared,
            method_name1,
            &mut fake_external,
            context.clone(),
            &vm_config,
            &fees,
            &promise_results,
             ProtocolVersion::MAX,
            profile_data.clone(),
        );
        println!("result is {:?}", result);
        /*match result.1 {
            Some(err) => {
                match err => {
                } => {},
                _ => assert!(false, "Unexpected error: {?:}", err),
            },
            None => {},
        }*/
    }
}
