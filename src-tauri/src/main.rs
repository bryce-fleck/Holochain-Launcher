#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]
use futures::lock::Mutex;
use launcher::config::LauncherConfig;
use launcher::error::RunLauncherError;
use std::sync::Arc;

use system_tray::initial_system_tray;
use tauri;
use tauri::api::process::kill_children;
use tauri::Manager;
use tauri::RunEvent;
use tauri::SystemTray;
use tauri::SystemTrayEvent;

mod commands;
mod file_system;
mod launcher;
mod menu;
mod running_state;
mod setup;
mod system_tray;

use crate::commands::{
  enable_app::{disable_app, enable_app},
  factory_reset::execute_factory_reset,
  get_state_info::get_state_info,
  get_web_app_info::get_web_app_info,
  install_app::install_app,
  open_app::open_app_ui,
  password::{initialize_keystore, unlock_and_launch},
  uninstall_app::uninstall_app,
};
use crate::launcher::manager::LauncherManager;
use crate::launcher::state::LauncherState;
use crate::menu::build_menu;
use crate::menu::handle_menu_event;
use crate::setup::logs::setup_logs;
use crate::system_tray::handle_system_tray_event;

fn main() {
  if let Err(err) = setup_logs() {
    println!("Error setting up the logs: {:?}", err);
  }

  let already_running = LauncherManager::is_launcher_already_running();

  // If holochain is already running, only display a small notice window
  if already_running {
    let build_result = tauri::Builder::default()
      .manage(LauncherState::Error(
        RunLauncherError::AnotherInstanceIsAlreadyRunning,
      ))
      .invoke_handler(tauri::generate_handler![get_state_info])
      .run(tauri::generate_context!());
    if let Err(err) = build_result {
      log::error!("Error building the window: {}", err);
    }
    return ();
  }

  let builder_result = tauri::Builder::default()
    .menu(build_menu())
    .on_menu_event(|event| handle_menu_event(event.menu_item_id(), event.window()))
    .system_tray(SystemTray::new().with_menu(initial_system_tray()))
    .on_system_tray_event(|app, event| match event {
      SystemTrayEvent::MenuItemClick { id, .. } => handle_system_tray_event(app, id),
      _ => {}
    })
    .invoke_handler(tauri::generate_handler![
      get_state_info,
      open_app_ui,
      initialize_keystore,
      unlock_and_launch,
      install_app,
      enable_app,
      disable_app,
      uninstall_app,
      get_web_app_info,
      execute_factory_reset,
      setup::logs::log,
    ])
    .setup(|app| {
      let handle = app.handle().clone();

      let manager_launch = tauri::async_runtime::block_on(async move {
        LauncherManager::launch(
          LauncherConfig {
            log_level: log::Level::Info,
          },
          handle,
        )
        .await
      });

      let launcher_state = match manager_launch {
        Ok(launcher_manager) => {
          log::info!("Launch setup successful");
          LauncherState::Running(Arc::new(Mutex::new(launcher_manager)))
        }
        Err(error) => {
          kill_children();
          log::error!("There was an error launching holochain: {:?}", error);
          LauncherState::Error(RunLauncherError::ErrorLaunching(error))
        }
      };

      app.manage(launcher_state);

      Ok(())
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
