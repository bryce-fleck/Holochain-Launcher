import { HolochainVersion, LauncherStateInfo } from "@/types";
import { invoke } from "@tauri-apps/api/tauri";
import { createStore } from "vuex";

export interface LauncherAdminState {
  launcherStateInfo: "loading" | LauncherStateInfo;
}

export const store = createStore<LauncherAdminState>({
  state() {
    return {
      launcherStateInfo: "loading",
    };
  },
  getters: {
    oldFiles(state) {
      if (state.launcherStateInfo === "loading") return false;
      return (
        state.launcherStateInfo.state.type === "Error" &&
        state.launcherStateInfo.state.content.type === "OldFilesExist"
      );
    },
    isAlreadyRunning(state) {
      if (state.launcherStateInfo === "loading") return false;
      return (
        state.launcherStateInfo.state.type === "Error" &&
        state.launcherStateInfo.state.content.type ===
          "AnotherInstanceIsAlreadyRunning"
      );
    },
    errorLaunching(state) {
      if (state.launcherStateInfo === "loading") return undefined;
      if (
        state.launcherStateInfo.state.type === "Error" &&
        state.launcherStateInfo.state.content.type === "ErrorLaunching"
      )
        return state.launcherStateInfo.state.content.content;

      if (
        state.launcherStateInfo.state.type === "Running" &&
        state.launcherStateInfo.state.content.type === "Error" &&
        state.launcherStateInfo.state.content.content.type ===
          "LaunchKeystoreError"
      ) {
        if (
          typeof state.launcherStateInfo.state.content.content.content
            .content === "string"
        ) {
          return state.launcherStateInfo.state.content.content.content.content;
        } else {
          if (
            state.launcherStateInfo.state.content.content.content.content
              .type === "BinaryNotFound"
          ) {
            return "Error finding the tauri binary";
          } else {
            return state.launcherStateInfo.state.content.content.content.content
              .content;
          }
        }
      }

      if (
        state.launcherStateInfo.state.type === "Running" &&
        state.launcherStateInfo.state.content.type === "Running"
      ) {
        const error = Object.values(
          state.launcherStateInfo.state.content.content
        ).find((v) => v.type === "Error");
        if (error) return error.content as string;
      }
    },
    setupNeeded(state) {
      if (state.launcherStateInfo === "loading") return undefined;

      return (
        state.launcherStateInfo.state.type === "Running" &&
        state.launcherStateInfo.state.content.type === "Error" &&
        state.launcherStateInfo.state.content.content.type === "InitNecessary"
      );
    },
    passwordNeeded(state) {
      if (state.launcherStateInfo === "loading") return undefined;

      return (
        state.launcherStateInfo.state.type === "Running" &&
        state.launcherStateInfo.state.content.type === "Error" &&
        state.launcherStateInfo.state.content.content.type ===
          "PasswordNecessary"
      );
    },
    holochainVersions(state) {
      const stateInfo = state.launcherStateInfo;

      if (
        stateInfo === "loading" ||
        stateInfo.state.type === "Error" ||
        stateInfo.state.content.type === "Error"
      )
        return [];

      return Object.keys(stateInfo.state.content.content);
    },
    appsForVersion: (state) => (holochainVersion: HolochainVersion) => {
      const stateInfo = state.launcherStateInfo;

      if (
        stateInfo === "loading" ||
        stateInfo.state.type === "Error" ||
        stateInfo.state.content.type === "Error"
      )
        return [];

      const appsByVersion = stateInfo.state.content.content[holochainVersion];

      if (appsByVersion.type === "Error") return [];

      // Sort apps alphabetically

      return appsByVersion.content.sort((app1, app2) => {
        if (app1.installed_app_id < app2.installed_app_id) {
          return -1;
        }
        if (app1.installed_app_id > app2.installed_app_id) {
          return 1;
        }
        return 0;
      });
    },
  },
  mutations: {
    loadStateInfo(state) {
      state.launcherStateInfo = "loading";
    },
    setStateInfo(state, StateInfo) {
      state.launcherStateInfo = StateInfo;
    },
  },
  actions: {
    async fetchStateInfo({ commit }) {
      commit("loadStateInfo");
      const StateInfo = await invoke("get_state_info", {});

      commit("setStateInfo", StateInfo);
    },
  },
});
