use std::path::PathBuf;

use tauri::api::path::{config_dir, data_dir};

use crate::holochain_version::HolochainVersion;

pub struct FileSystemManager {
  holochain_version: HolochainVersion,
}

impl FileSystemManager {
  pub fn new(holochain_version: HolochainVersion) -> Self {
    FileSystemManager { holochain_version }
  }

  /** Config */

  pub fn holochain_config_path(&self) -> PathBuf {
    let version: String = self.holochain_version.into();

    config_dir()
      .expect("Could not get config dir")
      .join("holochain")
      .join(version)
  }

  pub fn conductor_config_path(&self) -> PathBuf {
    self.holochain_config_path().join("conductor-config.yml")
  }

  /** Data */

  pub fn data_path() -> PathBuf {
    data_dir()
      .expect("Could not get config dir")
      .join("holochain")
  }

  pub fn data_path_for_this_holochain_version(&self) -> PathBuf {
    let version: String = self.holochain_version.into();

    Self::data_path().join(version)
  }

  pub fn conductor_data_path(&self) -> PathBuf {
    self
      .data_path_for_this_holochain_version()
      .join("conductor")
  }

  pub fn app_ui_path(&self, app_id: &String) -> PathBuf {
    self.uis_data_path().join(app_id)
  }

  pub fn uis_data_path(&self) -> PathBuf {
    self.data_path_for_this_holochain_version().join("uis")
  }

  pub fn caddyfile_path() -> PathBuf {
    Self::data_path().join("Caddyfile")
  }

  pub fn port_mapping_path() -> PathBuf {
    Self::data_path().join("port_mapping.yml")
  }

  pub fn keystore_data_path(&self) -> PathBuf {
    let version: String = self.holochain_version.lair_keystore_version().into();

    data_dir()
      .expect("Could not get config dir")
      .join("lair")
      .join(version)
  }
}
