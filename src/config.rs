use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use arti_client::config::{CfgPath, ConfigBuildError, TorClientConfigBuilder};
use arti_client::TorClientConfig;

use crate::utils;

#[napi]
pub struct ConfigCircuitTiming {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigCircuitTiming {
  /**
   * How long after a circuit has first been used should we give it out for new requests?
   */
  #[napi]
  pub fn max_dirtiness(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .circuit_timing()
      .max_dirtiness(Duration::from_millis(millis as u64));
    self
  }

  /**
   * When waiting for requested circuits,
   * wait at least this long before using a suitable-looking circuit launched by some other request.
   */
  #[napi]
  pub fn request_loyalty(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .circuit_timing()
      .request_loyalty(Duration::from_millis(millis as u64));
    self
  }

  /**
   * When a circuit is requested, we stop retrying new circuits after this many attempts.
   */
  #[napi]
  pub fn request_max_retries(&mut self, retries: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .circuit_timing()
      .request_max_retries(retries);
    self
  }

  /**
   * When a circuit is requested, we stop retrying new circuits after this much time.
   */
  #[napi]
  pub fn request_timeout(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .circuit_timing()
      .request_timeout(Duration::from_millis(millis as u64));
    self
  }
}

#[napi]
pub struct ConfigDirectoryTolerance {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigDirectoryTolerance {
  /**
   * For how long before a directory document is valid should we accept it?
   * Having a nonzero value here allows us to tolerate a little clock skew.
   * Defaults to 1 day.
   */
  #[napi]
  pub fn pre_valid_tolerance(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .directory_tolerance()
      .pre_valid_tolerance(Duration::from_millis(millis as u64));
    self
  }

  /**
   * For how long after a directory document is valid should we consider it usable?
   * Having a nonzero value here allows us to tolerate a little clock skew,
   * and makes us more robust to temporary failures for the directory authorities to reach consensus.
   * Defaults to 3 days (per prop212).
   */
  #[napi]
  pub fn post_valid_tolerance(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .directory_tolerance()
      .post_valid_tolerance(Duration::from_millis(millis as u64));
    self
  }
}

#[napi]
pub struct ConfigDownloadSchedule {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigDownloadSchedule {
  /**
   * Top-level configuration for how to retry our initial bootstrap attempt.
   */
  #[napi]
  pub fn retry_bootstrap(&mut self) -> &Self {
    self
      .config
      .borrow_mut()
      .download_schedule()
      .retry_bootstrap();
    self
  }

  /**
   * Configuration for how to retry an authority cert download.
   */
  #[napi]
  pub fn retry_certs(&mut self) -> &Self {
    self.config.borrow_mut().download_schedule().retry_certs();
    self
  }

  /**
   * Configuration for how to retry a consensus download.
   */
  #[napi]
  pub fn retry_consensus(&mut self) -> &Self {
    self
      .config
      .borrow_mut()
      .download_schedule()
      .retry_consensus();
    self
  }

  /**
   * Configuration for how to retry a microdescriptor download.
   */
  #[napi]
  pub fn retry_microdescs(&mut self) -> &Self {
    self
      .config
      .borrow_mut()
      .download_schedule()
      .retry_microdescs();
    self
  }
}

#[napi]
pub struct ConfigNetParams {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigNetParams {
  /**
   * Facility to override network parameters from the values set in the consensus.
   */
  #[napi]
  pub fn override_net_params(&mut self, key: String, value: i32) -> &Self {
    self
      .config
      .borrow_mut()
      .override_net_params()
      .insert(key, value);
    self
  }
}

#[napi]
pub struct ConfigPathRules {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigPathRules {
  /**
   * Set the length of a bit-prefix for a default IPv4 subnet-family.
   * Any two relays will be considered to belong to the same family if their IPv4 addresses share at least this many initial bits.
   */
  #[napi]
  pub fn ipv4_subnet_family_prefix(&mut self, value: u8) -> &Self {
    self
      .config
      .borrow_mut()
      .path_rules()
      .ipv4_subnet_family_prefix(value);
    self
  }

