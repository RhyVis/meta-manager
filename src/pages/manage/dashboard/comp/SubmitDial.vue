<script setup lang="ts">
import { computed, ref } from "vue";
import { PlatformType } from "@/lib/bridge.ts";
import { get, set, useToggle } from "@vueuse/core";
import { type QForm, useQuasar, type ValidationRule } from "quasar";
import { openSelectFile, openSelectFolder } from "@/lib/api.ts";
import { command_metadata_add, command_metadata_create } from "@/lib/command.ts";
import { useLibraryStore } from "@/stores/library.ts";
import { extractFilenameFromPath } from "@/pages/manage/dashboard/script.ts";
import { generateRandomPassword } from "@/lib/util.ts";

const value = defineModel<boolean>();

const { notify } = useQuasar();
const [loading, setLoading] = useToggle(false);
const library = useLibraryStore();

const formRef = ref<QForm>();

const enum FormMode {
  ArchiveAdd = 0,
  CompressionNew = 1,
}

const formMode = ref(FormMode.ArchiveAdd);
const formModeIsArchive = computed(() => formMode.value == FormMode.ArchiveAdd);

const formPlatform = ref<string>(PlatformType.Unknown);
const formTitle = ref("");
const formAppId = ref("");
const formPath = ref("");
const formUsePassword = ref(false);
const formPassword = ref("");

const validateAppId: ValidationRule = () => {
  if (formPlatform.value == PlatformType.Unknown) {
    return true;
  } else {
    return formAppId.value.length > 0 || "请输入APPID";
  }
};
const fillPassword = () => {
  if (get(formModeIsArchive)) return;
  if (formPassword.value) {
    set(formPassword, generateRandomPassword(12));
  } else {
    set(formPassword, "META");
  }
};

const handleSelFile = async () => {
  const path = await openSelectFile();
  if (path) {
    formPath.value = path;
    if (!formTitle.value) {
      set(formTitle, extractFilenameFromPath(path, true));
    }
  } else {
    notify({
      type: "warning",
      message: "未选择文件",
    });
  }
};
const handleSelDir = async () => {
  const path = await openSelectFolder();
  if (path) {
    formPath.value = path;
    if (!formTitle.value) {
      set(formTitle, extractFilenameFromPath(path, false));
    }
  } else {
    notify({
      type: "warning",
      message: "未选择文件夹",
    });
  }
};
const handleSubmit = async () => {
  if (!formRef.value) {
    notify({
      type: "warning",
      message: "未知页面元素错误",
      caption: "尝试刷新页面",
    });
    return;
  }
  const validation = await formRef.value.validate();
  if (!validation) {
    notify({
      type: "negative",
      message: "请检查输入",
    });
    return;
  }
  try {
    setLoading(true);
    if (get(formModeIsArchive)) {
      await command_metadata_add({
        title: get(formTitle),
        archivePath: get(formPath),
        info: {
          name: get(formPlatform),
          id: get(formAppId),
        },
      });
    } else {
      notify({
        message: "压缩数据中……",
        position: "bottom-right",
      });
      if (get(formUsePassword) && formPassword.value) {
        await command_metadata_create({
          title: get(formTitle),
          fromPath: get(formPath),
          info: {
            name: get(formPlatform),
            id: get(formAppId),
          },
          password: get(formPassword),
        });
      } else {
        await command_metadata_create({
          title: get(formTitle),
          fromPath: get(formPath),
          info: {
            name: get(formPlatform),
            id: get(formAppId),
          },
        });
      }
    }
    await library.getLibrary();
    set(formTitle, "");
    set(formAppId, "");
    set(formPath, "");
    set(formPassword, "");
    set(formPlatform, PlatformType.Unknown);
    notify({
      type: "positive",
      message: "元数据已添加",
      position: "bottom-right",
    });
    set(value, false);
  } catch (e) {
    notify({
      type: "negative",
      message: "提交信息失败",
      caption: e as string,
    });
  } finally {
    setLoading(false);
  }
};
</script>

