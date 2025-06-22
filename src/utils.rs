use napi::{Error as NapiError, Result as NapiResult, Status};
use std::error::Error as StdError;

pub fn map_error<T, E>(result: Result<T, E>) -> NapiResult<T>
where
  E: StdError,
{
  result.map_err(|err| NapiError::new(Status::GenericFailure, err.to_string()))
}
