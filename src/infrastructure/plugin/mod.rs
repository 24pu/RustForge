use wasmtime::{Engine, Module, Store, Linker, Memory, Instance};
use anyhow::Result;

pub struct WasmPlugin {
    engine: Engine,
    _module: Module,
    store: Store<()>,
    instance: Instance,
    memory: Memory,
}

impl WasmPlugin {
    pub fn load(path: &str) -> Result<Self> {
        let engine = Engine::default();
        let module = Module::from_file(&engine, path)?;
        let mut store = Store::new(&engine, ());
        let linker = Linker::new(&engine);
        let instance = linker.instantiate(&mut store, &module)?;
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow::anyhow!("memory export not found"))?
            .clone();
        Ok(Self {
            engine,
            _module: module,
            store,
            instance,
            memory,
        })
    }

    pub fn call_execute(&mut self, input: &str) -> Result<String> {
        let exec_func = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "execute")?;
        let len_func = self.instance
            .get_typed_func::<(), i32>(&mut self.store, "get_last_result_len")?;
        
        let input_bytes = input.as_bytes();
        let input_len = input_bytes.len();
        self.memory.write(&mut self.store, 0, input_bytes)?;
        let result_ptr = exec_func.call(&mut self.store, (0, input_len as i32))?;
        let result_len = len_func.call(&mut self.store, ())? as usize;
        let mut buffer = vec![0u8; result_len];
        self.memory.read(&mut self.store, result_ptr as usize, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    pub fn call_metadata(&mut self) -> Result<String> {
        let func = self.instance.get_typed_func::<(), i32>(&mut self.store, "plugin_metadata")?;
        let len_func = self.instance.get_typed_func::<(), i32>(&mut self.store, "get_last_result_len")?;
        let ptr = func.call(&mut self.store, ())?;
        let len = len_func.call(&mut self.store, ())? as usize;
        let mut buffer = vec![0u8; len];
        self.memory.read(&mut self.store, ptr as usize, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    pub async fn call_render_page(&mut self, page: &str) -> Result<String> {
        let func = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "render_page")?;
        let len_func = self.instance
            .get_typed_func::<(), i32>(&mut self.store, "get_last_result_len")?;
        let page_bytes = page.as_bytes();
        self.memory.write(&mut self.store, 0, page_bytes)?;
        let result_ptr = func.call(&mut self.store, (0, page_bytes.len() as i32))?;
        let result_len = len_func.call(&mut self.store, ())? as usize;
        let mut buffer = vec![0u8; result_len];
        self.memory.read(&mut self.store, result_ptr as usize, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    pub async fn call_is_page_protected(&mut self, page: &str) -> Result<String> {
        let func = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "is_page_protected")?;
        let len_func = self.instance
            .get_typed_func::<(), i32>(&mut self.store, "get_last_result_len")?;
        let page_bytes = page.as_bytes();
        self.memory.write(&mut self.store, 0, page_bytes)?;
        let result_ptr = func.call(&mut self.store, (0, page_bytes.len() as i32))?;
        let result_len = len_func.call(&mut self.store, ())? as usize;
        let mut buffer = vec![0u8; result_len];
        self.memory.read(&mut self.store, result_ptr as usize, &mut buffer)?;
        Ok(String::from_utf8(buffer)?)
    }

    
}