<template>
  <q-dialog v-model="value">
    <q-card style="min-width: 450px">
      <q-card-section>
        <div class="text-h6">添加元数据</div>
      </q-card-section>

      <q-card-section>
        <q-form class="q-gutter-sm" @submit.prevent="handleSubmit" ref="formRef">
          <q-btn-toggle
            class="q-mb-md"
            v-model="formMode"
            spread
            no-caps
            rounded
            :disable="loading"
            :options="[
              { value: FormMode.ArchiveAdd, slot: 'left' },
              { value: FormMode.CompressionNew, slot: 'right' },
            ]"
          >
            <template #left>
              <div class="row items-center no-wrap">
                <div class="text-center">现有存储存档</div>
                <q-icon right size="sm" name="archive" />
              </div>
            </template>
            <template #right>
              <div class="row items-center no-wrap">
                <div class="text-center">新建压缩存档</div>
                <q-icon right size="sm" name="folder_zip" />
              </div>
            </template>
          </q-btn-toggle>

          <q-field class="q-mb-lg" stack-label outlined dense label="平台">
            <div class="q-gutter-md">
              <q-radio
                v-model="formPlatform"
                :val="PlatformType.Steam"
                :label="PlatformType.Steam"
                :disable="loading"
              />
              <q-radio
                v-model="formPlatform"
                :val="PlatformType.DLSite"
                :label="PlatformType.DLSite"
                :disable="loading"
              />
            </div>
          </q-field>

          <q-input
            v-model="formTitle"
            outlined
            label="标题"
            :loading="loading"
            :disable="loading"
            lazy-rules
            :rules="[(val) => !!val || '请输入标题']"
          />

          <q-input
            v-model="formAppId"
            outlined
            label="APPID"
            :loading="loading"
            :disable="loading"
            lazy-rules
            :rules="[validateAppId]"
          >
            <template #append>
              <q-btn flat dense icon="shuffle" no-caps @click="formAppId = '-'">
                <q-tooltip>
                  <div>填充占位符</div>
                </q-tooltip>
              </q-btn>
            </template>
          </q-input>

          <q-input
            v-model="formPassword"
            v-if="!formModeIsArchive && formUsePassword"
            outlined
            label="密码"
            clearable
            :loading="loading"
            :disable="loading"
            lazy-rules
            :rules="[(val) => !formUsePassword || !!val || '请输入密码']"
          >
            <template #append>
              <q-btn flat dense icon="password" no-caps @click="fillPassword">
                <q-tooltip>
                  <div>随机生成密码</div>
                </q-tooltip>
              </q-btn>
            </template>
          </q-input>

          <q-input
            v-model="formPath"
            outlined
            :label="formModeIsArchive ? '存档路径' : '压缩路径'"
            :loading="loading"
            :disable="loading"
            readonly
            lazy-rules
            :rules="[(val) => !!val || '请选择存档路径']"
          >
            <template #append>
              <q-btn-dropdown
                v-if="formMode == FormMode.ArchiveAdd"
                flat
                dense
                icon="folder"
                no-caps
              >
                <q-list>
                  <q-item v-close-popup clickable @click="handleSelFile">
                    <q-item-section>选择文件</q-item-section>
                  </q-item>
                  <q-item v-close-popup clickable @click="handleSelDir">
                    <q-item-section>选择文件夹</q-item-section>
                  </q-item>
                </q-list>
              </q-btn-dropdown>
              <q-btn v-else flat dense icon="folder" no-caps @click="handleSelDir" />
            </template>
          </q-input>
        </q-form>
      </q-card-section>

      <q-card-actions align="right">
        <q-toggle
          v-if="!formModeIsArchive"
          v-model="formUsePassword"
          :disable="loading"
          icon="lock"
        />
        <q-btn
          flat
          label="取消"
          color="grey"
          :loading="loading"
          :disable="loading"
          @click="value = false"
        />
        <q-btn
          flat
          label="保存"
          color="primary"
          :loading="loading"
          :disable="loading"
          @click="handleSubmit"
        />
      </q-card-actions>
    </q-card>
  </q-dialog>
</template>
