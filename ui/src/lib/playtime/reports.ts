import { formatSeconds } from "../api/api";
import { t, rich, formatDate } from "../i18n/i18n.svelte";
import { PlaytimeStatus, SessionEndReason, TrackingCoverage } from "./playtime.constants";
import { rankLabel } from "../ranks/ranks";
import type { MessagePartType } from "../i18n/messages";
import type { AccountType, ReportType, SessionRowType } from "../types";

export const reportSummary = (report: ReportType): MessagePartType[] =>
  rich(t.report_summary, {
    time: formatSeconds(report.total_seconds),
    games: t.report_games_count({ count: report.game_count }),
    count: report.machine
      ? t.accounts_count({ count: report.account_count })
      : t.report_sessions_count({ count: report.session_count }),
  });

export const reportStatistics = (report: ReportType): MessagePartType[] =>
  rich(t.report_statistics, {
    since: formatDate(report.statistics.history_started_at),
    sessions: report.statistics.session_count,
    average: formatSeconds(report.statistics.average_session_seconds),
    longest: formatSeconds(report.statistics.longest_session_seconds),
  });

export const reportNotice = (report: ReportType): MessagePartType[] => {
  const parts = rich(t.report_history, { since: formatDate(report.statistics.history_started_at) });
  if (!report.machine)
    parts.push({ text: " · " }, ...rich(t.report_rank, { rank: rankLabel(report.rank) }));
  return parts;
};

export const statusLabel = (status: PlaytimeStatus): string =>
  status === PlaytimeStatus.RUNNING ? t.report_status_running() : "";

export const coverageLabel = (coverage: TrackingCoverage): string =>
  coverage === TrackingCoverage.FULL ? t.report_coverage_full() : t.report_coverage_partial();

const endReasonLabels: Record<SessionEndReason, () => string> = {
  [SessionEndReason.ACTIVE]: () => t.common_unknown(),
  [SessionEndReason.UNKNOWN]: () => t.common_unknown(),
  [SessionEndReason.STEAM_RUNNING_LIST]: () => t.session_end_steam_stopped(),
  [SessionEndReason.ROOT_PROCESS_EXIT_LOG]: () => t.session_end_root_process_exit(),
  [SessionEndReason.PROCESS_EXIT]: () => t.session_end_process_exit(),
  [SessionEndReason.PROCESS_MISSING_AFTER_RESTART]: () => t.session_end_missing_after_restart(),
  [SessionEndReason.REPLACED_BY_NEW_LAUNCH]: () => t.session_end_new_launch(),
  [SessionEndReason.TRACKING_DISABLED]: () => t.session_end_tracking_disabled(),
};

export const endReasonLabel = (reason: SessionEndReason): string => endReasonLabels[reason]();

export const sessionEnd = (session: SessionRowType): string => {
  if (session.status === PlaytimeStatus.RUNNING) return t.report_status_running();
  const reason = endReasonLabel(session.end_reason);
  return session.exit_code === null
    ? reason
    : t.session_end_with_code({ reason, code: session.exit_code });
};

export const sessionTooltip = (session: SessionRowType): string =>
  [
    t.session_id({ value: session.session_id }),
    t.session_account({
      value: session.account_name === "******" ? t.common_hidden_value() : session.account_name,
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

export const playtimeTooltip = (entry: AccountType): string =>
  entry.session_count === 0
    ? t.playtime_empty()
    : t.playtime_details({
        time: formatSeconds(entry.playtime_seconds),
        sessions: t.report_sessions_count({ count: entry.session_count }),
      });
