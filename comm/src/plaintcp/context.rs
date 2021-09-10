use tokio::sync::RwLock;
use std::sync::Arc;
use fnv::FnvHashMap as HashMap;
use super::super::Context;

const DEFAULT_RETRY_DURATION: std::time::Duration 
    = std::time::Duration::from_millis(1);
    
#[derive(Debug)]
pub struct TcpContext {
    /// The amount of time I need to wait before 
    pub(crate) retry_duration: std::time::Duration,
    pub(crate) connections: Arc<HashMap<&'a str, RwLock<String>>>,
    pub(crate) rt: tokio::runtime::Runtime,
}

impl Context for TcpContext {
}

pub struct TcpContextBuilder {
    pub retry_duration: Option<std::time::Duration>,
    pub connections: Option<RwLock<String>>,
    pub rt: Option<tokio::runtime::Runtime>,
}

impl std::default::Default for TcpContextBuilder {
   fn default() -> Self {
       Self {
           retry_duration: Some(DEFAULT_RETRY_DURATION),
           connections: None,
           rt: None,
       }
   }
}

#[allow(dead_code)]
impl TcpContextBuilder {
    pub fn build(self) -> Result<TcpContext, String> {
        self.validate()?;
        Ok(
            TcpContext{
                retry_duration: self.retry_duration.unwrap(),
                connections: self.connections.unwrap(),
                rt: self.rt.unwrap(),
            }
        )  
    } 
    
    fn validate(&self) -> Result<(), String> {
        if self.rt.is_none() {
            return Err("Runtime cannot be None".to_string());
        }
        Ok(())
    }
}