  /**
   * Set the length of a bit-prefix for a default IPv6 subnet-family.
   * Any two relays will be considered to belong to the same family if their IPv6 addresses share at least this many initial bits.
   */
  #[napi]
  pub fn ipv6_subnet_family_prefix(&mut self, value: u8) -> &Self {
    self
      .config
      .borrow_mut()
      .path_rules()
      .ipv6_subnet_family_prefix(value);
    self
  }

  /**
   * Set the whole list (overriding the default)
   */
  #[napi]
  pub fn set_long_lived_ports(&mut self, values: Vec<u16>) -> &Self {
    self
      .config
      .borrow_mut()
      .path_rules()
      .set_long_lived_ports(values);
    self
  }

  /**
   * Set the whole list (overriding the default)
   * e.g: "127.0.0.0/8:*"
   */
  #[napi]
  pub fn set_reachable_addrs(&mut self, values: Vec<String>) -> napi::Result<&Self> {
    let mut pats = Vec::new();

    for value in values {
      let pat = utils::map_error(value.parse())?;
      pats.push(pat);
    }

    self
      .config
      .borrow_mut()
      .path_rules()
      .set_reachable_addrs(pats);
    Ok(self)
  }
}

#[napi]
pub struct ConfigPreemptiveCircuits {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigPreemptiveCircuits {
  /**
   * If we have at least this many available circuits, we suspend construction of preemptive circuits. whether our available circuits support our predicted exit ports or not.
   */
  #[napi]
  pub fn disable_at_threshold(&mut self, value: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .preemptive_circuits()
      .disable_at_threshold(value as usize);
    self
  }

  /**
   * How many available circuits should we try to have, at minimum, for each predicted exit port?
   */
  #[napi]
  pub fn min_exit_circs_for_port(&mut self, value: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .preemptive_circuits()
      .min_exit_circs_for_port(value as usize);
    self
  }

  /**
   * After we see the client request a connection to a new port, how long should we predict that the client will still want to have circuits available for that port?
   */
  #[napi]
  pub fn prediction_lifetime(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .preemptive_circuits()
      .prediction_lifetime(Duration::from_millis(millis as u64));
    self
  }

  /**
   * Set the whole list (overriding the default)
   */
  #[napi]
  pub fn set_initial_predicted_ports(&mut self, ports: Vec<u16>) -> &Self {
    self
      .config
      .borrow_mut()
      .preemptive_circuits()
      .set_initial_predicted_ports(ports);
    self
  }
}

#[napi]
pub struct ConfigStorage {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigStorage {
  /**
   * Location on disk for cached information.
   * This follows the rules for /var/cache: "sufficiently old" filesystem objects in it may be deleted outside of the control of Arti, and Arti will continue to function properly. It is also fine to delete the directory as a whole, while Arti is not running.
   */
  #[napi]
  pub fn cache_dir(&mut self, dir: String) -> &Self {
    let path = CfgPath::new(dir);
    self.config.borrow_mut().storage().cache_dir(path);
    self
  }

  /**
   * Location on disk for less-sensitive persistent state information.
   */
  #[napi]
  pub fn state_dir(&mut self, dir: String) -> &Self {
    let path = CfgPath::new(dir);
    self.config.borrow_mut().storage().state_dir(path);
    self
  }

  /**
   * Whether keystore use is enabled.
   */
  #[napi]
  pub fn keystore(&mut self, enabled: bool) -> &Self {
    self
      .config
      .borrow_mut()
      .storage()
      .keystore()
      .enabled(tor_config::BoolOrAuto::Explicit(enabled));
    self
  }
}

#[napi]
pub struct ConfigStreamTimeouts {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl ConfigStreamTimeouts {
  /**
   * How long should we wait before timing out a stream when connecting to a host?
   */
  #[napi]
  pub fn connect_timeout(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .stream_timeouts()
      .connect_timeout(Duration::from_millis(millis as u64));
    self
  }

