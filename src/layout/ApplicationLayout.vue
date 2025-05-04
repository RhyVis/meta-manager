<script setup lang="ts">
import { useToggle } from "@vueuse/core";

const [drawerOpen, setDrawerOpen] = useToggle(false);
</script>

<template>
  <q-layout view="hHh Lpr lFf" id="main-layout">
    <q-header elevated id="main-header">
      <q-toolbar>
        <q-btn flat dense round icon="menu" aria-label="Menu" @click="setDrawerOpen()" />
        <q-toolbar-title><span> M.E.T.A. </span></q-toolbar-title>
      </q-toolbar>
    </q-header>
    <q-drawer v-model="drawerOpen">
      <q-btn to="dashboard">Switch Test</q-btn>
    </q-drawer>
    <q-page-container id="main-container">
      <RouterView />
    </q-page-container>
  </q-layout>
</template>
