<script setup lang="ts">
import PageLayout from "@/layout/PageLayout.vue";
import { command_library_export, command_library_import } from "@/lib/command.ts";
import { useQuasar } from "quasar";
import { useToggle } from "@vueuse/core";
import { useLibraryStore } from "@/stores/library.ts";

const library = useLibraryStore();
const { notify } = useQuasar();
const [loading, setLoading] = useToggle(false);

const handleExport = async () => {
  try {
    notify({
      message: "正在导出数据库",
      position: "bottom-right",
      icon: "info",
    });
    setLoading(true);
    await command_library_export();
    notify({
      message: "导出数据库成功",
      color: "positive",
      icon: "check_circle",
      position: "bottom-right",
    });
  } catch (e) {
    console.log(e);
    notify({
      message: "导出数据库失败",
      color: "negative",
      icon: "error",
    });
  } finally {
    setLoading(false);
  }
};
const handleImport = async () => {
  try {
    notify({
      message: "正在导入数据库",
      position: "bottom-right",
      icon: "info",
    });
    setLoading(true);
    if (await command_library_import()) {
      notify({
        message: "导入数据库成功",
        color: "positive",
        icon: "check_circle",
        position: "bottom-right",
      });
      await library.getLibrary();
    } else {
      notify({
        message: "library.json不存在，无法导入",
        color: "negative",
        icon: "error",
      });
    }
  } catch (e) {
    console.log(e);
    notify({
      message: "导入数据库失败",
      color: "negative",
      icon: "error",
    });
  } finally {
    setLoading(false);
  }
};
</script>

<template>
  <PageLayout>
    <div class="q-gutter-md q-mx-sm">
      <div class="text-h6">库操作</div>
      <q-btn-group>
        <q-btn
          label="导出数据库"
          icon="output"
          color="primary"
          :loading="loading"
          :disable="loading"
          @click.prevent="handleExport"
        />
        <q-btn
          label="导入数据库"
          icon="exit_to_app"
          color="primary"
          :loading="loading"
          :disable="loading"
          @click.prevent="handleImport"
        />
      </q-btn-group>
    </div>
  </PageLayout>
</template>
