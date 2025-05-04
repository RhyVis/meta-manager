<script setup lang="ts">
import { onMounted, ref } from "vue";
import {
  command_library_del,
  command_library_get,
  command_library_reload,
  command_library_replace,
  command_metadata_add_dl,
  command_metadata_add_steam,
  command_metadata_add_unknown,
} from "@/lib/command.ts";
import { set, useToggle } from "@vueuse/core";
import type { GameMetadata } from "@/lib/bridge.ts";
import { useQuasar } from "quasar";
import {
  dashboardColumns,
  extractFilenameFromPath,
  formatPath,
} from "@/pages/manage/dashboard/script.ts";
import { open } from "@tauri-apps/plugin-dialog";
import { storeToRefs } from "pinia";
import { useLibraryStore } from "@/stores/library.ts";
import DetailDial from "@/pages/manage/dashboard/comp/DetailDial.vue";

const { notify } = useQuasar();
const [loading, setLoading] = useToggle(false);
const [submitLoading, setSubmitLoading] = useToggle(false);
const [submitDialog, setSubmitDialog] = useToggle(false);
const [detailDialog, setDetailDialog] = useToggle(false);

const detailMetadata = ref<GameMetadata | null>(null);

const submitForm = ref({
  title: "",
  appId: "",
  archivePath: "",
});
const submitType = ref("");

const { lib } = storeToRefs(useLibraryStore());
const visibleColumns = ref(["title", "platform", "size", "actions"]);
const pagination = ref({
  rowsPerPage: 10,
});

const openFileDialog = async () => {
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "所有文件",
          extensions: ["*"],
        },
      ],
    });

    if (selected) {
      const path = selected as string;
      submitForm.value.archivePath = path;

      if (submitForm.value.title === "") {
        submitForm.value.title = extractFilenameFromPath(path, true);
      }
    }
  } catch (e) {
    console.error("Error in selecting file:", e);
    notify({
      type: "negative",
      message: "选择文件失败",
      caption: e as string,
    });
  }
};
const openFolderDialog = async () => {
  try {
    // 只选择文件夹
    const selected = await open({
      multiple: false,
      directory: true,
    });

    if (selected) {
      const path = selected as string;
      submitForm.value.archivePath = path;

      if (submitForm.value.title === "") {
        submitForm.value.title = extractFilenameFromPath(path, false);
      }
    }
  } catch (e) {
    console.error("Error in selecting file:", e);
    notify({
      type: "negative",
      message: "选择文件夹失败",
      caption: e as string,
    });
  }
};

const handleOpenDetailDialog = (metadata: GameMetadata) => {
  console.log("Opening detail dialog for", metadata);
  set(detailMetadata, metadata);
  setDetailDialog(true);
};
const handleReload = async () => {
  try {
    console.info("Reloading library");
    setLoading(true);
    set(lib, await command_library_reload());
    notify({
      type: "positive",
      message: "游戏库已重新加载",
      position: "bottom-right",
    });
  } catch (e) {
    console.error("Failed to reload library", e);
    notify({
      type: "negative",
      message: "Failed to reload library",
      caption: e as string,
    });
    set(lib, null);
  } finally {
    setLoading(false);
  }
};
const handleSubmit = async () => {
  if (submitForm.value.title.length == 0) {
    notify({
      type: "negative",
      message: "添加元数据失败",
      caption: "标题不能为空",
    });
    return;
  }
  try {
    console.log("Adding metadata:", submitForm.value);
    setSubmitLoading(true);
    switch (submitType.value) {
      case "steam": {
        if (submitForm.value.appId.length == 0) {
          notify({
            type: "negative",
            message: "添加元数据失败",
            caption: "APPID不能为空",
          });
          return;
        }
        await command_metadata_add_steam(submitForm.value);
        break;
      }
      case "dl": {
        if (submitForm.value.appId.length == 0) {
          notify({
            type: "negative",
            message: "添加元数据失败",
            caption: "APPID不能为空",
          });
          return;
        }
        await command_metadata_add_dl(submitForm.value);
        break;
      }
      default: {
        await command_metadata_add_unknown(submitForm.value);
        break;
      }
    }
    set(lib, await command_library_get());
    set(submitForm, {
      title: "",
      appId: "",
      archivePath: "",
    });
    notify({
      type: "positive",
      message: "游戏元数据已添加",
      position: "bottom-right",
    });
  } catch (e) {
    console.error("Failed to add metadata:", e);
    notify({
      type: "negative",
      message: "添加元数据失败",
      caption: e as string,
    });
  } finally {
    setSubmitLoading(false);
    setSubmitDialog(false);
  }
};
const handleUpdate = async (replacer: GameMetadata) => {
  console.log(`Updating metadata: ${replacer.id}`);
  try {
    setLoading(true);
    await command_library_replace(replacer);
    set(detailMetadata, replacer);
    set(lib, await command_library_get());
    notify({
      type: "positive",
      message: "游戏元数据已更新",
      position: "bottom-right",
    });
  } catch (e) {
    console.error("Failed to update metadata:", e);
    notify({
      type: "negative",
      message: "更新游戏元数据失败",
      caption: e as string,
    });
  } finally {
    setLoading(false);
  }
};
const handleDelete = async (id: string) => {
  console.log(`Trying to delete ${id}`);
  try {
    setLoading(true);
    await command_library_del(id);
    set(lib, await command_library_get());
    notify({
      type: "positive",
      message: "游戏元数据已删除",
      position: "bottom-right",
    });
    setLoading(false);
    setDetailDialog(false);
  } catch (e) {
    console.error("Failed to delete metadata:", e);
    notify({
      type: "negative",
      message: "删除游戏元数据失败",
      caption: e as string,
    });
  } finally {
    setLoading(false);
  }
};

