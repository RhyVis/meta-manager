export type GamePlatform = {
  platform: string;
  id?: string;
};

export enum PlatformType {
  Steam = "Steam",
  DLSite = "DLSite",
  Other = "Other",
  Unknown = "Unknown",
}

export const createGamePlatform = (type: PlatformType, otherID?: string): GamePlatform => {
  switch (type) {
    case PlatformType.Steam:
      return { platform: "Steam" };
    case PlatformType.DLSite:
      return { platform: "DLSite" };
    case PlatformType.Other:
      return { platform: "Other", id: otherID || "" };
    default:
      return { platform: "Unknown" };
  }
};

export type GameTag = {
  name: string;
  category?: string;
};

export type GameMetadata = {
  id: string;
  title: string;
  original_title?: string;
  platform: GamePlatform;
  platform_id?: string;

  description?: string;
  version?: string;
  developer?: string;
  publisher?: string;
  release_date?: string;

  archive_path?: string;
  archive_password?: string;
  deployed_path?: string;
  size_bytes?: number;

  tags?: GameTag[];

  date_created?: string;
  date_updated?: string;
};

export type GameLibrary = {
  entries: GameMetadata[];
};
