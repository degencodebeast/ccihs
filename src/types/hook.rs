use super::{CCIHSResult, CrossChainMessage};


pub enum HookType {
    PreDispatch,
    PostDispatch,
    PreExecution,
    PostExecution,
}

pub trait Hook {
    fn execute(&self, message: &mut CrossChainMessage) -> CCIHSResult<()>;
}