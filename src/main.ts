import { createApp } from "vue";
import "@material/mwc-textfield";
import "@material/mwc-textarea";
import "@material/mwc-fab";
import "@material/mwc-snackbar";
//import "@material/mwc-button";
//import "@material/mwc-dialog";

import { AdminWebsocket, AppWebsocket } from "@holochain/conductor-api";
import HcAdminPlugin from "@holochain/admin-ui";
import { invoke } from "@tauri-apps/api/tauri";
import "blob-polyfill";

import App from "./App.vue";
import createStore from "./store";

window.onerror = function (message, source, lineno, colno, error) {
  invoke("log", {
    log: `UI error: message: ${message}. source: ${source}. lineno: ${lineno}. colno: ${colno}. error: ${JSON.stringify(
      error
    )}`,
  });
};

async function setup() {
  const app = createApp(App);

  try {
    const adminPort = await invoke("get_admin_port", {});

    const adminWebsocket = await AdminWebsocket.connect(
      `ws://localhost:${adminPort}`
    );

    const appInterfaces = await adminWebsocket.listAppInterfaces();

    const port = appInterfaces[0];

    const appWebsocket = await AppWebsocket.connect(`ws://localhost:${port}`);

    const store = createStore(true);
    app.use(store as any);
    app
      .use(HcAdminPlugin as any, { store, appWebsocket, adminWebsocket })
      .mount("#app");

    await invoke("log", {
      log: `Connected to Holochain, Admin port = ${adminPort}, App port = ${port}`,
    });
  } catch (e) {
    const error = `Error connecting to Holochain: ${e}`;

    await invoke("log", { log: error });
    app.use(createStore(false) as any);
    app.mount("#app");
  }
}

setup();
