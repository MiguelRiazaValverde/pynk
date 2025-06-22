use crate::utils;
use arti_client::{CountryCode, StreamPrefs};
use std::str::FromStr;

#[napi(js_name = "StreamPrefs")]
#[derive(Default)]
pub struct NativeStreamPrefs {
  prefs: StreamPrefs,
}

#[napi]
impl NativeStreamPrefs {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  #[napi(factory)]
  pub fn create() -> Self {
    Self::new()
  }

  /**
   * Indicate that we don't care which country a stream appears to come from.
   */
  #[napi]
  pub fn any_exit_country(&mut self) -> &Self {
    self.prefs.any_exit_country();
    self
  }

  /**
   * Indicate that a stream should appear to come from the given country.
   * When this option is set, we will only pick exit relays that have an IP address that matches the country in our GeoIP database.
   *
   *  @param country_code - A two-letter ISO 3166-1 alpha-2 country code, e.g. "IT" or "UY".
   */
  #[napi]
  pub fn exit_country(&mut self, country_code: String) -> napi::Result<&Self> {
    self
      .prefs
      .exit_country(utils::map_error(CountryCode::from_str(&country_code))?);
    Ok(self)
  }

  /**
   * Indicate whether connection to a hidden service (.onion service) should be allowed.
   */
  #[napi]
  pub fn connect_to_onion_services(&mut self, value: bool) -> &Self {
    self
      .prefs
      .connect_to_onion_services(tor_config::BoolOrAuto::Explicit(value));
    self
  }

  /**
   * ndicate that a stream may only be made over IPv4.
   * When this option is set, we will only pick exit relays that support IPv4, and we will tell them to only give us IPv4 connections.
   */
  #[napi]
  pub fn ipv4_only(&mut self) -> &Self {
    self.prefs.ipv4_only();
    self
  }

  /**
   * Indicate that a stream may only be made over IPv6.
   * When this option is set, we will only pick exit relays that support IPv6, and we will tell them to only give us IPv6 connections.
   */
  #[napi]
  pub fn ipv6_only(&mut self) -> &Self {
    self.prefs.ipv6_only();
    self
  }

  /**
   * Indicate that a stream may be made over IPv4 or IPv6, but that we'd prefer IPv4.
   * This is the default.
   */
  #[napi]
  pub fn ipv4_preferred(&mut self) -> &Self {
    self.prefs.ipv4_preferred();
    self
  }

  /**
   * Indicate that a stream may be made over IPv4 or IPv6, but that we'd prefer IPv6.
   */
  #[napi]
  pub fn ipv6_preferred(&mut self) -> &Self {
    self.prefs.ipv6_preferred();
    self
  }

  /**
   * Return true if this stream has been configured as "optimistic".
   */
  #[napi]
  pub fn is_optimistic(&mut self) -> bool {
    self.prefs.is_optimistic()
  }

  /**
   * Indicate that no connection should share a circuit with any other.
   * Use with care: This is likely to have poor performance, and imposes a much greater load on the Tor network. Use this option only to make small numbers of connections each of which needs to be isolated from all other connections.
   * (Don't just use this as a "get more privacy!!" method: the circuits that it put connections on will have no more privacy than any other circuits. The only benefit is that these circuits will not be shared by multiple streams.)
   */
  #[napi]
  pub fn isolate_every_stream(&mut self) -> &Self {
    self.prefs.isolate_every_stream();
    self
  }

  /**
   * Indicate that connections with these preferences should have their own isolation group.
   */
  #[napi]
  pub fn new_isolation_group(&mut self) -> &Self {
    self.prefs.new_isolation_group();
    self
  }

  /**
   * Indicate that the stream should be opened "optimistically".
   * By default, streams are not "optimistic". When you call TorClient.connect, it won't give you a stream until the exit node has confirmed that it has successfully opened a connection to your target address. It's safer to wait in this way, but it is slower: it takes an entire round trip to get your confirmation.
   * If a stream is configured to be "optimistic", on the other hand, then TorClient.connect() will return the stream immediately, without waiting for an answer from the exit. You can start sending data on the stream right away, though of course this data will be lost if the connection is not actually successful.
   */
  #[napi]
  pub fn optimistic(&mut self) -> &Self {
    self.prefs.optimistic();
    self
  }

  pub fn get(&self) -> StreamPrefs {
    self.prefs.clone()
  }

  // TODO: isolation
}
