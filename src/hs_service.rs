use std::sync::Arc;

use futures_core::Stream;
use futures_util::stream::StreamExt;
use napi::bindgen_prelude::ObjectFinalize;
use napi::tokio::sync::Mutex;
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;
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

#[napi(js_name = "OnionService", custom_finalize)]
pub struct NativeOnionService {
  service: Option<Arc<RunningOnionService>>,
  rend_request: Arc<Mutex<Option<Box<dyn Stream<Item = RendRequest> + Unpin + Send>>>>,
  cancel_token: CancellationToken,
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
      service: Some(service),
      rend_request: Arc::new(Mutex::new(Some(Box::new(rend_request)))),
      cancel_token: CancellationToken::new(),
    }
  }

  /**
   * Waits until the hidden service reaches the `Running` state.
   * If `maxTime` is provided, throws an error if the timeout is exceeded.
   * If the service enters the `Broken` state, throws an error immediately.
   */
  #[napi]
  pub async fn wait_running(&self, max_time: Option<u32>) -> napi::Result<()> {
    use tokio::time::{sleep, Duration, Instant};

    let deadline = max_time.map(|ms| Instant::now() + Duration::from_millis(ms as u64));

    loop {
      match self.state() {
        StateOnionService::Running => return Ok(()),
        StateOnionService::Broken => {
          return Err(napi::Error::from_reason("Hidden service broken"));
        }
        _ => {
          if let Some(d) = deadline {
            if Instant::now() >= d {
              return Err(napi::Error::from_reason(
                "Timed out waiting for hidden service to start",
              ));
            }
          }

          sleep(Duration::from_millis(500)).await;
        }
      }
    }
  }

  /**
   * Retrieves the next RendRequest in the queue.
   */
  #[napi]
  pub async fn poll(&self) -> napi::Result<NativeRendRequest> {
    let token = self.cancel_token.clone();

    let fut = async {
      let mut rend_request = self.rend_request.lock().await;
      if let Some(rend_request) = rend_request.as_mut() {
        rend_request
          .next()
          .await
          .map(NativeRendRequest::from_rend_request)
          .ok_or(napi::Error::from_reason("Hidden service was closed"))
      } else {
        Err(napi::Error::from_reason("Hidden service was closed"))
      }
    };

    tokio::select! {
      biased;

      _ = token.cancelled() => Err(napi::Error::from_reason("Hidden service was closed")),
      result = fut => utils::map_error(result)
    }
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
      .as_ref()
      .and_then(|service| service.onion_address().map(|address| address.to_string()))
  }

  /**
   * Returns the current status of the hidden service.
   */
  #[napi]
  pub fn state(&self) -> StateOnionService {
    self
      .service
      .as_ref()
      .map(|service| match service.status().state() {
        tor_hsservice::status::State::Shutdown => StateOnionService::Shutdown,
        tor_hsservice::status::State::Bootstrapping => StateOnionService::Bootstrapping,
        tor_hsservice::status::State::DegradedReachable => StateOnionService::DegradedReachable,
        tor_hsservice::status::State::DegradedUnreachable => StateOnionService::DegradedUnreachable,
        tor_hsservice::status::State::Running => StateOnionService::Running,
        tor_hsservice::status::State::Recovering => StateOnionService::Recovering,
        tor_hsservice::status::State::Broken => StateOnionService::Broken,
        _ => StateOnionService::Unknown,
      })
      .unwrap_or(StateOnionService::Shutdown)
  }

  /**
   * Close the hidden service.
   */
  #[napi]
  pub fn close(&mut self) {
    self.cancel_token.cancel();
    self.service.take();
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async { self.rend_request.lock().await.take() });
  }
}

impl ObjectFinalize for NativeOnionService {
  fn finalize(mut self, _env: napi::Env) -> napi::Result<()> {
    self.close();
    Ok(())
  }
}

#[napi]
pub enum StateOnionService {
  Shutdown,
  Bootstrapping,
  DegradedReachable,
  DegradedUnreachable,
  Running,
  Recovering,
  Broken,
  Unknown,
}
