export enum ContentType {
  Unknown = "Unknown",
  Game = "Game",
  Comic = "Comic",
  Novel = "Novel",
  Music = "Music",
  Anime = "Anime",
}

export const contentTypeOptions = Object.entries(ContentType).map(([key, _]) => key);

export type Platform = {
  platform: string;
  id?: string;
};

export enum PlatformType {
  Steam = "Steam",
  DLSite = "DLSite",
  Other = "Other",
  Unknown = "Unknown",
}

export const createPlatform = (type: PlatformType, otherID?: string): Platform => {
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

export type Tag = {
  name: string;
  category?: string;
};

export type Metadata = {
  id: string;
  title: string;
  original_title?: string;
  content_type: ContentType;
  platform: Platform;
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

  tags?: Tag[];

  date_created?: string;
  date_updated?: string;
};

export type Library = {
  entries: Metadata[];
};
