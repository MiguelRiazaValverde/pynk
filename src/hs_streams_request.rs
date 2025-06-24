use std::sync::Arc;

use futures_core::Stream;
use futures_util::lock::Mutex;
use futures_util::StreamExt;
use tor_cell::relaycell::msg::{Connected, End, EndReason};
use tor_hsservice::StreamRequest;
use tor_proto::stream::IncomingStreamRequest;

use crate::stream::NativeTorStream;
use crate::utils;

#[napi(js_name = "StreamRequest")]
pub struct NativeStreamRequest {
  request: Option<StreamRequest>,
}

#[napi]
impl NativeStreamRequest {
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

  pub fn from_stream_request(request: StreamRequest) -> Self {
    Self {
      request: Some(request),
    }
  }

  /**
   * Returns whether the current incoming stream request is a `Begin` request.
   * This indicates the start of a new incoming stream.
   */
  #[napi]
  pub fn is_begin(&self) -> bool {
    self
      .request
      .as_ref()
      .map(|request| matches!(request.request(), IncomingStreamRequest::Begin(_)))
      .unwrap_or_default()
  }

  /**
   * Returns the destination address for the incoming `Begin` stream request.
   * If the current request is a `Begin` request, returns the address as a byte vector.
   * Otherwise, returns `null|undefined`.
   */
  #[napi]
  pub fn addr(&self) -> Option<Vec<u8>> {
    self
      .request
      .as_ref()
      .and_then(|request| match request.request() {
        IncomingStreamRequest::Begin(begin) => Some(begin.addr().to_vec()),
        _ => None,
      })
  }

  /**
   * Returns the destination port for the incoming `Begin` stream request.
   * If the current request is a `Begin` request, returns the port number.
   * Otherwise, returns `None`.
   */
  #[napi]
  pub fn port(&self) -> Option<u16> {
    self
      .request
      .as_ref()
      .and_then(|request| match request.request() {
        IncomingStreamRequest::Begin(begin) => Some(begin.port()),
        _ => None,
      })
  }

  /**
   * Accept this request and send the client a CONNECTED message.
   * Returns a TorStream.
   */
  #[napi]
  pub async unsafe fn accept(&mut self) -> napi::Result<Option<NativeTorStream>> {
    if let Some(request) = self.request.take() {
      let data_stream = utils::map_error(request.accept(Connected::new_empty()).await)?;
      Ok(Some(NativeTorStream::from_stream(data_stream)))
    } else {
      Ok(None)
    }
  }

  /**
   * Reject this request, and send the client an END message.
   */
  #[napi]
  pub async unsafe fn reject(&mut self) -> napi::Result<()> {
    if let Some(request) = self.request.take() {
      utils::map_error(request.reject(End::new_with_reason(EndReason::DONE)).await)?;
    }
    Ok(())
  }

  /**
   * Reject this request and close the rendezvous circuit entirely, along with all other streams attached to the circuit.
   */
  #[napi]
  pub async unsafe fn shutdown_circuit(&mut self) -> napi::Result<()> {
    if let Some(request) = self.request.take() {
      utils::map_error(request.shutdown_circuit())?;
    }
    Ok(())
  }
}

#[napi(js_name = "StreamsRequest")]
pub struct NativeStreamsRequest {
  streams_request: Arc<Mutex<Box<dyn Stream<Item = StreamRequest> + Send + Unpin + 'static>>>,
}

unsafe impl Send for NativeStreamsRequest {}
unsafe impl Sync for NativeStreamsRequest {}

#[napi]
impl NativeStreamsRequest {
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

  pub fn from_streams_request(
    streams_request: impl Stream<Item = StreamRequest> + Send + Unpin + 'static,
  ) -> Self {
    Self {
      streams_request: Arc::new(Mutex::new(Box::new(streams_request))),
    }
  }

  /**
   * Retrieves the next StreamRequest in the queue.
   */
  #[napi]
  pub async unsafe fn poll(&mut self) -> Option<NativeStreamRequest> {
    self
      .streams_request
      .lock()
      .await
      .next()
      .await
      .map(NativeStreamRequest::from_stream_request)
  }
}
