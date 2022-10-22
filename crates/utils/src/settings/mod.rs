use crate::{
    error::PorplError,
    location_info,
    settings::structs::Settings,
  };
use anyhow::Context;
use deser_hjson::from_str;
use once_cell::sync::Lazy;
use std::{env, fs, io::Error};

pub mod structs;

static DEFAULT_CONFIG_FILE: &str = "./config/defaults.hjson";

pub static SETTINGS: Lazy<Settings> = 
  Lazy::new(|| Settings::init().expect("Failed to load the settings file"));


impl Settings {
    /// Reads config from the configuration file
    /// 
    /// Warning: Only call this once.
    pub(crate) fn init() -> Result<Self, PorplError> {
        let config = from_str::<Settings>(&Self::read_config_file()
            .map_err(|_| PorplError::from_string("error reading config file", 500))?)
            .map_err(|_| PorplError::from_string("error converting config to string", 500))?;
        
        if config.hostname == "unset" {
            return Err(PorplError::from_string("Hostname variable is not set!", 500));
        }

        Ok(config)
    }

    pub fn get_config_location() -> String {
        env::var("PORPL_CONFIG_LOCATION").unwrap_or_else(|_| DEFAULT_CONFIG_FILE.to_string())
    }

    pub fn read_config_file() -> Result<String, Error> {
        fs::read_to_string(Self::get_config_location())
    }


    pub fn get_database_url(&self) -> String {
        let conf = &self.database;
        format!(
          "postgres://{}:{}@{}:{}/{}",
          conf.user, conf.password, conf.host, conf.port, conf.database,
        )
    }

    pub fn get_protocol_and_hostname(&self) -> String {
        format!("{}://{}", self.get_protocol_string(), self.hostname)
    }

    ///Returns "http" or "https" depending on tls_enabled setting
    pub fn get_protocol_string(&self) -> &'static str {
        if self.tls_enabled {
            "https"
        } else {
            "http"
        }
    }

    pub fn get_hostname_without_port(&self) -> Result<String, anyhow::Error> {
        Ok(
            self
                .hostname
                .split(':')
                .collect::<Vec<&str>>()
                .first()
                .context(location_info!())?
                .to_string(),
        )
    }


}