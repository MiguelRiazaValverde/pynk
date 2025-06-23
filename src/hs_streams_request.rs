use std::sync::Arc;

use futures_core::Stream;
use futures_util::lock::Mutex;
use futures_util::StreamExt;
use tor_cell::relaycell::msg::{Connected, End, EndReason};
use tor_hsservice::StreamRequest;

use crate::stream::NativeTorStream;
use crate::utils;

#[napi(js_name = "StreamRequest")]
pub struct NativeStreamRequest {
  request: Option<StreamRequest>,
}

#[napi]
impl NativeStreamRequest {
  pub fn from_stream_request(request: StreamRequest) -> Self {
    Self {
      request: Some(request),
    }
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
}

#[napi(js_name = "StreamsRequest")]
pub struct NativeStreamsRequest {
  streams_request: Arc<Mutex<Box<dyn Stream<Item = StreamRequest> + Send + Unpin + 'static>>>,
}

unsafe impl Send for NativeStreamsRequest {}
unsafe impl Sync for NativeStreamsRequest {}

#[napi]
impl NativeStreamsRequest {
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