onMounted(() => {
  setLoading(true);
  command_library_get()
    .then((data) => {
      set(lib, data);
    })
    .catch((e) => {
      console.error("Failed to get library:", e);
      notify({
        type: "negative",
        message: "获取游戏库失败",
        caption: e as string,
      });
    })
    .finally(() => {
      setLoading(false);
    });
});
</script>

<template>
  <div class="q-pa-md col">
    <!-- 标题栏 -->
    <div class="row items-center q-mb-md">
      <h5 class="q-my-none">游戏库管理</h5>
      <q-space />
    </div>

    <!-- 操作按钮区域 -->
    <div class="row q-mb-md q-gutter-md">
      <q-btn
        color="primary"
        icon="refresh"
        label="重载库"
        :loading="loading"
        @click="handleReload"
      />
      <q-btn color="secondary" icon="add" label="添加元数据" @click="setSubmitDialog(true)" />
    </div>

    <!-- 列显示控制 -->
    <q-btn color="secondary" icon="view_column" flat>
      <q-menu>
        <q-list style="min-width: 200px">
          <q-item
            tag="label"
            v-for="column in dashboardColumns.filter(
              (col) => col.name != 'title' && col.name != 'actions',
            )"
            :key="column.name"
          >
            <q-item-section>
              <q-checkbox v-model="visibleColumns" :val="column.name" :label="column.label" />
            </q-item-section>
          </q-item>
        </q-list>
      </q-menu>
    </q-btn>

    <!-- 表格区域 -->
    <q-table
      :rows="lib?.entries || []"
      :columns="dashboardColumns"
      :visible-columns="visibleColumns"
      :loading="loading"
      v-model:pagination="pagination"
      row-key="id"
      flat
      bordered
    >
      <template #no-data>
        <div class="full-width row flex-center q-pa-md text-grey-8">暂无游戏数据，请导入</div>
      </template>

      <template #body-cell-archive_path="props">
        <q-td :props="props">
          <div style="max-width: 200px">
            {{ formatPath(props.value) }}
            <q-tooltip>{{ props.value }}</q-tooltip>
          </div>
        </q-td>
      </template>
      <template #body-cell-deploy_path="props">
        <q-td :props="props">
          <div style="max-width: 200px">
            {{ formatPath(props.value) }}
            <q-tooltip>{{ props.value }}</q-tooltip>
          </div>
        </q-td>
      </template>
      <template #body-cell-actions="props">
        <q-td :props="props">
          <q-btn
            flat
            round
            icon="info"
            size="sm"
            @click.stop="handleOpenDetailDialog(props.row as GameMetadata)"
          >
            <q-tooltip>查看详情</q-tooltip>
          </q-btn>
        </q-td>
      </template>

      <template #loading>
        <q-inner-loading showing color="primary" />
      </template>
    </q-table>
  </div>

  <!-- 添加游戏详情对话框 -->
  <DetailDial
    :metadata="detailMetadata"
    v-model="detailDialog"
    @update="handleUpdate"
    @delete="handleDelete"
  />

  <!-- 添加游戏对话框 -->
  <q-dialog v-model="submitDialog">
    <q-card style="min-width: 350px">
      <q-card-section>
        <div class="text-h6">添加游戏</div>
      </q-card-section>

      <q-card-section>
        <q-form @submit.prevent="handleSubmit" class="q-gutter-md">
          <q-field stack-label label="平台">
            <div class="q-gutter-md">
              <q-radio v-model="submitType" val="steam" label="Steam" />
              <q-radio v-model="submitType" val="dl" label="DLSite" />
            </div>
          </q-field>
          <q-input
            filled
            v-model="submitForm.title"
            label="标题"
            :loading="submitLoading"
            lazy-rules
            :rules="[(val) => !!val || '请输入标题']"
          />
          <q-input
            filled
            v-model="submitForm.appId"
            label="APPID"
            :loading="submitLoading"
            lazy-rules
            :rules="[(val) => !!val || '请输入APPID']"
          />
          <q-input
            filled
            v-model="submitForm.archivePath"
            label="存档路径"
            :loading="submitLoading"
            readonly
            lazy-rules
            :rules="[(val) => !!val || '请选择存档路径']"
          >
            <template v-slot:append>
              <q-btn-dropdown flat dense icon="folder" no-caps>
                <q-list>
                  <q-item clickable v-close-popup @click="openFileDialog">
                    <q-item-section>选择文件</q-item-section>
                  </q-item>
                  <q-item clickable v-close-popup @click="openFolderDialog">
                    <q-item-section>选择文件夹</q-item-section>
                  </q-item>
                </q-list>
              </q-btn-dropdown>
            </template>
          </q-input>
        </q-form>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat label="取消" color="grey" @click="setSubmitDialog(false)" />
        <q-btn flat label="保存" color="primary" @click="handleSubmit" :loading="submitLoading" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
