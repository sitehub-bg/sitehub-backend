use std::net::IpAddr;
use std::time::Duration;

use anyhow::{Context, bail};
use figment::Figment;
use figment::providers::{Env, Format, Toml};
use serde::{Deserialize, Serialize};

const MAX_TIMEOUT_SECS: u64 = 300;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub host: String,
    pub port: u16,
    pub request_timeout_secs: u64,
    pub shutdown_grace_secs: u64,
}

// Env vars allowed at runtime. SITEHUB_CONFIG points at the config file and is
// not a Config field; the rest map to fields. Any SITEHUB_* var not in this list
// is treated as a typo and fails startup.
const ALLOWED_ENV_VARS: &[&str] = &[
    "SITEHUB_CONFIG",
    "SITEHUB_HOST",
    "SITEHUB_PORT",
    "SITEHUB_REQUEST_TIMEOUT_SECS",
    "SITEHUB_SHUTDOWN_GRACE_SECS",
];

// Hand-written Debug per ADR-0034: any new field must be added here explicitly,
// forcing the author to decide whether it's safe to log. Secret-bearing fields
// should be wrapped in a redacting type and not exposed via Debug.
impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("request_timeout_secs", &self.request_timeout_secs)
            .field("shutdown_grace_secs", &self.shutdown_grace_secs)
            .finish()
    }
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        check_env_var_typos()?;

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

        // request_timeout_secs == 0 means disabled (no per-request timeout).
        // Allowed in dev for debugging; production should always set a real bound.
        if self.request_timeout_secs > MAX_TIMEOUT_SECS {
            bail!(
                "request_timeout_secs must be <= {MAX_TIMEOUT_SECS} (got {}); use 0 to disable",
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

    pub fn request_timeout_enabled(&self) -> bool {
        self.request_timeout_secs > 0
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout_secs)
    }

    pub fn shutdown_grace(&self) -> Duration {
        Duration::from_secs(self.shutdown_grace_secs)
    }

    pub fn shutdown_timeout(&self) -> Duration {
        // When request_timeout is disabled (dev), the shutdown deadline is just
        // the grace window — we don't wait for an unlimited request.
        if self.request_timeout_enabled() {
            self.request_timeout().saturating_add(self.shutdown_grace())
        } else {
            self.shutdown_grace()
        }
    }
}

fn check_env_var_typos() -> anyhow::Result<()> {
    let unknown: Vec<String> = std::env::vars()
        .map(|(k, _)| k)
        .filter(|k| k.starts_with("SITEHUB_") && !ALLOWED_ENV_VARS.contains(&k.as_str()))
        .collect();

    if !unknown.is_empty() {
        bail!(
            "unknown SITEHUB_* environment variables (likely typos): {}. Allowed: {}",
            unknown.join(", "),
            ALLOWED_ENV_VARS.join(", "),
        );
    }
    Ok(())
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
    fn accepts_zero_request_timeout_as_disabled() {
        let mut c = ok_config();
        c.request_timeout_secs = 0;
        c.validate().unwrap();
        assert!(!c.request_timeout_enabled());
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

    use std::io::Write;

    fn write_toml(contents: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        f
    }

    fn isolated_env() -> Vec<(&'static str, Option<&'static str>)> {
        ALLOWED_ENV_VARS.iter().map(|v| (*v, None)).collect()
    }

    const VALID_TOML: &str = r#"
        host = "127.0.0.1"
        port = 4000
        request_timeout_secs = 7
        shutdown_grace_secs = 3
    "#;

    #[test]
    fn load_fails_when_config_env_var_not_set() {
        temp_env::with_vars(isolated_env(), || {
            let err = Config::load().unwrap_err();
            assert!(err.to_string().contains("SITEHUB_CONFIG"), "got: {err}");
        });
    }

    #[test]
    fn load_fails_when_config_file_missing() {
        let mut env = isolated_env();
        env.push(("SITEHUB_CONFIG", Some("/nonexistent/path.toml")));
        temp_env::with_vars(env, || {
            assert!(Config::load().is_err());
        });
    }

    #[test]
    fn load_reads_values_from_toml_file() {
        let f = write_toml(VALID_TOML);
        let path = f.path().to_str().unwrap().to_string();
        let mut env = isolated_env();
        env.push(("SITEHUB_CONFIG", Some(path.as_str())));
        temp_env::with_vars(env, || {
            let cfg = Config::load().unwrap();
            assert_eq!(cfg.host, "127.0.0.1");
            assert_eq!(cfg.port, 4000);
            assert_eq!(cfg.request_timeout_secs, 7);
            assert_eq!(cfg.shutdown_grace_secs, 3);
        });
    }

    #[test]
    fn load_env_var_overrides_toml_value() {
        let f = write_toml(VALID_TOML);
        let path = f.path().to_str().unwrap().to_string();
        let mut env = isolated_env();
        env.push(("SITEHUB_CONFIG", Some(path.as_str())));
        env.push(("SITEHUB_PORT", Some("9999")));
        temp_env::with_vars(env, || {
            let cfg = Config::load().unwrap();
            assert_eq!(cfg.port, 9999, "env var should override TOML");
        });
    }

    #[test]
    fn load_fails_on_unknown_toml_key() {
        let f = write_toml(
            r#"
            host = "127.0.0.1"
            port = 4000
            request_timeout_secs = 7
            shutdown_grace_secs = 3
            mystery_field = "oops"
        "#,
        );
        let path = f.path().to_str().unwrap().to_string();
        let mut env = isolated_env();
        env.push(("SITEHUB_CONFIG", Some(path.as_str())));
        temp_env::with_vars(env, || {
            assert!(Config::load().is_err(), "deny_unknown_fields should reject");
        });
    }

    #[test]
    fn load_fails_on_unknown_sitehub_env_var() {
        // Simulates a realistic typo: SITEHUB_PROT (transposed) instead of SITEHUB_PORT.
        // The allowlist should reject it at startup rather than silently ignore.
        let f = write_toml(VALID_TOML);
        let path = f.path().to_str().unwrap().to_string();
        let mut env = isolated_env();
        env.push(("SITEHUB_CONFIG", Some(path.as_str())));
        env.push(("SITEHUB_PROT", Some("3000")));
        temp_env::with_vars(env, || {
            let err = Config::load().unwrap_err();
            assert!(err.to_string().contains("SITEHUB_PROT"), "got: {err}");
        });
    }
}
