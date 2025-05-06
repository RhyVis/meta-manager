<script setup lang="ts">
import AppLayoutSide from "@/layout/partial/AppLayoutSide.vue";
import { useToggle } from "@vueuse/core";

const [drawerOpen, setDrawerOpen] = useToggle(false);
</script>

<template>
  <q-layout id="main-layout" view="hHh Lpr lFf">
    <q-header id="main-header" elevated>
      <q-toolbar>
        <q-btn flat dense round icon="menu" aria-label="Menu" @click="setDrawerOpen()" />
        <q-toolbar-title><span> M.E.T.A. </span></q-toolbar-title>
      </q-toolbar>
    </q-header>
    <q-drawer v-model="drawerOpen">
      <AppLayoutSide />
    </q-drawer>
    <q-page-container id="main-container">
      <RouterView />
    </q-page-container>
  </q-layout>
</template>