  /**
   * How long should we wait before timing out when resolving a DNS PTR record?
   */
  #[napi]
  pub fn resolve_ptr_timeout(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .stream_timeouts()
      .resolve_ptr_timeout(Duration::from_millis(millis as u64));
    self
  }

  /**
   * How long should we wait before timing out when resolving a DNS record?
   */
  #[napi]
  pub fn resolve_timeout(&mut self, millis: u32) -> &Self {
    self
      .config
      .borrow_mut()
      .stream_timeouts()
      .resolve_timeout(Duration::from_millis(millis as u64));
    self
  }
}

#[napi]
pub enum PaddingLevel {
  None,
  Reduced,
  Normal,
}

impl PaddingLevel {
  fn napi(&self) -> tor_config::PaddingLevel {
    match self {
      Self::None => tor_config::PaddingLevel::None,
      Self::Reduced => tor_config::PaddingLevel::Reduced,
      Self::Normal => tor_config::PaddingLevel::Normal,
    }
  }
}

#[napi(js_name = "TorClientConfig")]
#[derive(Default)]
pub struct NativeTorClientConfig {
  config: Rc<RefCell<TorClientConfigBuilder>>,
}

#[napi]
impl NativeTorClientConfig {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self {
      config: Default::default(),
    }
  }

  #[napi(factory)]
  pub fn create() -> Self {
    Self::new()
  }

  /**
   * Should we allow attempts to make Tor connections to local addresses?
   * This option is off by default, since (by default) Tor exits will always reject connections to such addresses
   */
  #[napi]
  pub fn allow_local_addrs(&mut self, value: bool) -> &Self {
    self
      .config
      .borrow_mut()
      .address_filter()
      .allow_local_addrs(value);
    self
  }

  /**
   * Padding conf
   */
  #[napi]
  pub fn padding(&mut self, level: PaddingLevel) -> &Self {
    self.config.borrow_mut().channel().padding(level.napi());
    self
  }

  /**
   * Circuit timing conf
   */
  #[napi(getter)]
  pub fn circuit_timing(&self) -> ConfigCircuitTiming {
    ConfigCircuitTiming {
      config: self.config.clone(),
    }
  }

  /**
   * Directory tolerance conf
   */
  #[napi(getter)]
  pub fn directory_tolerance(&self) -> ConfigDirectoryTolerance {
    ConfigDirectoryTolerance {
      config: self.config.clone(),
    }
  }

  /**
   * Download schedule conf
   */
  #[napi(getter)]
  pub fn download_schedule(&self) -> ConfigDownloadSchedule {
    ConfigDownloadSchedule {
      config: self.config.clone(),
    }
  }

  /**
   * Net params conf
   */
  #[napi(getter)]
  pub fn net_params(&self) -> ConfigNetParams {
    ConfigNetParams {
      config: self.config.clone(),
    }
  }

  /**
   * Path rules conf
   */
  #[napi(getter)]
  pub fn path_rules(&self) -> ConfigPathRules {
    // TODO: complete
    ConfigPathRules {
      config: self.config.clone(),
    }
  }

  /**
   * Preemptive circuits conf
   */

  #[napi(getter)]
  pub fn preemptive_circuits(&self) -> ConfigPreemptiveCircuits {
    // TODO: complete
    ConfigPreemptiveCircuits {
      config: self.config.clone(),
    }
  }

  /**
   * Storage conf
   */
  #[napi(getter)]
  pub fn storage(&self) -> ConfigStorage {
    ConfigStorage {
      config: self.config.clone(),
    }
  }

  /**
   * Stream timeouts conf
   */
  #[napi(getter)]
  pub fn stream_timeouts(&self) -> ConfigStreamTimeouts {
    ConfigStreamTimeouts {
      config: self.config.clone(),
    }
  }

  // TODO:
  // TOR NETWORK
  // VANGUARDS
  // USE OBSOLETE SOFTWARE

  pub fn build(&self) -> Result<TorClientConfig, ConfigBuildError> {
    self.config.borrow().build()
  }
}
