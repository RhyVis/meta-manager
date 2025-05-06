<script setup lang="ts">
import { openSelectFile, openSelectFolder } from "@/lib/api.ts";
import { createPlatform, type Metadata, PlatformType } from "@/lib/bridge.ts";
import { command_library_deploy, command_library_deploy_off } from "@/lib/command.ts";
import {
  formatByteSize,
  metadataDeployed,
  platformOptions,
} from "@/pages/manage/dashboard/script.ts";
import { useLibraryStore } from "@/stores/library.ts";
import { openPath } from "@tauri-apps/plugin-opener";
import { useToggle } from "@vueuse/core";
import dayjs from "dayjs";
import { cloneDeep } from "lodash-es";
import { useQuasar } from "quasar";
import { computed, ref, watch } from "vue";

const { dialog, notify } = useQuasar();
const libraryStore = useLibraryStore();

const props = defineProps<{
  metadata: Metadata | null;
}>();
const value = defineModel({
  type: Boolean,
  required: true,
});
const emit = defineEmits<{
  delete: [id: string];
  update: [metadata: Metadata];
  close: [];
}>();

const [loading, setLoading] = useToggle(false);
const deployed = computed(() => {
  if (!props.metadata) return false;
  return metadataDeployed(props.metadata);
});

const editedData = ref<Partial<Metadata>>({});
const editFields = ref({
  title: { state: false, cache: "" },
  original_title: { state: false, cache: "" },
  platform: { state: false, cache: { platform: "Unknown", id: null } },
  platform_id: { state: false, cache: "" },
  description: { state: false, cache: "" },
  version: { state: false, cache: "" },
  developer: { state: false, cache: "" },
  publisher: { state: false, cache: "" },
  archive_password: { state: false, cache: "" },
  release_date: { state: false, cache: "" },
});
const platformProxy = computed({
  get: () => editedData.value.platform?.platform || "Unknown!",
  set: (value) => {
    switch (value) {
      case "Steam":
        editedData.value.platform = createPlatform(PlatformType.Steam);
        break;
      case "DLSite":
        editedData.value.platform = createPlatform(PlatformType.DLSite);
        break;
      case "Other":
        editedData.value.platform = createPlatform(PlatformType.Other);
        break;
      default:
        editedData.value.platform = createPlatform(PlatformType.Unknown);
        break;
    }
  },
});
const platformOtherID = ref("-");

const toggleEditField = <K extends keyof typeof editFields.value>(field: K) => {
  const fieldData = editFields.value[field];
  if (!fieldData.state) {
    fieldData.state = true;
    fieldData.cache = cloneDeep(fieldData.cache);
  } else {
    fieldData.state = false;
    if (field === "platform") {
      if (editedData.value.platform?.platform === "Other") {
        editedData.value.platform!.id = platformOtherID.value;
      } else {
        editedData.value.platform!.id = undefined;
      }
    }
    const changed = JSON.stringify(fieldData.cache) !== JSON.stringify(editedData.value[field]);
    if (changed) {
      updateField(field, editedData.value[field] as unknown);
    }
  }
};
const cancelEdit = <K extends keyof typeof editFields.value>(field: K) => {
  const fieldData = editFields.value[field];
  fieldData.state = false;
  editedData.value[field] = cloneDeep(fieldData.cache) as never;
};
const updateField = (field: keyof typeof editFields.value, value: unknown) => {
  if (!props.metadata) return;
  const updateData = { ...props.metadata, [field]: value };
  console.log("Changed field:", field, "to", value);
  emit("update", updateData);
};
const handleOpenPath = async (path: string | undefined) => {
  if (!path) {
    console.warn("No path provided");
    notify({
      type: "warning",
      message: "没有提供路径",
    });
    return;
  }
  try {
    await openPath(path);
  } catch (e) {
    console.error(`Failed to open path '${path}'`, e);
    notify({
      type: "negative",
      message: `打开路径失败: ${path}`,
      caption: e as string,
    });
  }
};
const handleUpdateArchivePath = async (fileMode: boolean = true) => {
  try {
    const selected = fileMode ? await openSelectFile() : await openSelectFolder();
    if (!selected || !props.metadata) return;
    const updateData = { ...props.metadata, archive_path: selected };
    emit("update", updateData);
  } catch (e) {
    console.error("Error selecting file/folder:", e);
  }
};
const handleDeploy = async () => {
  try {
    if (!props.metadata) return;
    const selected = await openSelectFolder();
    if (!selected) {
      console.warn("No folder selected");
      notify({
        type: "warning",
        message: "没有选择文件夹",
      });
      return;
    }

    setLoading(true);
    notify({
      message: "部署中……",
      position: "bottom-right",
    });
    await command_library_deploy(props.metadata.id, selected);
    await libraryStore.getLibrary();

    notify({
      type: "positive",
      message: "部署成功",
      position: "bottom-right",
    });

    emit("close");
  } catch (e) {
    console.error("Error deploying:", e);
    notify({
      type: "negative",
      message: "部署失败",
      caption: e as string,
    });
  } finally {
    setLoading(false);
  }
};
const handleDeployOff = async () => {
  if (!props.metadata) return;
  console.log("Off deploying folder:", props.metadata);
  try {
    dialog({
      title: "取消部署",
      message: `确定要取消部署 "${props.metadata.title}" 吗？该路径的文件夹将被清空！`,
      persistent: true,
      cancel: true,
    }).onOk(async () => {
      setLoading(true);
      try {
        await command_library_deploy_off(props.metadata!.id);
      } catch (e) {
        console.error("Error off deploying:", e);
        notify({
          type: "negative",
          message: "取消部署失败",
          caption: e as string,
        });
        await libraryStore.getLibrary();
        setLoading(false);
        emit("close");
        return;
      }
      await libraryStore.getLibrary();
      setLoading(false);
      notify({
        type: "positive",
        message: "取消部署成功",
        position: "bottom-right",
      });
      emit("close");
    });
  } catch (e) {
    console.error("Error off deploying folder:", e);
  } finally {
    setLoading(false);
  }
};
const handleDelete = () => {
  if (!props.metadata) return;
  dialog({
    title: "删除记录",
    message: `确定要删除元数据 "${props.metadata.title}" 吗？`,
    persistent: true,
    cancel: true,
  }).onOk(() => {
    emit("delete", props.metadata!.id);
  });
};

