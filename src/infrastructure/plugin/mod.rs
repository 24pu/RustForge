use wasmtime::{Engine, Module, Store, Linker, Memory, Instance};
use anyhow::Result;

// 内存偏移量常量，避免不同函数间数据覆盖
const INPUT_OFFSET: usize = 0;              // 用于 execute 的输入
const PAGE_INPUT_OFFSET: usize = 2048;      // 用于 render_page / is_page_protected 的输入

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
        
        // 使用 INPUT_OFFSET 写入输入
        self.memory.write(&mut self.store, INPUT_OFFSET, input_bytes)?;
        
        // 调用 execute，传入输入地址和长度
        let result_ptr = exec_func.call(&mut self.store, (INPUT_OFFSET as i32, input_len as i32))?;
        
        // 获取结果长度
        let result_len = len_func.call(&mut self.store, ())? as usize;
        
        // 从返回的指针读取结果
        let mut buffer = vec![0u8; result_len];
        self.memory.read(&mut self.store, result_ptr as usize, &mut buffer)?;
        
        let result = String::from_utf8(buffer)?;
        Ok(result)
    }

    pub fn call_metadata(&mut self) -> Result<String> {
        let func = self.instance
            .get_typed_func::<(), i32>(&mut self.store, "plugin_metadata")?;
        let len_func = self.instance
            .get_typed_func::<(), i32>(&mut self.store, "get_last_result_len")?;
        
        let ptr = func.call(&mut self.store, ())?;
        let len = len_func.call(&mut self.store, ())? as usize;
        
        let mut buffer = vec![0u8; len];
        self.memory.read(&mut self.store, ptr as usize, &mut buffer)?;
        
        let result = String::from_utf8(buffer)?;
        Ok(result)
    }

    // 保持异步，但实际是同步操作
    pub async fn call_render_page(&mut self, page: &str) -> Result<String> {
        let func = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "render_page")?;
        let len_func = self.instance
            .get_typed_func::<(), i32>(&mut self.store, "get_last_result_len")?;
        
        let page_bytes = page.as_bytes();
        let page_len = page_bytes.len();
        
        // 使用 PAGE_INPUT_OFFSET 写入页面名称
        self.memory.write(&mut self.store, PAGE_INPUT_OFFSET, page_bytes)?;
        
        // 调用 render_page，传入输入地址和长度
        let result_ptr = func.call(&mut self.store, (PAGE_INPUT_OFFSET as i32, page_len as i32))?;
        
        // 获取结果长度
        let result_len = len_func.call(&mut self.store, ())? as usize;
        
        // 从返回的指针读取结果
        let mut buffer = vec![0u8; result_len];
        self.memory.read(&mut self.store, result_ptr as usize, &mut buffer)?;
        
        let result = String::from_utf8(buffer)?;
        Ok(result)
    }

    // 保持异步，但实际是同步操作
    pub async fn call_is_page_protected(&mut self, page: &str) -> Result<String> {
        let func = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "is_page_protected")?;
        let len_func = self.instance
            .get_typed_func::<(), i32>(&mut self.store, "get_last_result_len")?;
        
        let page_bytes = page.as_bytes();
        let page_len = page_bytes.len();
        
        // 使用 PAGE_INPUT_OFFSET 写入页面名称
        self.memory.write(&mut self.store, PAGE_INPUT_OFFSET, page_bytes)?;
        
        // 调用 is_page_protected，传入输入地址和长度
        let result_ptr = func.call(&mut self.store, (PAGE_INPUT_OFFSET as i32, page_len as i32))?;
        
        // 获取结果长度
        let result_len = len_func.call(&mut self.store, ())? as usize;
        
        // 从返回的指针读取结果
        let mut buffer = vec![0u8; result_len];
        self.memory.read(&mut self.store, result_ptr as usize, &mut buffer)?;
        
        let result = String::from_utf8(buffer)?;
        
        // 添加调试日志（生产环境可移除）
       // println!("[DEBUG] is_page_protected('{}') = '{}'", page, result);
        
        Ok(result)
    }
}