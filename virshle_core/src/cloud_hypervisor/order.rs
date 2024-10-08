// Error Handling
use virshle_error::{VirshleError, VirtError, WrapError};
use miette::{IntoDiagnostic, Result};

pub trait Order<T> {
    fn order_by_id(&mut self) -> Result<&mut Vec<T>, VirshleError>;
    fn order_by_name(&mut self) -> Result<&mut Vec<T>, VirshleError>;
}
