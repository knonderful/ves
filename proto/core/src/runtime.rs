use crate::ProtoCore;
use anyhow::Result;
use std::path::Path;
use ves_proto_common::gpu::{PaletteColor, PaletteIndex, PaletteTableIndex};
use wasmtime::{
    AsContext, Caller, Config, Engine, Extern, Linker, Memory, Module, Store, StoreContext, Trap,
    TypedFunc,
};

pub struct Runtime {
    store: Store<ProtoCore>,
    create_instance_fn: TypedFunc<(), u32>,
    step_fn: TypedFunc<u32, ()>,
}

impl Runtime {
    pub(crate) fn from_path(path: &Path, core: ProtoCore) -> Result<Self> {
        let wasm_file = std::fs::canonicalize(path)?;
        let engine = Engine::new(Config::new().debug_info(true))?;
        let module = Module::from_file(&engine, &wasm_file)?;
        let mut store = Store::new(&engine, core);

        let mut linker = Linker::new(&engine);
        linker.func_wrap(
            "log", // module
            "log", // function
            move |mut caller: Caller<'_, ProtoCore>, level: u32, ptr: u32, len: u32| {
                let mem = Self::get_memory(&mut caller)?;
                let message = Self::get_str(Self::get_slice(caller.as_context(), &mem, ptr, len)?)?;

                let log_level = level.try_into().map_err(Trap::new)?;
                caller.data().logger.log(log_level, message);

                Ok(())
            },
        )?;

        linker.func_wrap(
            "gpu",     // module
            "oam_set", // function
            move |mut caller: Caller<'_, ProtoCore>, index: u32, entry: u64| {
                let index = u8::try_from(index)
                    .map_err(|_| Trap::new("Could not convert index value to u8."))?;

                caller.data_mut().set_oam_entry(index.into(), entry.into());

                Ok(())
            },
        )?;

        linker.func_wrap(
            "gpu",         // module
            "palette_set", // function
            move |mut caller: Caller<'_, ProtoCore>, palette: u32, index: u32, color: u32| {
                let palette = u8::try_from(palette)
                    .map(PaletteTableIndex::from)
                    .map_err(|_| Trap::new("Could not convert palette value to u8."))?;
                let index = u8::try_from(index)
                    .map(PaletteIndex::from)
                    .map_err(|_| Trap::new("Could not convert index value to u8."))?;
                let color = u16::try_from(color)
                    .map(PaletteColor::from)
                    .map_err(|_| Trap::new("Could not convert color value to u16."))?;

                caller.data_mut().set_palette_entry(palette, index, color);

                Ok(())
            },
        )?;

        let instance = linker.instantiate(&mut store, &module)?;

        let create_instance_fn =
            instance.get_typed_func::<(), u32, _>(&mut store, "create_instance")?;

        let step_fn = instance.get_typed_func::<u32, (), _>(&mut store, "step")?;

        Ok(Self {
            store,
            create_instance_fn,
            step_fn,
        })
    }

    pub(crate) fn create_instance(&mut self) -> Result<u32, Trap> {
        self.create_instance_fn.call(&mut self.store, ())
    }

    pub(crate) fn step(&mut self, args: u32) -> Result<&ProtoCore, Trap> {
        self.step_fn.call(&mut self.store, args)?;
        Ok(self.store.data())
    }

    fn get_memory<T>(caller: &mut Caller<'_, T>) -> std::result::Result<Memory, Trap> {
        match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => Ok(mem),
            _ => Err(Trap::new("Failed to find memory.")),
        }
    }

    fn get_slice<'a, 'b, T: 'a>(
        store: impl Into<StoreContext<'a, T>>,
        mem: &'b Memory,
        ptr: u32,
        len: u32,
    ) -> std::result::Result<&'a [u8], Trap> {
        let index_from: usize = ptr
            .try_into()
            .map_err(|_| Trap::new(format!("Could not convert ptr ({ptr}) to usize.")))?;
        let index_to: usize = len
            .try_into()
            .map_err(|_| Trap::new(format!("Could not convert len ({len}) to usize.")))?;

        mem.data(store)
            .get(index_from..)
            .and_then(|arr| arr.get(..index_to))
            .ok_or_else(|| {
                Trap::new(format!(
                    "Could not get slice with pointer {} and length {}.",
                    ptr, len
                ))
            })
    }

    fn get_str(data: &[u8]) -> std::result::Result<&str, Trap> {
        match std::str::from_utf8(data) {
            Ok(str) => Ok(str),
            Err(_) => Err(Trap::new("Invalid UTF-8")),
        }
    }
}
