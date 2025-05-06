<script setup lang="ts">
import { type Metadata } from "@/lib/bridge.ts";
import { command_library_del, command_library_set } from "@/lib/command.ts";
import DetailDial from "@/pages/manage/dashboard/comp/DetailDial.vue";
import { dashboardColumns, formatPath } from "@/pages/manage/dashboard/script.ts";
import { useLibraryStore } from "@/stores/library.ts";
import { set, useToggle } from "@vueuse/core";
import { storeToRefs } from "pinia";
import { useQuasar } from "quasar";
import { computed, onMounted, ref } from "vue";
import SubmitDial from "@/pages/manage/dashboard/comp/SubmitDial.vue";

const { notify } = useQuasar();
const [loading, setLoading] = useToggle(false);
const [submitDialog, setSubmitDialog] = useToggle(false);
const [detailDialog, setDetailDialog] = useToggle(false);

const detailMetadata = ref<Metadata | null>(null);

const libraryStore = useLibraryStore();
const visibleColumns = ref(["title", "platform", "size", "actions"]);
const pagination = ref({ rowsPerPage: 10 });
const { lib } = storeToRefs(libraryStore);

const searchText = ref("");
const filteredEntries = computed(() => {
  if (!searchText.value) return lib.value?.entries ?? [];
  const search = searchText.value.toLowerCase();
  return (
    lib.value?.entries.filter((item) => {
      return (
        item.title?.toLowerCase().includes(search) ||
        item.original_title?.toLowerCase().includes(search) ||
        item.platform.platform?.toLowerCase().includes(search) ||
        item.platform.id?.toLowerCase().includes(search) ||
        item.platform_id?.toLowerCase().includes(search) ||
        item.developer?.toLowerCase().includes(search) ||
        item.publisher?.toLowerCase().includes(search)
      );
    }) ?? []
  );
});
const handleOpenDetailDialog = (metadata: Metadata) => {
  console.log("Opening detail dialog for", metadata);
  set(detailMetadata, metadata);
  setDetailDialog(true);
};
const handleReload = async () => {
  try {
    setLoading(true);
    await libraryStore.reload();
    notify({
      type: "positive",
      message: "库已重新加载",
      position: "bottom-right",
    });
  } finally {
    setLoading(false);
  }
};
const handleUpdate = async (replacer: Metadata) => {
  console.log(`Updating metadata: ${replacer.id}`);
  try {
    setLoading(true);
    await command_library_set(replacer);
    set(detailMetadata, replacer);
    await libraryStore.getLibrary();
    notify({
      type: "positive",
      message: "元数据已更新",
      position: "bottom-right",
    });
  } catch (e) {
    console.error("Failed to update metadata:", e);
    notify({
      type: "negative",
      message: "更新元数据失败",
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
    await libraryStore.getLibrary();
    notify({
      type: "positive",
      message: "元数据已删除",
      position: "bottom-right",
    });
    setLoading(false);
    setDetailDialog(false);
  } catch (e) {
    console.error("Failed to delete metadata:", e);
    notify({
      type: "negative",
      message: "删除元数据失败",
      caption: e as string,
    });
  } finally {
    setLoading(false);
  }
};

onMounted(() => {
  setLoading(true);
  try {
    libraryStore.getLibrary();
  } catch (e) {
    console.error("Failed to get library:", e);
    notify({
      type: "negative",
      message: "获取库失败",
      caption: e as string,
    });
  } finally {
    setLoading(false);
  }
});
</script>

<template>
  <div class="q-pa-md col">
    <!-- 标题栏 -->
    <div class="row items-center q-mb-md r-no-sel">
      <h5 class="q-my-none">元数据库</h5>
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

    <!-- 控制区域 -->
    <div class="row items-center q-mb-md q-gutter-sm">
      <!-- 搜索输入框 -->
      <q-input
        class="col-grow"
        v-model="searchText"
        dense
        outlined
        placeholder="搜索（标题、原始标题、平台ID、开发者、发行商）"
        clearable
      >
        <template v-slot:prepend>
          <q-icon name="search" />
        </template>
      </q-input>

      <!-- 列显示控制按钮 -->
      <q-btn color="blue-grey" icon="view_column" outline>
        <q-menu>
          <q-list dense style="min-width: 200px">
            <q-item
              v-for="column in dashboardColumns.filter(
                (col) => col.name != 'title' && col.name != 'actions',
              )"
              tag="label"
              :key="column.name"
            >
              <q-item-section>
                <q-checkbox v-model="visibleColumns" :val="column.name" :label="column.label" />
              </q-item-section>
            </q-item>
          </q-list>
        </q-menu>
      </q-btn>
    </div>

    <!-- 表格区域 -->
    <q-table
      v-model:pagination="pagination"
      :rows="filteredEntries"
      :columns="dashboardColumns"
      :visible-columns="visibleColumns"
      :loading="loading"
      row-key="id"
      flat
      bordered
    >
      <template #no-data>
        <div class="full-width row flex-center q-pa-md text-grey-8 r-no-sel">暂无数据，请导入</div>
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
            @click.stop="handleOpenDetailDialog(props.row as Metadata)"
          >
            <q-tooltip class="r-no-sel">查看详情，右键以删除</q-tooltip>
            <q-popup-proxy cover context-menu transition-show="scale" transition-hide="scale">
              <q-card>
                <q-card-section>
                  <div class="text-subtitle2">确定要删除 {{ props.row.title }} 吗？</div>
                </q-card-section>
                <q-card-actions align="center">
                  <q-btn v-close-popup flat dense color="gery-7" icon="cancel" />
                  <q-btn
                    v-close-popup
                    flat
                    dense
                    color="negative"
                    icon="check"
                    @click="handleDelete(props.row.id)"
                  />
                </q-card-actions>
              </q-card>
            </q-popup-proxy>
          </q-btn>
        </q-td>
      </template>

      <template #loading>
        <q-inner-loading showing color="primary" />
      </template>
    </q-table>
  </div>

  <!-- 详情对话框 -->
  <DetailDial
    v-model="detailDialog"
    :metadata="detailMetadata"
    @update="handleUpdate"
    @delete="handleDelete"
    @close="setDetailDialog(false)"
  />

  <!-- 添加对话框 -->
  <SubmitDial v-model="submitDialog" />
</template>