watch(
  () => props.metadata,
  () => {
    if (!props.metadata) return;
    Object.keys(editFields.value).forEach((key) => {
      editFields.value[key as keyof typeof editFields.value].state = false;
    });
    editedData.value = { ...props.metadata };
  },
  { immediate: true },
);
</script>

<template>
  <q-dialog v-model="value">
    <q-card style="min-width: 500px; max-width: 85vw">
      <q-card-section class="row items-center">
        <div class="text-h6">元数据详情</div>
        <q-space />
        <q-btn v-close-popup icon="close" flat round dense />
      </q-card-section>

      <q-card-section>
        <template v-if="metadata">
          <div class="q-gutter-md">
            <q-field v-if="!editFields.title.state" dense label="标题" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.title }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('title')" />
              </template>
            </q-field>
            <q-input
              v-else
              v-model="editedData.title"
              dense
              stack-label
              label="标题"
              @keyup.enter="toggleEditField('title')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('title')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('title')" />
              </template>
            </q-input>

            <q-field v-if="!editFields.original_title.state" dense label="原始标题" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.original_title || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('original_title')" />
              </template>
            </q-field>
            <q-input
              v-else
              v-model="editedData.original_title"
              dense
              stack-label
              label="原始标题"
              @keyup.enter="toggleEditField('original_title')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('original_title')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('original_title')" />
              </template>
            </q-input>

            <q-field v-if="!editFields.platform.state" dense label="平台" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.platform?.platform || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('platform')" />
              </template>
            </q-field>
            <template v-else>
              <q-select v-model="platformProxy" :options="platformOptions" dense label="平台">
                <template #after>
                  <q-btn icon="check" flat round dense @click="toggleEditField('platform')" />
                  <q-btn icon="close" flat round dense @click="cancelEdit('platform')" />
                </template>
              </q-select>
              <q-input
                v-if="editedData.platform?.platform === 'Other'"
                v-model="platformOtherID"
                dense
                stack-label
                label="其它类型"
              />
            </template>

            <q-field v-if="!editFields.platform_id.state" dense label="平台ID" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.platform_id || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('platform_id')" />
              </template>
            </q-field>
            <q-input
              v-else
              v-model="editedData.platform_id"
              dense
              stack-label
              label="平台ID"
              @keyup.enter="toggleEditField('platform_id')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('platform_id')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('platform_id')" />
              </template>
            </q-input>

            <q-input
              v-if="!editFields.description.state"
              dense
              label="描述"
              type="textarea"
              autogrow
              readonly
              stack-label
              :model-value="metadata.description || '-'"
            >
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('description')" />
              </template>
            </q-input>
            <q-input
              v-else
              v-model="editedData.description"
              dense
              stack-label
              autogrow
              type="textarea"
              label="描述"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('description')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('description')" />
              </template>
            </q-input>

            <q-field v-if="!editFields.version.state" dense label="版本" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.version || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('version')" />
              </template>
            </q-field>
            <q-input
              v-else
              v-model="editedData.version"
              dense
              stack-label
              label="版本"
              @keyup.enter="toggleEditField('version')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('version')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('version')" />
              </template>
            </q-input>

            <q-field v-if="!editFields.developer.state" dense label="开发者" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.developer || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('developer')" />
              </template>
            </q-field>
            <q-input
              v-else
              v-model="editedData.developer"
              dense
              stack-label
              label="开发者"
              @keyup.enter="toggleEditField('developer')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('developer')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('developer')" />
              </template>
            </q-input>

            <q-field v-if="!editFields.publisher.state" dense label="发行商" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.publisher || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('publisher')" />
              </template>
            </q-field>
            <q-input
              v-else
              v-model="editedData.publisher"
              dense
              stack-label
              label="发行商"
              @keyup.enter="toggleEditField('publisher')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('publisher')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('publisher')" />
              </template>
            </q-input>

            <q-field dense label="大小" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ formatByteSize(metadata.size_bytes) }}
                  <q-tooltip>
                    {{ metadata.size_bytes }}
                  </q-tooltip>
                </div>
              </template>
            </q-field>

            <q-field dense label="存档路径" stack-label>
              <template #control>
                <div
                  class="self-center full-width no-outline text-wrap"
                  @click="handleOpenPath(metadata.archive_path)"
                >
                  {{ metadata.archive_path || "-" }}
                </div>
              </template>
              <template #after>
                <q-btn-dropdown flat dense icon="folder" no-caps>
                  <q-list dense>
                    <q-item v-close-popup clickable @click="handleUpdateArchivePath(true)">
                      <q-item-section>选择文件</q-item-section>
                    </q-item>
                    <q-item v-close-popup clickable @click="handleUpdateArchivePath(false)">
                      <q-item-section>选择文件夹</q-item-section>
                    </q-item>
                  </q-list>
                </q-btn-dropdown>
              </template>
            </q-field>

            <q-field v-if="!editFields.archive_password.state" dense label="存档密码" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.archive_password || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('archive_password')" />
              </template>
            </q-field>
            <q-input
              v-else
              v-model="editedData.archive_password"
              dense
              stack-label
              label="存档密码"
              @keyup.enter="toggleEditField('archive_password')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('archive_password')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('archive_password')" />
              </template>
            </q-input>

            <q-field dense label="部署路径" stack-label>
              <template #control>
                <div
                  class="self-center full-width no-outline text-wrap"
                  @click="handleOpenPath(metadata.deployed_path)"
                >
                  {{ metadata.deployed_path || "未部署" }}
                </div>
              </template>
              <template v-if="metadata.deployed_path && metadata.deployed_path.length !== 0" #after>
                <q-btn icon="close" flat round dense @click="handleDeployOff">
                  <q-tooltip>取消部署</q-tooltip>
                </q-btn>
              </template>
            </q-field>

            <q-field dense label="标签" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  <template v-if="!deployed">
                    <q-chip
                      v-for="tag in metadata.tags"
                      :key="tag.name"
                      size="sm"
                      color="primary"
                      text-color="white"
                    >
                      {{ tag.name }}
                      <q-tooltip v-if="tag.category">{{ tag.category }}</q-tooltip>
                    </q-chip>
                  </template>
                  <template v-else>无</template>
                </div>
              </template>
            </q-field>

            <q-field stack-label dense label="发行日期">
              <template #control>
                <div class="self-center full-width no-outline">
                  {{
                    editFields.release_date.state
                      ? editedData.release_date
                        ? `${editedData.release_date} (编辑中)`
                        : "未选择"
                      : metadata.release_date
                        ? dayjs(metadata.release_date).format("YYYY/MM/DD")
                        : "-"
                  }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense>
                  <q-popup-proxy
                    cover
                    transition-show="scale"
                    transition-hide="scale"
                    @before-show="toggleEditField('release_date')"
                  >
                    <q-date v-model="editedData.release_date" minimal>
                      <div class="row items-center justify-end q-gutter-sm">
                        <q-btn
                          v-close-popup
                          icon="close"
                          color="primary"
                          flat
                          @click="cancelEdit('release_date')"
                        />
                        <q-btn
                          v-close-popup
                          icon="check"
                          color="primary"
                          flat
                          @click="toggleEditField('release_date')"
                        />
                      </div>
                    </q-date>
                  </q-popup-proxy>
                </q-btn>
              </template>
            </q-field>

            <q-field dense label="创建日期" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{
                    metadata.date_created
                      ? dayjs(metadata.date_created).format("YYYY/MM/DD HH:mm:ss")
                      : "-"
                  }}
                </div>
              </template>
            </q-field>

            <q-field dense label="更新日期" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{
                    metadata.date_updated
                      ? dayjs(metadata.date_updated).format("YYYY/MM/DD HH:mm:ss")
                      : "-"
                  }}
                </div>
              </template>
            </q-field>
          </div>
        </template>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn
          v-show="deployed"
          flat
          color="warning"
          icon="file_download_off"
          label="取消部署"
          :loading="loading"
          @click="handleDeployOff"
        />
        <q-btn
          v-show="!deployed"
          flat
          color="primary"
          icon="file_download"
          label="部署"
          :disable="!metadata?.archive_path"
          :loading="loading"
          @click="handleDeploy"
        />
        <q-btn
          flat
          color="negative"
          icon="delete"
          label="删除记录"
          :loading="loading"
          @click="handleDelete"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
