struct Injector {

}

impl Injector {
    pub fn find_process(exeutable_name: &str) -> Result<Self, u32> {
        unimplemented!()
    }

    pub fn create_process(executable_path: &str) -> Result<Self, u32> {
        unimplemented!()
    }

    pub fn inject_dll(&mut self, dll_path: &str) -> Result<(), u32> {
        unimplemented!()
    }

    pub fn close_process(&mut self) -> Result<(), u32> {
        unimplemented!()
    }
}
