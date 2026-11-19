import { formatSeconds } from "../api/api";
import { t, rich, formatDate } from "../i18n/i18n.svelte";
import { rankLabel } from "../ranks/ranks";
import type { MessagePartType } from "../i18n/messages";
import type { AccountDto } from "../bindings/AccountDto";
import type { ReportDto } from "../bindings/ReportDto";
import type { SessionDto } from "../bindings/SessionDto";
import type { PlaytimeStatus } from "../bindings/PlaytimeStatus";
import type { TrackingMode } from "../bindings/TrackingMode";
import type { EndReason } from "../bindings/EndReason";
import { MASKED_LABEL } from "../accounts/identity";

export const reportSummary = (report: ReportDto): MessagePartType[] =>
  rich(t.report_summary, {
    time: formatSeconds(report.total_seconds),
    games: t.report_games_count({ count: report.game_count }),
    count: report.machine
      ? t.accounts_count({ count: report.account_count })
      : t.report_sessions_count({ count: report.session_count }),
  });

export const reportStatistics = (report: ReportDto): MessagePartType[] =>
  rich(t.report_statistics, {
    since: formatDate(report.statistics.history_started_at),
    sessions: report.statistics.session_count,
    average: formatSeconds(report.statistics.average_session_seconds),
    longest: formatSeconds(report.statistics.longest_session_seconds),
  });

export const reportNotice = (report: ReportDto): MessagePartType[] => {
  const parts = rich(t.report_history, { since: formatDate(report.statistics.history_started_at) });
  if (!report.machine)
    parts.push({ text: " · " }, ...rich(t.report_rank, { rank: rankLabel(report.rank) }));
  return parts;
};

export const statusLabel = (status: PlaytimeStatus): string =>
  status === "RUNNING" ? t.report_status_running() : "";

export const coverageLabel = (coverage: TrackingMode): string =>
  coverage === "FULL" ? t.report_coverage_full() : t.report_coverage_partial();

const endReasonLabels: Record<EndReason, () => string> = {
  UNKNOWN: () => t.common_unknown(),
  STEAM_RUNNING_LIST: () => t.session_end_steam_stopped(),
  ROOT_PROCESS_EXIT_LOG: () => t.session_end_root_process_exit(),
  PROCESS_EXIT: () => t.session_end_process_exit(),
  PROCESS_MISSING_AFTER_RESTART: () => t.session_end_missing_after_restart(),
  REPLACED_BY_NEW_LAUNCH: () => t.session_end_new_launch(),
  TRACKING_DISABLED: () => t.session_end_tracking_disabled(),
};

export const endReasonLabel = (reason: EndReason): string => endReasonLabels[reason]();

export const sessionEnd = (session: SessionDto): string => {
  if (session.status === "RUNNING") return t.report_status_running();
  const reason = endReasonLabel(session.end_reason ?? "UNKNOWN");
  return session.exit_code === null
    ? reason
    : t.session_end_with_code({ reason, code: session.exit_code });
};

export const sessionTooltip = (session: SessionDto): string =>
  [
    t.session_id({ value: session.session_id }),
    t.session_account({
      value: session.account_name === MASKED_LABEL ? t.common_hidden_value() : session.account_name,
    }),
    t.session_launch({ value: formatDate(session.launch_timestamp) }),
    t.session_counted_from({ value: formatDate(session.started_at) }),
    t.session_pid({ value: session.root_pid ?? t.common_unavailable() }),
    t.session_created({
      value:
        session.process_create_time === null
          ? t.common_unavailable()
          : formatDate(session.process_create_time),
    }),
    t.session_coverage({ value: coverageLabel(session.coverage) }),
    t.session_end({ value: sessionEnd(session) }),
  ].join("\n");

export const playtimeTooltip = (entry: AccountDto): string =>
  entry.session_count === 0
    ? t.playtime_empty()
    : t.playtime_details({
        time: formatSeconds(entry.playtime_seconds),
        sessions: t.report_sessions_count({ count: entry.session_count }),
      });
