<script setup lang="ts">
import { useLibraryStore } from "@/stores/library.ts";
import { onMounted } from "vue";

const libraryStore = useLibraryStore();

onMounted(() => {
  if (libraryStore.empty) {
    libraryStore.getLibrary();
  }
});
</script>

<template>
  <q-list class="q-mr-sm q-ml-xs" padding>
    <q-item clickable to="/" v-ripple>
      <q-item-section avatar>
        <q-icon name="home" />
      </q-item-section>
      <q-item-section>首页</q-item-section>
    </q-item>

    <q-item clickable to="/dashboard" v-ripple>
      <q-item-section avatar>
        <q-icon name="dashboard" />
      </q-item-section>
      <q-item-section>
        <div class="flex items-center">
          <span>管理</span>
          <q-badge v-if="libraryStore.size > 0" color="primary" class="q-ml-sm" floating>
            {{ libraryStore.size }}
          </q-badge>
        </div>
      </q-item-section>
    </q-item>
  </q-list>
</template>
