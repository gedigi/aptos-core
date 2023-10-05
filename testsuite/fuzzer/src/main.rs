// Copyright © Aptos Foundation

use std::collections::HashMap;
use move_binary_format::file_format::{CompiledModule, CompiledScript, FunctionDefinitionIndex};
use move_core_types::{
    language_storage::{ModuleId, TypeTag},
    value::MoveValue, account_address::AccountAddress, identifier::Identifier,
};
use arbitrary::Arbitrary;
use dearbitrary::Dearbitrary;

#[derive(Debug, Eq, PartialEq, arbitrary::Arbitrary, dearbitrary::Dearbitrary)]
pub enum ExecVariant {
    Script {
        script: CompiledScript,
        type_args: Vec<TypeTag>,
        args: Vec<MoveValue>,
    },
    CallFunction {
        module: ModuleId,
        function: FunctionDefinitionIndex,
        type_args: Vec<TypeTag>,
        args: Vec<Vec<u8>>,
    },
}

#[derive(Debug, Eq, arbitrary::Arbitrary, dearbitrary::Dearbitrary, PartialEq)]
pub struct RunnableState {
    pub dep_modules: Vec<CompiledModule>,
    pub exec_variant: ExecVariant,
}

fn main() {
    println!("Hello, world!");
    let runnable_states = to_runnablestate(&read_csv("/home/gerardo/move_modules.csv"));
    let mut cnt: usize = 0;
    println!("Number of runnable states: {}", runnable_states.len());
    for runnable_state in runnable_states {
        let mut d = dearbitrary::Dearbitrator::new();
        runnable_state.dearbitrary(&mut d);
        let bytes = d.finish();
        
        let mut u = arbitrary::Unstructured::new(&bytes);
        let runnable_state_generated = RunnableState::arbitrary(&mut u).unwrap();
        if runnable_state != runnable_state_generated {
            println!("Failed <CompiledModule>: {:#?}", runnable_state.dep_modules[0]);
            cnt += 1;
        }
    }
    println!("Number of different states: {}", cnt);
}

// function that read a CSV files and get the third element of each row and return a vector of bytes
fn read_csv(path: &str) -> HashMap<ModuleId, CompiledModule> {
    let mut reader = csv::Reader::from_path(path).unwrap();

    let mut v = HashMap::new();
    
    for result in reader.records() {
        let record = result.unwrap();

        let address = record.get(0).unwrap();
        let module_name = record.get(1).unwrap();
        let bytecode = hex::decode(record.get(2).unwrap()).unwrap();

        let key = ModuleId::new(AccountAddress::from_hex_literal(address).unwrap(), Identifier::new(module_name).unwrap());
        let compiled_module = match CompiledModule::deserialize(&bytecode) {
            Ok(module) => module,
            Err(err) => panic!("Error deserializing module: {:?}", err),
        };
        v.insert(key, compiled_module);
    }
    v
}

fn to_runnablestate(
    map: &HashMap<ModuleId, CompiledModule>,
) -> Vec<RunnableState> {
    map.iter().map(|(module_id, compiled_module)| {
        RunnableState {
            dep_modules: vec![compiled_module.to_owned()],
            exec_variant: ExecVariant::CallFunction {
                module: module_id.to_owned(),
                function: FunctionDefinitionIndex::new(0),
                type_args: vec![],
                args: vec![],
            },
        }
    }).collect()
}