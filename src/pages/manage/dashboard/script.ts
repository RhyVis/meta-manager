import { type GameMetadata, PlatformType } from "@/lib/bridge.ts";
import type { QTableColumn } from "quasar";

export type MetadataNewSubmit = {
  title: string;
  appId: string;
  archivePath: string;
};

export const metadataDeployed = (metadata: GameMetadata | null) => {
  return metadata != null && metadata.deployed_path != null && metadata.deployed_path.length > 0;
};

export const extractFilenameFromPath = (path: string, fileMode: boolean): string => {
  const normalizedPath = path.replace(/\\/g, "/");
  const parts = normalizedPath.split("/");
  let filename = parts[parts.length - 1];

  if (fileMode && filename.includes(".")) {
    filename = filename.substring(0, filename.lastIndexOf("."));
  }

  return filename;
};
export const formatByteSize = (bytes?: number) => {
  if (bytes === undefined || bytes === null) {
    return "未知";
  }

  const units = ["B", "KB", "MB", "GB", "TB", "PB", "EB", "ZB", "YB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return unitIndex === 0 ? `${size} ${units[unitIndex]}` : `${size.toFixed(2)} ${units[unitIndex]}`;
};
export const formatPath = (path: string) => {
  if (!path || path.length <= 30) return path || "";
  const start = path.substring(0, 15);
  const end = path.substring(path.length - 15);
  return `${start}...${end}`;
};

export const dashboardColumns: QTableColumn[] = [
  {
    name: "title",
    required: true,
    label: "标题",
    align: "left",
    field: "title",
    sortable: true,
  },
  {
    name: "original_title",
    label: "原始标题",
    field: "original_title",
  },
  {
    name: "platform",
    label: "平台",
    field: (row: GameMetadata) => {
      if (!row.platform) {
        return "-";
      } else if (row.platform.platform === PlatformType.Other) {
        return `Other(${row.platform.id})`;
      } else {
        return row.platform.platform;
      }
    },
    sortable: true,
  },
  {
    name: "platform_id",
    label: "平台ID",
    field: "platform_id",
  },
  {
    name: "description",
    label: "描述",
    field: "description",
  },
  {
    name: "version",
    label: "版本",
    field: "version",
  },
  {
    name: "developer",
    label: "开发者",
    field: "developer",
  },
  {
    name: "publisher",
    label: "发行商",
    field: "publisher",
  },
  {
    name: "release_date",
    label: "发行日期",
    field: "release_date",
    sortable: true,
  },
  {
    name: "size",
    label: "大小",
    sortable: true,
    field: (row: GameMetadata) => formatByteSize(row.size_bytes),
  },
  {
    name: "archive_path",
    label: "存档路径",
    field: "archive_path",
  },
  {
    name: "deployed_path",
    label: "部署路径",
    field: "deployed_path",
  },
  {
    name: "tags",
    label: "标签",
    field: (row: GameMetadata) => row.tags?.map((tag) => tag.name).join(", ") || "",
  },
  {
    name: "date_created",
    label: "创建日期",
    field: "date_created",
    sortable: true,
    format: (val) => (val ? new Date(val).toLocaleString() : "-"),
  },
  {
    name: "date_updated",
    label: "更新日期",
    field: "date_updated",
    sortable: true,
    format: (val) => (val ? new Date(val).toLocaleString() : "-"),
  },
  {
    name: "actions",
    label: "操作",
    field: "actions",
    align: "center",
  },
];

export const platformOptions = Object.keys(PlatformType);
