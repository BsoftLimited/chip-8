mod batch;
pub use batch::{ Batch, BatchConfig};

mod shader;
use shader::Shader;

pub trait Disposable{ unsafe fn dispose(&mut self); }