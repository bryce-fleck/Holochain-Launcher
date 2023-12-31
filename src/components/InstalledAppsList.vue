<template>

  <!-- TODO: remove this, but add little dot to settings when there are ui updates to do -->
  <HCGenericDialog
    @confirm="updateGui"
    ref="updateGuiDialog"
    :primaryButtonLabel="$t('buttons.install')"
    :closeOnSideClick="true"
  >
  <div class="column" style="padding: 0 30px; align-items: flex-start; max-width: 500px;">
    <div style="width: 100%; text-align: center; font-weight: 600; font-size: 27px; margin-bottom: 25px">
      {{ $t("dialogs.guiUpdate.title") }}
    </div>
    <div style="margin-bottom: 15px;">
      {{ $t("dialogs.guiUpdate.mainText") }}:
    </div>
    <div>
      <span style="font-weight: bold; margin-right: 15px;">{{ $t("dialogs.guiUpdate.version") }}:</span>{{ selectedGuiUpdate ? selectedGuiUpdate.version : "loading..." }}
    </div>
    <div style="font-weight: bold;">
      {{ $t("dialogs.guiUpdate.changelog") }}:
    </div>
    <div style="background: rgb(217,217,217); border-radius: 8px; padding: 10px; width: 480px; min-height: 100px; max-height: 200px; overflow-y: auto; margin-top: 5px; white-space: pre-wrap;">
      {{ selectedGuiUpdate ? selectedGuiUpdate.changelog : "loading..." }}
    </div>
    <div style="margin-top: 20px;">
      {{ $t("dialogs.guiUpdate.question") }}
    </div>
  </div>

  </HCGenericDialog>

  <HCLoading ref="downloading" :text="loadingText" />

  <HCSnackbar
    :labelText="errorText"
    ref="snackbar"
  ></HCSnackbar>

  <div
    style="
      display: flex;
      flex: 1;
      flex-direction: column;
      margin-bottom: 80px;
      padding: 0 30px;
      width: 70%;
      align-items: center;
      min-width: 900px;
    "
  >

    <!-- Web Apps -->
    <div
      class="row section-title"
      style="margin-top: -18px"
    >
      <span
        @click="showWebApps = !showWebApps"
        class="show-hide"
        style="opacity: 0.7; cursor: pointer; margin-left: 10px"
      >
        &nbsp;<!-- {{ showWebApps ? "[-]" : "[show]" }} -->
      </span>
    </div>
    <div v-if="showWebApps" style="margin-bottom: 50px; width: 100%">
      <div
        v-if="noWebApps"
        style="margin-top: 30px; color: rgba(0, 0, 0, 0.6); text-align: center"
      >
        {{ $t("main.noApps") }}
        {{
          selectedHolochainVersion === "All Versions"
            ? "."
            : " in this Holochain Version."
        }}
      </div>

      <div
        v-else
        class="app-grid-container"
      >
        <div
          v-for="app in sortedApps"
          :key="app.webAppInfo.installed_app_info.installed_app_id"
        >
          <InstalledAppCard
            v-if="app.webAppInfo.web_uis.default.type !== 'Headless'"
            :app="app"
            @openApp="$emit('openApp', $event)"
            @uninstallApp="$emit('uninstall-app', $event)"
            @disableApp="$emit('disable-app', $event)"
            @enableApp="$emit('enable-app', $event)"
            @startApp="$emit('startApp', $event)"
            @updateGui="openUpdateGuiDialog($event)"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, PropType } from "vue";
import { uniq } from "lodash-es";

import "@material/mwc-button";
import "@material/mwc-icon-button";
import "@material/mwc-icon";

