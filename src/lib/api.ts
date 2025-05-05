import { open } from "@tauri-apps/plugin-dialog";
import { Notify } from "quasar";

export async function openSelectFile(): Promise<string | null> {
  try {
    return await open({
      multiple: false,
      directory: false,
      filters: [
        {
          name: "所有文件",
          extensions: ["*"],
        },
      ],
    });
  } catch (e) {
    console.error(e);
    Notify.create({
      type: "negative",
      message: "选择文件失败",
      caption: e as string,
    });
    return Promise.reject(e);
  }
}

export async function openSelectFolder(): Promise<string | null> {
  try {
    return await open({
      multiple: false,
      directory: true,
    });
  } catch (e) {
    console.error(e);
    Notify.create({
      type: "negative",
      message: "选择文件夹失败",
      caption: e as string,
    });
    return Promise.reject(e);
  }
}
