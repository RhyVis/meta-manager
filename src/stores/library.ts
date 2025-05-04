import { defineStore } from "pinia";
import type { GameLibrary } from "@/lib/bridge.ts";

interface LibraryState {
  lib: GameLibrary | null;
}

export const useLibraryStore = defineStore("library", {
  state: (): LibraryState => ({
    lib: null,
  }),
});
