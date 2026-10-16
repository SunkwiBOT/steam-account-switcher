import type { Language, ResolvedLanguageType } from "./i18n/i18n.constants";
import type { InstallKind } from "./accounts/installations.constants";
import type {
  PlaytimeStatus,
  SessionEndReason,
  TrackingCoverage,
} from "./playtime/playtime.constants";
import type { RankKind } from "./ranks/ranks.constants";
import type {
  ConfirmationKind,
  GuardKind,
  LinkConfirmKind,
  LinkStatus,
} from "./steam-guard/steam-guard.constants";
import type { SearchScope, UpdateInterval, VaultLockInterval } from "./state/state.constants";

export type InstallationType = {
  index: number;
  kind: InstallKind;
  root: string;
};

export type InstallationSettingType = {
  root: string;
  kind: InstallKind;
  custom: boolean;
  hidden: boolean;
};

export type SettingsType = {
  language: Language;
  resolved_language: ResolvedLanguageType;
  show_installation_selector: boolean;
  tracking_enabled: boolean;
  tracking_history: TrackingHistoryType[];
  close_to_tray: boolean;
  update_interval: UpdateInterval;
  search_scope: SearchScope;
  vault_lock_interval: VaultLockInterval;
  last_update_check_at: number | null;
  entries: InstallationSettingType[];
  database_path: string;
};

export type TrackingHistoryType = {
  installation: TrackingInstallationType | null;
  account_count: number;
};

export type TrackingInstallationType = {
  kind: InstallKind;
  root: string;
};

export type LogSnapshotType = {
  path: string;
  content: string;
  truncated: boolean;
  previous_available: boolean;
};

export type RankEntryType = {
  value: string;
  family: string;
};

export type RankTemplateType = {
  id: string;
  label: string;
  kind: RankKind;
  max_rating: number | null;
  ranks: RankEntryType[];
};

export type AccountType = {
  steam_id: string;
  account_name: string;
  persona_name: string;
  display_name: string;
  initial: string;
  avatar: string | null;
  rank: string;
  rank_template: string;
  rank_kind: RankKind;
  rank_family: string;
  playtime_seconds: number;
  last_login_seconds: number;
  is_current: boolean;
  is_local: boolean;
  session_count: number;
  color: string | null;
  note: string | null;
  folder_id: string | null;
  has_authenticator: boolean;
  has_authenticator_session: boolean;
};

export type ToastAccountType = Pick<
  AccountType,
  "avatar" | "initial" | "display_name" | "persona_name"
>;

export type ToastFolderType = Pick<FolderType, "name" | "color" | "icon" | "system">;

export type VaultStatusType = {
  path: string;
  lock_in_seconds: number | null;
  encrypted: boolean;
  locked: boolean;
  indexed: boolean;
  count: number;
  error: string | null;
};

export type StoredAuthenticatorType = {
  steam_id: string;
  account_name: string;
  serial_number: string;
  device_id: string;
  has_tokens: boolean;
};

export type CodeInfoType = {
  valid_until: number;
  code: string;
  seconds_remaining: number;
  period: number;
};

export type ConfirmationItemType = {
  id: string;
  nonce: string;
  kind: ConfirmationKind;
  headline: string;
  summary: string;
  creator_id: string;
  creation_time: number;
  icon: string | null;
  multi: boolean;
};

export type GuardMethodType = {
  kind: GuardKind;
  message: string;
};

export type LoginBeginType = {
  session_id: string;
  methods: GuardMethodType[];
  qr_svg: string | null;
  challenge_url: string | null;
};

export type LoginResultType = {
  session_id: string;
  steam_id: string;
  account_name: string;
  stored: boolean;
};

export type LinkStartType = {
  session_id: string;
  status: LinkStatus;
  steam_id: string;
  account_name: string;
  revocation_code: string | null;
  phone_hint: string | null;
  confirm_kind: LinkConfirmKind | null;
};

export type FolderType = {
  id: string;
  name: string;
  account_count: number;
  system: boolean;
  color: string | null;
  icon: string | null;
};

export type ActiveGameType = {
  app_id: string;
  name: string;
  account_id: string;
  elapsed_seconds: number;
};

export type SnapshotType = {
  revision: number;
  installations: InstallationType[];
  selected: number;
  accounts: AccountType[];
  auto_login_user: string;
  masked: boolean;
  steam_ids_visible: boolean;
  machine_seconds: number;
  active_games: ActiveGameType[];
  tracker_ready: boolean;
  steam_running: boolean;
  rank_templates: RankTemplateType[];
  settings: SettingsType;
  folders: FolderType[];
  vault: VaultStatusType;
};

export type PlaytimeCellType = {
  steam_id: string;
  seconds: number;
  session_count: number;
};

export type PlaytimeSnapshotType = {
  machine_seconds: number;
  tracker_ready: boolean;
  accounts: PlaytimeCellType[];
  active_games: ActiveGameType[];
};

export type GameRowType = {
  app_id: string;
  name: string;
  sessions: number;
  seconds: number;
  status: PlaytimeStatus;
};

export type MachineGameRowType = {
  app_id: string;
  name: string;
  accounts: number;
  sessions: number;
  seconds: number;
  status: PlaytimeStatus;
};

export type AccountPlaytimeRowType = {
  account_id: string;
  display_name: string;
  account_name: string;
  initial: string;
  avatar: string | null;
  games: number;
  seconds: number;
  status: PlaytimeStatus;
};

export type SessionRowType = {
  session_id: string;
  account_id: string;
  account_name: string;
  display_name: string;
  initial: string;
  avatar: string | null;
  game_name: string;
  app_id: string;
  started_at: number;
  ended_at: number | null;
  launch_timestamp: number;
  process_create_time: number | null;
  root_pid: number | null;
  coverage: TrackingCoverage;
  end_reason: SessionEndReason;
  exit_code: number | null;
  status: PlaytimeStatus;
  seconds: number;
};

export type DailyPointType = {
  day: string;
  seconds: number;
  sessions: number;
};

export type ReportType = {
  account_id: string | null;
  total_seconds: number;
  game_count: number;
  account_count: number;
  session_count: number;
  rank: string;
  rank_template: string;
  statistics: {
    history_started_at: number;
    session_count: number;
    average_session_seconds: number;
    longest_session_seconds: number;
  };
  games: GameRowType[];
  machine_games: MachineGameRowType[];
  accounts: AccountPlaytimeRowType[];
  daily7: DailyPointType[];
  daily30: DailyPointType[];
  daily90: DailyPointType[];
  sessions: SessionRowType[];
  machine: boolean;
};

export type UpdateInfoType = {
  version: string;
  current_version: string;
  notes: string | null;
};

export type UpdateProgressType = {
  downloaded: number;
  total: number | null;
};

export type AppInfoType = {
  name: string;
  version: string;
  repository: string;
};

export type AvatarSourceType = {
  avatar: string | null;
  initial: string;
  persona_name?: string;
};

export type TableCellType = {
  text: string;
  title?: string;
  avatar?: AvatarSourceType;
};

export type BackupImportType = {
  imported: number;
  skipped: number;
  snapshot: SnapshotType;
};
