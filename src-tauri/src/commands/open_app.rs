use holochain_manager::versions::HolochainVersion;

use crate::launcher::state::LauncherState;

#[tauri::command]
pub async fn open_app_ui(
  state: tauri::State<'_, LauncherState>,
  holochain_version: HolochainVersion,
  app_id: String,
) -> Result<(), String> {
  let mut mutex = (*state).lock().await;
  let manager = mutex.get_running()?;

  manager
    .open_app(holochain_version, &app_id)
    .map_err(|err| format!("Error opening app: {}", err))?;

  log::info!("Opening app {}", app_id.clone(),);

  Ok(())
}

#[tauri::command]
pub fn report_issue() -> () {
  open_url("https://github.com/holochain/launcher/issues/new?assignees=&labels=bug&template=bug_report.md&title=".into()).unwrap();
}

#[tauri::command]
pub fn open_url(url: String) -> Result<(), String> {
  tauri::async_runtime::spawn(async move {
    open::that(url.clone().as_str()).map_err(|err| format!("Could not open url: {}", err))
  });

  Ok(())
}
