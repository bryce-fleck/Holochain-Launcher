import { HdkVersion } from "@/hdk";
import {
  AppWebsocket,
  EntryHash,
  InstalledAppInfo,
  InstalledCell,
} from "@holochain/client";
import { Happ, HappRelease } from "./types";

export interface ContentAddress<C> {
  address: EntryHash;
  content: C;
}

export interface AppWithReleases {
  app: ContentAddress<Happ>;
  releases: Array<ContentAddress<HappRelease>>;
}

export function filterByHdkVersion(
  hdkVersions: HdkVersion[],
  apps: Array<AppWithReleases>
): Array<AppWithReleases> {
  const filteredReleases: Array<AppWithReleases> = apps.map((app) => ({
    app: app.app,
    releases: app.releases.filter((r) =>
      hdkVersions.includes(r.content.hdk_version)
    ),
  }));

  return filteredReleases.filter((app) => app.releases.length > 0);
}

export async function getAllPublishedApps(
  appWebsocket: AppWebsocket,
  devhubHapp: InstalledAppInfo
): Promise<Array<AppWithReleases>> {
  const cells = devhubCells(devhubHapp);
  const allAppsOutput = await appWebsocket.callZome({
    cap_secret: null,
    cell_id: cells.happs.cell_id,
    fn_name: "get_all_happs",
    zome_name: "happ_library",
    payload: null,
    provenance: cells.happs.cell_id[1],
  });

  const allApps: Array<ContentAddress<Happ>> = allAppsOutput.payload.items;

  const promises = allApps.map((app) =>
    getAppsReleases(appWebsocket, devhubHapp, app)
  );

  return Promise.all(promises);
}

export async function getAppsReleases(
  appWebsocket: AppWebsocket,
  devhubHapp: InstalledAppInfo,
  app: ContentAddress<Happ>
): Promise<AppWithReleases> {
  const cells = devhubCells(devhubHapp);

  const appReleasesOutput = await appWebsocket.callZome({
    cap_secret: null,
    cell_id: cells.happs.cell_id,
    fn_name: "get_happ_releases",
    zome_name: "happ_library",
    payload: {
      for_happ: app.address,
    },
    provenance: cells.happs.cell_id[1],
  });

  const releases = appReleasesOutput.payload.items;

  return {
    app,
    releases,
  };
}

export function getLatestRelease(
  apps: AppWithReleases
): ContentAddress<HappRelease> {
  const guiReleases = apps.releases.filter((r) => !!r.content.gui);
  return guiReleases.sort(
    (r1, r2) => r1.content.last_updated - r2.content.last_updated
  )[0];
}

export async function fetchWebHapp(
  appWebsocket: AppWebsocket,
  devhubHapp: InstalledAppInfo,
  name: string,
  happReleaseEntryHash: EntryHash
): Promise<Uint8Array> {
  const cells = devhubCells(devhubHapp);

  const result = await appWebsocket.callZome({
    cap_secret: null,
    cell_id: cells.happs.cell_id,
    fn_name: "get_webhapp_package",
    zome_name: "happ_library",
    payload: {
      name,
      id: happReleaseEntryHash,
      dnarepo_dna_hash: cells.dnarepo.cell_id[0],
      webassets_dna_hash: cells.webassets.cell_id[0],
    },
    provenance: cells.happs.cell_id[1],
  });

  return result.payload;
}

function devhubCells(devhubHapp: InstalledAppInfo) {
  const happs = devhubHapp.cell_data.find((c) => c.role_id === "happs");
  const dnarepo = devhubHapp.cell_data.find((c) => c.role_id === "dnarepo");
  const webassets = devhubHapp.cell_data.find((c) => c.role_id === "webassets");

  if (!happs || !dnarepo || !webassets) throw new Error("Bad app info");

  return {
    happs,
    dnarepo,
    webassets,
  };
}
