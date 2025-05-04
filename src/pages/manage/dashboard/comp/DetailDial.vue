<script setup lang="ts">
import { createGamePlatform, type GameMetadata, PlatformType } from "@/lib/bridge.ts";
import { formatByteSize, platformOptions } from "@/pages/manage/dashboard/script.ts";
import { useQuasar } from "quasar";
import { computed, ref, watch } from "vue";
import { cloneDeep } from "lodash-es";

const { dialog } = useQuasar();

const { metadata } = defineProps<{
  metadata: GameMetadata | null;
}>();
const value = defineModel({
  type: Boolean,
  required: true,
});
const emit = defineEmits<{
  delete: [id: string];
  update: [metadata: GameMetadata];
}>();

const editedData = ref<Partial<GameMetadata>>({});
const editFields = ref({
  title: { state: false, cache: "" },
  original_title: { state: false, cache: "" },
  platform: { state: false, cache: { platform: "Unknown", id: null } },
  platform_id: { state: false, cache: "" },
  description: { state: false, cache: "" },
  version: { state: false, cache: "" },
  developer: { state: false, cache: "" },
  publisher: { state: false, cache: "" },
});
const platformProxy = computed({
  get: () => editedData.value.platform?.platform || "Unknown!",
  set: (value) => {
    switch (value) {
      case "Steam":
        editedData.value.platform = createGamePlatform(PlatformType.Steam);
        break;
      case "DLSite":
        editedData.value.platform = createGamePlatform(PlatformType.DLSite);
        break;
      case "Other":
        editedData.value.platform = createGamePlatform(PlatformType.Other);
        break;
      default:
        editedData.value.platform = createGamePlatform(PlatformType.Unknown);
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
  if (!metadata) return;
  const updateData = { ...metadata, [field]: value };
  console.log("Changed field:", field, "to", value);
  emit("update", updateData);
};

const handleDelete = () => {
  if (!metadata) return;
  dialog({
    title: "删除记录",
    message: `确定要删除游戏记录 "${metadata.title}" 吗？`,
    persistent: true,
    cancel: true,
  }).onOk(() => {
    emit("delete", metadata.id);
  });
};

watch(
  () => metadata,
  () => {
    if (!metadata) return;
    Object.keys(editFields.value).forEach((key) => {
      editFields.value[key as keyof typeof editFields.value].state = false;
    });
    editedData.value = { ...metadata };
  },
  { immediate: true },
);
</script>

<template>
  <q-dialog v-model="value">
    <q-card style="min-width: 500px; max-width: 85vw">
      <q-card-section class="row items-center">
        <div class="text-h6">游戏详情</div>
        <q-space />
        <q-btn icon="close" flat round dense v-close-popup />
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
              dense
              stack-label
              label="标题"
              v-model="editedData.title"
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
              dense
              stack-label
              label="原始标题"
              v-model="editedData.original_title"
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
              <q-select :options="platformOptions" v-model="platformProxy" dense label="平台">
                <template #append>
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
              dense
              stack-label
              label="平台ID"
              v-model="editedData.platform_id"
              @keyup.enter="toggleEditField('platform_id')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('platform_id')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('platform_id')" />
              </template>
            </q-input>

            <q-field v-if="!editFields.description.state" dense label="描述" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ metadata.description || "-" }}
                </div>
              </template>
              <template #append>
                <q-btn icon="edit" flat round dense @click="toggleEditField('description')" />
              </template>
            </q-field>
            <q-input
              v-else
              dense
              stack-label
              label="描述"
              v-model="editedData.description"
              @keyup.enter="toggleEditField('description')"
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
              dense
              stack-label
              label="版本"
              v-model="editedData.version"
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
              dense
              stack-label
              label="开发者"
              v-model="editedData.developer"
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
              dense
              stack-label
              label="发行商"
              v-model="editedData.publisher"
              @keyup.enter="toggleEditField('publisher')"
            >
              <template #append>
                <q-btn icon="check" flat round dense @click="toggleEditField('publisher')" />
                <q-btn icon="close" flat round dense @click="cancelEdit('publisher')" />
              </template>
            </q-input>

            <q-field dense label="发行日期" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{
                    metadata.release_date
                      ? new Date(metadata.release_date).toLocaleDateString()
                      : "-"
                  }}
                </div>
              </template>
            </q-field>

            <q-field dense label="大小" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{ formatByteSize(metadata.size_bytes) }}
                </div>
              </template>
            </q-field>

            <q-field dense label="存档路径" stack-label>
              <template #control>
                <div class="self-center full-width no-outline text-wrap">
                  {{ metadata.archive_path || "-" }}
                </div>
              </template>
            </q-field>

            <q-field dense label="部署路径" stack-label>
              <template #control>
                <div class="self-center full-width no-outline text-wrap">
                  {{ metadata.deployed_path || "-" }}
                </div>
              </template>
            </q-field>

            <q-field dense label="标签" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  <template v-if="metadata.tags && metadata.tags.length > 0">
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
                  <template v-else>-</template>
                </div>
              </template>
            </q-field>

            <q-field dense label="创建日期" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{
                    metadata.date_created ? new Date(metadata.date_created).toLocaleString() : "-"
                  }}
                </div>
              </template>
            </q-field>

            <q-field dense label="更新日期" stack-label>
              <template #control>
                <div class="self-center full-width no-outline">
                  {{
                    metadata.date_updated ? new Date(metadata.date_updated).toLocaleString() : "-"
                  }}
                </div>
              </template>
            </q-field>
          </div>
        </template>
      </q-card-section>

      <q-card-actions align="right">
        <q-btn flat color="negative" icon="delete" label="删除记录" @click="handleDelete" />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
