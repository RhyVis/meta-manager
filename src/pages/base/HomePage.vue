<script setup lang="ts">
import TyParagraph from "@/components/typography/TyParagraph.vue";
import TySubtitle from "@/components/typography/TySubtitle.vue";
import TyTitle from "@/components/typography/TyTitle.vue";
import PageLayout from "@/layout/PageLayout.vue";
import { useRouter } from "vue-router";

const { push } = useRouter();

const navigateToDashboard = () => {
  push("/dashboard");
};
</script>

<template>
  <PageLayout>
    <ty-paragraph class="r-no-sel">
      <ty-title>M.E.T.A.</ty-title>
      <ty-subtitle>某种元数据管理器。</ty-subtitle>
    </ty-paragraph>

    <div class="q-mt-xl flex justify-center">
      <q-btn
        class="q-px-xl q-py-sm"
        color="primary"
        label="进入管理界面"
        icon="dashboard"
        size="lg"
        rounded
        @click="navigateToDashboard"
      />
    </div>
  </PageLayout>
</template>
