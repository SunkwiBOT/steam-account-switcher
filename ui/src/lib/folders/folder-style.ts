import {
  Book,
  Crosshair,
  Crown,
  Flame,
  Folder,
  FolderOpen,
  Gamepad2,
  Gem,
  Ghost,
  Globe,
  Heart,
  Music,
  Palette,
  Rocket,
  Shield,
  Star,
  Swords,
  Target,
  Trophy,
  Zap,
} from "@lucide/svelte";
import { t } from "../i18n/i18n.svelte";
import { CARD_COLORS } from "../api/api";

export type FolderIconType = {
  id: string;
  icon: typeof Folder;
  label: () => string;
};

export const FOLDER_ICONS: FolderIconType[] = [
  { id: "folder", icon: Folder, label: () => t.folder_icon_folder() },
  { id: "folder-open", icon: FolderOpen, label: () => t.folder_icon_folder_open() },
  { id: "star", icon: Star, label: () => t.folder_icon_star() },
  { id: "heart", icon: Heart, label: () => t.folder_icon_heart() },
  { id: "flame", icon: Flame, label: () => t.folder_icon_flame() },
  { id: "zap", icon: Zap, label: () => t.folder_icon_zap() },
  { id: "trophy", icon: Trophy, label: () => t.folder_icon_trophy() },
  { id: "crown", icon: Crown, label: () => t.folder_icon_crown() },
  { id: "target", icon: Target, label: () => t.folder_icon_target() },
  { id: "crosshair", icon: Crosshair, label: () => t.folder_icon_crosshair() },
  { id: "gamepad-2", icon: Gamepad2, label: () => t.folder_icon_gamepad_2() },
  { id: "swords", icon: Swords, label: () => t.folder_icon_swords() },
  { id: "shield", icon: Shield, label: () => t.folder_icon_shield() },
  { id: "rocket", icon: Rocket, label: () => t.folder_icon_rocket() },
  { id: "ghost", icon: Ghost, label: () => t.folder_icon_ghost() },
  { id: "gem", icon: Gem, label: () => t.folder_icon_gem() },
  { id: "book", icon: Book, label: () => t.folder_icon_book() },
  { id: "music", icon: Music, label: () => t.folder_icon_music() },
  { id: "palette", icon: Palette, label: () => t.folder_icon_palette() },
  { id: "globe", icon: Globe, label: () => t.folder_icon_globe() },
];

export const folderIcon = (id: string | null | undefined): typeof Folder =>
  FOLDER_ICONS.find((entry) => entry.id === id)?.icon ?? Folder;

/// New folders get a random colour and a decorative icon, never the plain folder or no colour.
export const randomFolderStyle = (): { color: string; icon: string } => {
  const colors = CARD_COLORS.filter((color) => color);
  const icons = FOLDER_ICONS.filter((entry) => entry.id !== "folder");
  return {
    color: colors[Math.floor(Math.random() * colors.length)],
    icon: icons[Math.floor(Math.random() * icons.length)].id,
  };
};
