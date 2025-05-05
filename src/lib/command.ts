import type { GameLibrary, GameMetadata } from "@/lib/bridge.ts";
import type { MetadataNewSubmit } from "@/pages/manage/dashboard/script.ts";
import { invoke } from "@tauri-apps/api/core";

export const command_library_get = async (): Promise<GameLibrary> => await invoke("library_get");

export const command_library_del = async (id: string): Promise<boolean> =>
  invoke("library_del", { id });

export const command_library_reload = async (): Promise<GameLibrary> =>
  await invoke("library_reload");

export const command_library_replace = async (replacer: GameMetadata) =>
  await invoke("library_replace", { replacer });

export const command_library_deploy = async (id: string, path: string) =>
  await invoke("library_deploy", { id, path });

export const command_library_deploy_off = async (id: string) =>
  await invoke("library_deploy_off", { id });

export const command_metadata_add_steam = async (data: MetadataNewSubmit): Promise<boolean> =>
  await invoke("metadata_add_steam", {
    title: data.title,
    id: data.appId,
    archivePath: data.archivePath,
  });

export const command_metadata_add_dl = async (data: MetadataNewSubmit): Promise<boolean> =>
  await invoke("metadata_add_dl", {
    title: data.title,
    id: data.appId,
    archivePath: data.archivePath,
  });

export const command_metadata_add_unknown = async (data: MetadataNewSubmit): Promise<boolean> =>
  await invoke("metadata_add_unknown", {
    title: data.title,
    archivePath: data.archivePath,
  });
