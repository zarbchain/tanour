use crate::action::Action;
use crate::error::Error;
use crate::memory;
use crate::provider::Provider;
use crate::types::Address;
use crate::utils;
use log::{debug, trace};
#[cfg(feature = "cranelift")]
use wasmer::Cranelift;
#[cfg(not(feature = "cranelift"))]
use wasmer::Singlepass;
use wasmer::{
    wasmparser::Operator, BaseTunables, CompilerConfig, Engine, Pages, Store, Target, Universal,
    WASM_PAGE_SIZE,
};
use wasmer::{Exports, Function, ImportObject, Instance as WasmerInstance, Module, Val};

#[derive(Debug, Clone)]
pub struct ResultData {
    pub gas_left: u64,
    pub data: Vec<u8>,
}

pub fn execute(provider: &mut dyn Provider, action: &Action) -> crate::Result<()> {
    let module = compile(&action.code, action.memory_limit)?;

    let mut import_obj = ImportObject::new();
    let mut env_imports = Exports::new();

    import_obj.register("env", env_imports);

    let wasmer_instance = Box::from(WasmerInstance::new(&module, &import_obj).map_err(
        |original| Error::InstantiationError {
            msg: format!("{:?}", original),
        },
    )?);

    Ok(())
}

/// Compiles a given Wasm bytecode into a module.
/// The given memory limit (in bytes) is used when memories are created.
pub fn compile(code: &[u8], memory_limit: u64) -> crate::Result<Module> {
    let gas_limit = 0;
    let mut config;

    #[cfg(feature = "cranelift")]
    {
        config = Cranelift::default();
    };

    #[cfg(not(feature = "cranelift"))]
    {
        config = Singlepass::default();
    };

    let engine = Universal::new(config).engine();
    let base = BaseTunables::for_target(&Target::default());
    let tunables =
        memory::LimitingTunables::new(base, memory::limit_to_pages(memory_limit as usize));
    let store = Store::new_with_tunables(&engine, tunables);

    let module = Module::new(&store, code).map_err(|original| Error::CompileError {
        msg: format!("{:?}", original),
    })?;

    Ok(module)
}

#[cfg(test)]
#[path = "./execute_test.rs"]
mod execute_test;
