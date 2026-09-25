/* Payloads exchanged with Rust, mirroring src-tauri/src/commands.rs. */

export interface Installation {
  index: number;
  label: string;
  kind: string;
  root: string;
}

export interface InstallationSetting {
  root: string;
  label: string;
  kind: string;
  custom: boolean;
  hidden: boolean;
}

export interface Settings {
  show_installation_selector: boolean;
  show_machine_playtime: boolean;
  entries: InstallationSetting[];
  settings_path: string;
}

export interface RankEntry {
  value: string;
  family: string;
}

export interface RankTemplate {
  id: string;
  label: string;
  kind: "tiered" | "rating";
  max_rating: number | null;
  ranks: RankEntry[];
}

export interface Account {
  steam_id: string;
  account_name: string;
  persona_name: string;
  display_name: string;
  initial: string;
  avatar: string | null;
  rank: string;
  rank_template: string;
  rank_kind: string;
  rank_family: string;
  playtime_seconds: number;
  playtime_text: string;
  playtime_tooltip: string;
  last_login_text: string;
  last_login_absolute: string;
  last_login_seconds: number;
  is_current: boolean;
  session_count: number;
  color: string | null;
  folder_id: string | null;
}

export interface Folder {
  id: string;
  name: string;
  account_count: number;
}

export interface ActiveGame {
  app_id: string;
  name: string;
  account_id: string;
  elapsed_text: string;
}

export interface Snapshot {
  installations: Installation[];
  selected: number;
  accounts: Account[];
  auto_login_user: string;
  masked: boolean;
  steam_ids_visible: boolean;
  machine_seconds: number;
  machine_text: string;
  active_games: ActiveGame[];
  tracker_ready: boolean;
  status: string;
  steam_running: boolean;
  rank_templates: RankTemplate[];
  settings: Settings;
  folders: Folder[];
}

export interface PlaytimeCell {
  steam_id: string;
  seconds: number;
  text: string;
  session_count: number;
}

export interface PlaytimeSnapshot {
  machine_seconds: number;
  machine_text: string;
  tracker_ready: boolean;
  accounts: PlaytimeCell[];
  active_games: ActiveGame[];
}

export interface GameRow {
  app_id: string;
  name: string;
  sessions: number;
  seconds: number;
  seconds_text: string;
  status: string;
}

export interface MachineGameRow {
  app_id: string;
  name: string;
  accounts: number;
  sessions: number;
  seconds: number;
  seconds_text: string;
  status: string;
}

export interface AccountPlaytimeRow {
  account_id: string;
  display_name: string;
  account_name: string;
  initial: string;
  avatar: string | null;
  games: number;
  seconds: number;
  seconds_text: string;
  status: string;
}

export interface SessionRow {
  session_id: string;
  account_id: string;
  account_name: string;
  display_name: string;
  initial: string;
  avatar: string | null;
  game_name: string;
  app_id: string;
  started_text: string;
  ended_text: string;
  duration_text: string;
  coverage: string;
  end_reason: string;
  status: string;
  seconds: number;
  tooltip: string;
}

export interface DailyPoint {
  day: string;
  seconds: number;
  sessions: number;
}

export interface Report {
  account_id: string | null;
  title: string;
  subtitle: string;
  summary_text: string;
  statistics_text: string;
  notice_text: string;
  games: GameRow[];
  machine_games: MachineGameRow[];
  accounts: AccountPlaytimeRow[];
  daily7: DailyPoint[];
  daily30: DailyPoint[];
  daily90: DailyPoint[];
  sessions: SessionRow[];
  machine: boolean;
}

export interface UpdateInfo {
  version: string;
  current_version: string;
  notes: string | null;
}

export interface UpdateProgress {
  downloaded: number;
  total: number | null;
}

export interface AppInfo {
  name: string;
  version: string;
  repository: string;
}

/** What the generated avatars need, whatever the payload around them. */
export interface AvatarSource {
  avatar: string | null;
  initial: string;
}

/** One cell of a report table. */
export interface TableCell {
  text: string;
  title?: string;
  avatar?: AvatarSource;
}
