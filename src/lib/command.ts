import type { Library, Metadata } from "@/lib/bridge.ts";
import type {
  MetadataCreation,
  MetadataSubmit,
  PlatformInfo,
} from "@/pages/manage/dashboard/script.ts";
import { invoke } from "@tauri-apps/api/core";

export const command_library_get = async (): Promise<Library> => await invoke("library_get");

export const command_library_set = async (data: Metadata) => await invoke("library_set", { data });

export const command_library_del = async (id: string): Promise<boolean> =>
  invoke("library_del", { id });

export const command_library_deploy = async (id: string, path: string) =>
  await invoke("library_deploy", { id, path });

export const command_library_deploy_off = async (id: string) =>
  await invoke("library_deploy_off", { id });

export const command_metadata_add = async (data: MetadataSubmit) =>
  await invoke("metadata_add", data);

export const command_metadata_create = async (data: MetadataCreation) =>
  await invoke("metadata_create", data);