import { HolochainAppInfo, HolochainAppInfoExtended, ResourceLocator, StorageInfo } from "../types";
import { getCellId, isAppRunning } from "../utils";
import InstalledAppCard from "./InstalledAppCard.vue";
import HCSelectCard from "./subcomponents/HCSelectCard.vue";
import StackedChart from "./subcomponents/StackedChart.vue";
import HCGenericDialog from "./subcomponents/HCGenericDialog.vue";
import HCLoading from "./subcomponents/HCLoading.vue";
import { invoke } from "@tauri-apps/api/tauri";
import prettyBytes from "pretty-bytes";
import HCSnackbar from "./subcomponents/HCSnackbar.vue";
import { getHappReleasesByEntryHashes, fetchGui, appstoreCells, fetchGuiReleaseEntry } from "../appstore/appstore-interface";
import { AppInfo, AppWebsocket, decodeHashFromBase64, DnaHashB64, encodeHashToBase64, EntryHash, InstalledAppId } from "@holochain/client";
import { Entity, FilePackage, GUIReleaseEntry, HappReleaseEntry } from "../appstore/types";
import { ActionTypes } from "../store/actions";
import { i18n } from "../locale";
import { APPSTORE_APP_ID, DEVHUB_APP_ID } from "../constants";
import { locatorToLocatorB64 } from "../utils";


