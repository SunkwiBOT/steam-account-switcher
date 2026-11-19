import type { UpdateInterval } from "../bindings/UpdateInterval";

const intervals: Record<UpdateInterval, number> = {
  NEVER: 0,
  STARTUP: 0,
  FIFTEEN_MINUTES: 15 * 60,
  THIRTY_MINUTES: 30 * 60,
  ONE_HOUR: 60 * 60,
  SIX_HOURS: 6 * 60 * 60,
  ONE_DAY: 24 * 60 * 60,
  ONE_WEEK: 7 * 24 * 60 * 60,
};

export const updateCheckDue = (
  interval: UpdateInterval,
  checkedAt: number,
  now: number,
  startup = false,
): boolean => {
  if (interval === "NEVER") return false;
  if (interval === "STARTUP") return startup;
  return checkedAt <= 0 || now < checkedAt || now - checkedAt >= intervals[interval];
};
