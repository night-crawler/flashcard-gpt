use crate::error::CoreError;

pub trait ResponseExt {
    fn errors_or_ok(&mut self) -> Result<(), CoreError>;
}

impl ResponseExt for surrealdb::Response {
    fn errors_or_ok(&mut self) -> Result<(), CoreError> {
        let errors = self.take_errors();
        if !errors.is_empty() {
            return Err(CoreError::CreateError(format!("{:?}", errors).into()));
        }
        Ok(())
    }
}