export default defineComponent({
  name: "InstalledAppsList",
  components: {
    InstalledAppCard,
    HCSelectCard,
    StackedChart,
    HCGenericDialog,
    HCLoading,
    HCSnackbar,
  },
  props: {
    installedApps: {
      type: Object as PropType<Array<HolochainAppInfo>>,
      required: true,
    },
  },
  data(): {
    appWebsocket: AppWebsocket | undefined;
    appstoreAppInfo: AppInfo | undefined;
    sortOptions: [string, string][];
    sortOption: string | undefined;
    selectedHolochainVersion: string;
    showHeadlessApps: boolean;
    showWebApps: boolean;
    showHolochainVersions: boolean;
    refreshing: boolean;
    refreshTimeout: number | null;
    extendedAppInfos: Record<InstalledAppId, HolochainAppInfoExtended> | undefined;
    selectedApp: HolochainAppInfoExtended | undefined;
    selectedGuiUpdate: GUIReleaseEntry | undefined;
    selectedGuiUpdateLocator: ResourceLocator | undefined;
    loadingText: string;
    errorText: string;
  } {
    return {
      appWebsocket: undefined,
      appstoreAppInfo: undefined,
      sortOptions: [
        [i18n.global.t('main.name'), "name"],
        [i18n.global.t('main.nameDescending'), "name descending"],
        // ["Holochain Version", "Holochain Version"],
      ],
      sortOption: undefined,
      selectedHolochainVersion: "All Versions",
      showHeadlessApps: true,
      showWebApps: true,
      showHolochainVersions: true,
      refreshing: false,
      refreshTimeout: null,
      extendedAppInfos: undefined,
      selectedApp: undefined,
      selectedGuiUpdate: undefined,
      selectedGuiUpdateLocator: undefined,
      loadingText: "",
      errorText: "Unknown error occured",
    };
  },
  emits: ["openApp", "uninstall-app", "enable-app", "disable-app", "startApp"],
  async mounted() {
    const holochainId = this.$store.getters["holochainIdForDevhub"];
    // connect to AppWebsocket
    const port = this.$store.getters["appInterfacePort"](holochainId);
    const appWebsocket = await AppWebsocket.connect(`ws://localhost:${port}`, 40000);
    this.appWebsocket = appWebsocket;
    // TODO add correct installed app id here.
    const appstoreAppInfo = await appWebsocket.appInfo({
        installed_app_id: APPSTORE_APP_ID,
    });
    this.appstoreAppInfo = appstoreAppInfo;

    const extendedAppInfos: Record<InstalledAppId, HolochainAppInfoExtended> = {};

    // TODO: do i need this here?
    this.installedApps.forEach((app) => {
      extendedAppInfos[app.webAppInfo.installed_app_info.installed_app_id] = {
        webAppInfo: app.webAppInfo,
        holochainId: app.holochainId,
        holochainVersion: app.holochainVersion,
        guiUpdateAvailable: undefined,
      }
    });

    this.extendedAppInfos = extendedAppInfos;

    await this.checkForUiUpdates();
  },
  computed: {
    sortedApps() {
      // if extended happ releases are not yet fetched from the DevHub to include potential
      // GUI updates, just return installedApps with guiUpdateAvailable undefined
      let sortedAppList: Array<HolochainAppInfoExtended> = this.extendedAppInfos
          ? Object.values(this.extendedAppInfos)
          : this.installedApps.map((app) => {
        return {
          webAppInfo: app.webAppInfo,
          holochainId: app.holochainId,
          holochainVersion: app.holochainVersion,
          guiUpdateAvailable: undefined,
        }
      });

      // Filter out App Store and DevHub
      sortedAppList = sortedAppList.filter(
        (app) => app.webAppInfo.installed_app_info.installed_app_id !== APPSTORE_APP_ID && app.webAppInfo.installed_app_info.installed_app_id !== DEVHUB_APP_ID
      );

      if (this.selectedHolochainVersion !== "All Versions") {
        sortedAppList = sortedAppList.filter(
          (app) => app.holochainVersion === this.selectedHolochainVersion
        );
      }

      if (this.sortOption === "name") {
        sortedAppList = sortedAppList.sort((appA, appB) =>
          appA.webAppInfo.installed_app_info.installed_app_id.localeCompare(
            appB.webAppInfo.installed_app_info.installed_app_id
          )
        );
      } else if (this.sortOption === "name descending") {
        sortedAppList = sortedAppList.sort((appA, appB) =>
          appB.webAppInfo.installed_app_info.installed_app_id.localeCompare(
            appA.webAppInfo.installed_app_info.installed_app_id
          )
        );
      } else {
        // default is alphabetical by app id
        sortedAppList = sortedAppList.sort((appA, appB) =>
          appA.webAppInfo.installed_app_info.installed_app_id.localeCompare(
            appB.webAppInfo.installed_app_info.installed_app_id
          )
        );
      }

      return sortedAppList;
    },
    noWebApps(): boolean {
      return this.sortedApps.every(
        (app) => app.webAppInfo.web_uis.default.type === "Headless"
      );
    },
    holochainVersions(): string[] {
      const allApps = this.installedApps;
      return uniq(allApps.map((app) => app.holochainVersion));
    },
    holochainVersionOptions(): [string, string][] {
      let allApps = this.installedApps;
      let hcVersions: [string, string][] = [[i18n.global.t('main.allVersions'), "All Versions"]];
      uniq(allApps.map((app) => app.holochainVersion)).forEach((hcVer) => {
        hcVersions.push([hcVer, hcVer]);
      });
      return hcVersions;
    },
  },
  methods: {
    prettyBytes,
    isAppRunning,
    isAppHeadless(app: HolochainAppInfo) {
      return app.webAppInfo.web_uis.default.type === "Headless";
    },
    /**
     * This checks for UI updates for all apps that have a known happ release hash
     *
     */
    async checkForUiUpdates() {
      console.log("Checking for UI updates...");
      // check for GUI updates
      const allApps: Array<HolochainAppInfo> = this.$store.getters["allApps"];

      const updatableApps = allApps.filter((app) => app.webAppInfo.happ_release_info?.resource_locator);

      // sort all happ release ResourceLocators by DnaHash of the DevHub they originate from
      const updatableAppsByLocatorDna: Record<DnaHashB64, HolochainAppInfo[]> = {};

      updatableApps.forEach((app) => {
        const dnaHash = app.webAppInfo.happ_release_info!.resource_locator!.dna_hash;
        const apps = updatableAppsByLocatorDna[dnaHash];

        if (apps) {
          updatableAppsByLocatorDna[dnaHash] = [...apps, app]
        } else {
          updatableAppsByLocatorDna[dnaHash] = [app!]
        }
      });

      await Promise.allSettled(Object.values(updatableAppsByLocatorDna).map(async (apps) => {
        const entryHashes = apps.map((app) => decodeHashFromBase64(app.webAppInfo.happ_release_info!.resource_locator!.resource_hash));
        const devHubDnaHash = decodeHashFromBase64(apps[0].webAppInfo.happ_release_info!.resource_locator!.dna_hash);

        try {
          console.log("@checkForUiPudates: entryHashes: ", entryHashes.map((eh) => encodeHashToBase64(eh)));
          const happReleases: Array<HappReleaseEntry | undefined> = await getHappReleasesByEntryHashes((this.appWebsocket! as AppWebsocket), this.appstoreAppInfo!, devHubDnaHash, entryHashes);

          apps.forEach((app, idx) => {
            if (happReleases[idx]) {
              console.log("official_gui: ", happReleases[idx]!.official_gui ? encodeHashToBase64(happReleases[idx]!.official_gui!) : undefined)
            }

            // if it's installed as a webapp and the happ release has an official GUI, check whether it's a new GUI
            if (app.webAppInfo.web_uis.default.type === "WebApp" && happReleases[idx]?.official_gui) {
              const guiReleaseInfo = app.webAppInfo.web_uis.default.gui_release_info;
              const guiReleaseHash = app.webAppInfo.web_uis.default.gui_release_info?.resource_locator!.resource_hash;
              console.log("guiReleaseHash: ", guiReleaseHash);
              if (guiReleaseInfo && guiReleaseHash) {
                if(guiReleaseHash != encodeHashToBase64(happReleases[idx]!.official_gui!)) {
                  this.extendedAppInfos![app.webAppInfo.installed_app_info.installed_app_id].guiUpdateAvailable = {
                    dna_hash: devHubDnaHash,
                    resource_hash: happReleases[idx]!.official_gui!,
                  }
                }
              }
            }
          })

        } catch (e) {
          console.error(`Failed to get happ releases from DevHub host of network with DNA hash ${encodeHashToBase64(devHubDnaHash)}: ${JSON.stringify(e)}`);
        }

      }))

      // // console.log("@InstalledAppsList: allHappReleaseHashes from store's allApps: ", allHappReleaseHashes);
      // const happReleases: Array<HappReleaseEntry | undefined> = await getHappReleasesByEntryHashes((this.appWebsocket! as AppWebsocket), this.appstoreAppInfo!, allHappReleaseHrls);

      // console.log("@InstalledAppsList: happReleases: ", happReleases);

      // // compare with existing

      // const extendedAppInfos: Record<DnaHashB64, Array<HolochainAppInfoExtended>> = {};

      // allApps.forEach((appInfo: HolochainAppInfo, idx) => {

      //   if (happReleases[idx]) {
      //     console.log("official_gui: ", happReleases[idx]!.official_gui ? encodeHashToBase64(happReleases[idx]!.official_gui!) : undefined)
      //   }

      //   const isGuiUpdateAvailable = (appInfo.webAppInfo.web_uis.default.type === "WebApp" && happReleases[idx]?.official_gui)
      //     ? appInfo.webAppInfo.web_uis.default.gui_release_hash != encodeHashToBase64(happReleases[idx]?.official_gui!)
      //     : false

      //   return {
      //     webAppInfo: appInfo.webAppInfo,
      //     holochainId: appInfo.holochainId,
      //     holochainVersion: appInfo.holochainVersion,
      //     guiUpdateAvailable: isGuiUpdateAvailable ? happReleases[idx]?.official_gui : undefined,
      //   }
      // });

      // console.log("@InstalledAppsLlist: extendedAppInfos: ", extendedAppInfos);

      // this.extendedAppInfos = extendedAppInfos;
    },
    async openUpdateGuiDialog(app: HolochainAppInfoExtended) {
      this.selectedApp = app;

      // console.log("Gui release hash @openUpdateGuiDialog: ", app.guiUpdateAvailable);
      (this.$refs.updateGuiDialog as typeof HCGenericDialog).open();

      if (this.appWebsocket && this.appstoreAppInfo) {
          const cells = appstoreCells(this.appstoreAppInfo);
        //   const guiReleaseResponse = await this.appWebsocket?.callZome({
        //   cap_secret: null,
        //   cell_id: getCellId(cells.happs.find((c) => "provisioned" in c )!)!,
        //   fn_name: "get_gui_release",
        //   zome_name: "happ_library",
        //   payload: {
        //     id: app.guiUpdateAvailable,
        //   },
        //   provenance: getCellId(cells.happs.find((c) => "provisioned" in c )!)![1],
        // });

        const guiReleaseResponse = await fetchGuiReleaseEntry(this.appWebsocket as AppWebsocket, this.appstoreAppInfo, app.guiUpdateAvailable!);

        this.selectedGuiUpdate = guiReleaseResponse.content;
        this.selectedGuiUpdateLocator = app.guiUpdateAvailable;
        console.log("Got GUI Release: ", guiReleaseResponse.content);
      } else {
        alert!("Error: AppWebsocket or Appstore AppInfo undefined.")
        this.selectedGuiUpdate = undefined;
        this.selectedGuiUpdateLocator = undefined;
      }
    },
    async updateGui() {
      this.loadingText = "Connecting with DevHub";
      (this.$refs.downloading as typeof HCLoading).open();

      this.loadingText = "fetching UI from peer host...";

      let bytes = undefined;

      try {
        const guiResponse: Entity<FilePackage> = await fetchGui(
          this.appWebsocket! as AppWebsocket,
          this.appstoreAppInfo!,
          this.selectedGuiUpdateLocator!.dna_hash,
          this.selectedGuiUpdate!.web_asset_id,
        );
        bytes = guiResponse.content.bytes;
      } catch (e) {
        console.error("Error fetching the UI: ", e);
        this.errorText = `Error fetching the UI: ${e}`;
        (this.$refs.snackbar as typeof HCSnackbar).show();
        (this.$refs.downloading as typeof HCLoading).close();
        return;
      }

      this.loadingText = "Installing...";

      if (bytes) {
        try {
          await invoke("update_default_ui", {
            holochainId: this.selectedApp!.holochainId,
            appId: this.selectedApp!.webAppInfo.installed_app_info.installed_app_id,
            uiZipBytes: bytes,
            guiReleaseInfo: {
              resource_locator: locatorToLocatorB64(this.selectedApp!.guiUpdateAvailable!),
              version: this.selectedGuiUpdate?.version,
            },
          });
          this.loadingText = "";
          (this.$refs.downloading as typeof HCLoading).close();
          (this.$refs.updateGuiDialog as typeof HCGenericDialog).close();
          this.selectedGuiUpdate = undefined;
          this.selectedGuiUpdateLocator = undefined;

          // to remove the update button:
          await this.$store.dispatch(ActionTypes.fetchStateInfo);
          window.setTimeout(() => this.checkForUiUpdates(), 500);
        } catch (e) {
          console.error("Error updating the UI: ", e);
          this.errorText = `Error updating the UI: ${e}`;

          (this.$refs as any).snackbar.show();
          (this.$refs.downloading as typeof HCLoading).close();
          this.loadingText = "";
        }
      } else {
        console.error("Error updating the UI: Undefined bytes");
        this.errorText = `Error updating the UI: Undefined bytes`;

        (this.$refs as any).snackbar.show();
        (this.$refs.downloading as typeof HCLoading).close();
        this.loadingText = "";
      }
    },
  },
});
</script>

<style scoped>
.show-hide:hover {
  color: black;
}
.section-title {
  width: 98%;
  margin: 10px;
  max-width: 1080px;
  padding-bottom: 3px;
  align-items: center;
}

.borderBottomed {
  border-bottom: 2px solid rgba(0, 0, 0, 0.4);
}

.app-grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  /* This is better for small screens, once min() is better supported */
  /* grid-template-columns: repeat(auto-fill, minmax(min(200px, 100%), 1fr)); */
  gap: 1rem;
}
</style>
