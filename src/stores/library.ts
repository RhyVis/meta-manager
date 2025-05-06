import type { Library } from "@/lib/bridge.ts";
import { command_library_get } from "@/lib/command.ts";
import { defineStore } from "pinia";
import { Notify } from "quasar";

interface LibraryState {
  lib: Library | null;
}

export const useLibraryStore = defineStore("library", {
  state: (): LibraryState => ({
    lib: null,
  }),
  getters: {
    empty: (state) => state.lib === null,
    size: (state) => state.lib?.entries.length ?? 0,
  },
  actions: {
    update(lib: Library) {
      console.info("Updating library");
      this.lib = lib;
    },
    clear() {
      console.info("Clearing library");
      this.lib = null;
    },
    async getLibrary() {
      console.info("Getting library");
      try {
        this.lib = await command_library_get();
      } catch (e) {
        console.error("Failed to get library", e);
        Notify.create({
          type: "negative",
          message: "获取库失败",
          caption: e as string,
        });
      }
    },
    async reload() {
      console.info("Reloading library");
      try {
        this.lib = await command_library_get();
      } catch (e) {
        console.error("Failed to reload library", e);
        Notify.create({
          type: "negative",
          message: "刷新库失败",
          caption: e as string,
        });
      }
    },
  },
});
