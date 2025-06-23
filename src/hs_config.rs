use std::str::FromStr;

use arti_client::config::onion_service::{OnionServiceConfig, OnionServiceConfigBuilder};
use tor_hsservice::HsNickname;

use crate::utils;

#[napi(js_name = "OnionServiceConfig")]
#[derive(Default)]
pub struct NativeOnionServiceConfig {
  config: OnionServiceConfigBuilder,
}

#[napi]
impl NativeOnionServiceConfig {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  #[napi]
  pub fn create() -> Self {
    Self::new()
  }

  /**
   * The nickname used to look up this service's keys, state, configuration, etc.
   */
  #[napi]
  pub fn nickname(&mut self, nickname: String) -> napi::Result<()> {
    self
      .config
      .nickname(utils::map_error(HsNickname::from_str(&nickname))?);
    Ok(())
  }

  pub fn build(&self) -> Result<OnionServiceConfig, tor_config::ConfigBuildError> {
    self.config.build()
  }
}
