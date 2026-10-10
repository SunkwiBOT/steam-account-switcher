import { UpdateInterval } from "../state/state.constants";

const intervals: Record<UpdateInterval, number> = {
  [UpdateInterval.NEVER]: 0,
  [UpdateInterval.STARTUP]: 0,
  [UpdateInterval.FIFTEEN_MINUTES]: 15 * 60,
  [UpdateInterval.THIRTY_MINUTES]: 30 * 60,
  [UpdateInterval.ONE_HOUR]: 60 * 60,
  [UpdateInterval.SIX_HOURS]: 6 * 60 * 60,
  [UpdateInterval.ONE_DAY]: 24 * 60 * 60,
  [UpdateInterval.ONE_WEEK]: 7 * 24 * 60 * 60,
};

export const updateCheckDue = (
  interval: UpdateInterval,
  checkedAt: number,
  now: number,
  startup = false,
): boolean => {
  if (interval === UpdateInterval.NEVER) return false;
  if (interval === UpdateInterval.STARTUP) return startup;
  return checkedAt <= 0 || now < checkedAt || now - checkedAt >= intervals[interval];
};
