use std::sync::Arc;

use futures_core::Stream;
use futures_util::stream::StreamExt;
use napi::tokio::sync::Mutex;
use tor_hsservice::{RendRequest, RunningOnionService};

use crate::hs_streams_request::NativeStreamsRequest;
use crate::utils;

#[napi(js_name = "RendRequest")]
pub struct NativeRendRequest {
  request: Option<RendRequest>,
}

#[napi]
impl NativeRendRequest {
  /**
   * This class cannot be constructed manually.
   */
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    Err(napi::Error::new(
      napi::Status::GenericFailure,
      "This class cannot be constructed manually.".to_string(),
    ))
  }

  pub fn from_rend_request(request: RendRequest) -> Self {
    Self {
      request: Some(request),
    }
  }

  /**
   * Mark this request as accepted, and try to connect to the client's provided rendezvous point.
   */
  #[napi]
  pub async unsafe fn accept(&mut self) -> napi::Result<Option<NativeStreamsRequest>> {
    if let Some(request) = self.request.take() {
      let streams_request = utils::map_error(request.accept().await)?;
      Ok(Some(NativeStreamsRequest::from_streams_request(
        streams_request,
      )))
    } else {
      Ok(None)
    }
  }

  /**
   * Reject this request. (The client will receive no notification.)
   */
  #[napi]
  pub async unsafe fn reject(&mut self) -> napi::Result<()> {
    if let Some(request) = self.request.take() {
      utils::map_error(request.reject().await)?;
      Ok(())
    } else {
      Ok(())
    }
  }
}

#[napi(js_name = "OnionService")]
pub struct NativeOnionService {
  service: Arc<RunningOnionService>,
  rend_request: Arc<Mutex<Box<dyn Stream<Item = RendRequest> + Unpin + Send>>>,
}

#[napi]
impl NativeOnionService {
  /**
   * This class cannot be constructed manually.
   */
  #[napi(constructor)]
  pub fn new() -> napi::Result<Self> {
    Err(napi::Error::new(
      napi::Status::GenericFailure,
      "This class cannot be constructed manually.".to_string(),
    ))
  }

  pub fn from_service(
    service: Arc<RunningOnionService>,
    rend_request: impl Stream<Item = RendRequest> + Send + Unpin + 'static,
  ) -> Self {
    Self {
      service,
      rend_request: Arc::new(Mutex::new(Box::new(rend_request))),
    }
  }

  /**
   * Retrieves the next RendRequest in the queue.
   */
  #[napi]
  pub async fn poll(&self) -> Option<NativeRendRequest> {
    self
      .rend_request
      .lock()
      .await
      .next()
      .await
      .map(NativeRendRequest::from_rend_request)
  }

  /**
   * Return the onion address of this service.
   * Clients must know the service's onion address in order to discover or connect to it.
   * Returns `null|undefined` if the HsId of the service could not be found in any of the configured keystores.
   */
  #[napi]
  pub fn address(&self) -> Option<String> {
    self
      .service
      .onion_address()
      .map(|address| address.to_string())
  }
}
