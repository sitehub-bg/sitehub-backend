use std::net::IpAddr;
use std::time::Duration;

use anyhow::{Context, bail};
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::{Deserialize, Serialize};

const MAX_TIMEOUT_SECS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub shutdown_grace_secs: u64,
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        let path = std::env::var("SITEHUB_CONFIG")
            .context("SITEHUB_CONFIG must be set to the path of a config TOML file")?;

        let cfg: Self = Figment::from(Toml::file(&path))
            .merge(Env::prefixed("SITEHUB_"))
            .extract()
            .with_context(|| format!("failed to load config from {path}"))?;

        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> anyhow::Result<()> {
        self.host
            .parse::<IpAddr>()
            .with_context(|| format!("host {:?} is not a valid IP address", self.host))?;

        if self.port == 0 {
            bail!("port must be > 0");
        }

        if self.request_timeout_secs == 0 {
            bail!("request_timeout_secs must be > 0");
        }
        if self.request_timeout_secs > MAX_TIMEOUT_SECS {
            bail!(
                "request_timeout_secs must be <= {MAX_TIMEOUT_SECS} (got {})",
                self.request_timeout_secs
            );
        }

        if self.shutdown_grace_secs == 0 {
            bail!("shutdown_grace_secs must be > 0");
        }
        if self.shutdown_grace_secs > MAX_TIMEOUT_SECS {
            bail!(
                "shutdown_grace_secs must be <= {MAX_TIMEOUT_SECS} (got {})",
                self.shutdown_grace_secs
            );
        }

        Ok(())
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    pub fn shutdown_grace(&self) -> Duration {
        Duration::from_secs(self.shutdown_grace_secs)
    }

    pub fn shutdown_timeout(&self) -> Duration {
        self.request_timeout()
            .saturating_add(self.shutdown_grace())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ok_config() -> Config {
        Config {
            host: "0.0.0.0".into(),
            port: 3000,
            request_timeout_secs: 10,
            shutdown_grace_secs: 10,
        }
    }

    #[test]
    fn valid_config_passes() {
        ok_config().validate().unwrap();
    }

    #[test]
    fn rejects_invalid_host() {
        let mut c = ok_config();
        c.host = "not-an-ip".into();
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_zero_port() {
        let mut c = ok_config();
        c.port = 0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_zero_request_timeout() {
        let mut c = ok_config();
        c.request_timeout_secs = 0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_excessive_request_timeout() {
        let mut c = ok_config();
        c.request_timeout_secs = MAX_TIMEOUT_SECS + 1;
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_zero_shutdown_grace() {
        let mut c = ok_config();
        c.shutdown_grace_secs = 0;
        assert!(c.validate().is_err());
    }

    #[test]
    fn rejects_excessive_shutdown_grace() {
        let mut c = ok_config();
        c.shutdown_grace_secs = MAX_TIMEOUT_SECS + 1;
        assert!(c.validate().is_err());
    }
}
