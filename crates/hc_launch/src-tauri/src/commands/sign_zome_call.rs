use holochain_types::prelude::ZomeCallUnsigned;
use holochain_conductor_api::ZomeCall;
use lair_keystore_api::LairClient;
use std::collections::HashMap;
use holochain_launcher_utils::zome_call_signing::{ZomeCallUnsignedTauri, sign_zome_call_with_client};

use crate::error::HcLaunchError;



#[tauri::command]
pub async fn sign_zome_call(
  window: tauri::Window,
  state: tauri::State<'_, HashMap<String, LairClient>>,
  zome_call_unsigned: ZomeCallUnsignedTauri,
) -> Result<ZomeCall, HcLaunchError> {

  let window_label = window.label().to_string();

  // if window_label != "admin" {
  //   () // this function is allowed to be called by any window
  // }

  let unsigned_zome_call_converted: ZomeCallUnsigned = zome_call_unsigned.into();

  // get the right lair client from the hashmap
  let client = (*state).get(&window_label)
    .expect(format!("No lair client for this window with label '{}'", window_label).as_str());

  // sign the zome call
  sign_zome_call_with_client(
    unsigned_zome_call_converted,
    client,
  )
  .await
  .map_err(|e| HcLaunchError::SignZomeCallError(e))
}




