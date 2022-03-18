#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
use std::sync::Arc;
use std::sync::Mutex;

use managers::holochain::config::ManagerConfig;
use portpicker;
use state::LauncherManager;
use tauri;
use tauri::api::process::kill_children;
use tauri::RunEvent;
use tauri::SystemTrayEvent;

mod commands;
mod connection_status;
mod holochain_version;
mod managers;
mod menu;
mod setup;
mod state;
mod system_tray;
mod utils;

use crate::commands::{
  enable_app::{disable_app, enable_app},
  factory_reset::execute_factory_reset,
  get_connection_status::get_connection_status,
  get_web_app_info::get_web_app_info,
  install_app::install_app,
  open_app::open_app_ui,
  uninstall_app::uninstall_app,
};
use crate::connection_status::ConnectionStatus;
use crate::menu::build_menu;
use crate::menu::handle_menu_event;
use crate::setup::is_holochain_already_running;
use crate::setup::logs::setup_logs;
use crate::setup::version::get_holochain_version;
use crate::state::LauncherState;
use crate::system_tray::handle_system_tray_event;

fn main() {
  if let Err(err) = setup_logs() {
    println!("Error setting up the logs: {:?}", err);
  }

  let already_running =
    tauri::async_runtime::block_on(async move { is_holochain_already_running().await });

  // If holochain is already running, only display a small notice window
  if already_running {
    let build_result = tauri::Builder::default()
      .manage(LauncherState::AnotherInstanceIsAlreadyRunning)
      .invoke_handler(tauri::generate_handler![get_connection_status])
      .run(tauri::generate_context!());
    if let Err(err) = build_result {
      log::error!("Error building the window: {}", err);
    }
    return ();
  }

  let config = ManagerConfig::default();

  let manager_launch =
    tauri::async_runtime::block_on(async move { LauncherManager::launch(config).await });

  let connection_status = match manager_launch {
    Ok(launcher_manager) => {
      log::info!("Launch setup successful");
      ConnectionStatus::Connected(launcher_manager)
    }
    Err(err) => {
      kill_children();
      log::error!("There was an error launching holochain: {:?}", err);
      ConnectionStatus::Error { error: err }
    }
  };

  let state = LauncherState::Running(Arc::new(Mutex::new(connection_status)));

  let builder_result = tauri::Builder::default()
    .manage(state)
    .menu(SystemTray::new(builtin_system_tray()))
    .on_menu_event(|event| handle_menu_event(event.menu_item_id(), event.window()))
    .system_tray(build_system_tray())
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => handle_system_tray_event(app, id),
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![
      get_connection_status,
      open_app_ui,
      install_app,
      enable_app,
      disable_app,
      uninstall_app,
      get_web_app_info,
      execute_factory_reset,
      setup::logs::log,
    ])
    .setup(|app| {
      let launcher_state: LauncherState = *app.state();

      if let ConnectionStatus::Connected(launcher_manager) = launcher_state {
        launcher_manager.on_apps_changed(app);
      }
    })
    .build(tauri::generate_context!());

  match builder_result {
    Ok(builder) => {
      builder.run(|_app_handle, event| {
        if let RunEvent::ExitRequested { api, .. } = event {
          api.prevent_exit();
        }
      });
    }
    Err(err) => log::error!("Error building the app: {:?}", err),
  }
}
