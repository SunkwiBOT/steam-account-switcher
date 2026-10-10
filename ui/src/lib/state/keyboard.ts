// Index of the item to focus after a navigation key, or null when the key does not navigate.
// `current` is -1 when nothing in the list has focus yet.
export const nextMenuIndex = (current: number, count: number, key: string): number | null => {
  if (count === 0) return null;
  switch (key) {
    case "ArrowDown":
      return current < 0 || current >= count - 1 ? 0 : current + 1;
    case "ArrowUp":
      return current <= 0 ? count - 1 : current - 1;
    case "Home":
      return 0;
    case "End":
      return count - 1;
    default:
      return null;
  }
};

// Tab stays inside a dialog: wraps forward from the last item and backward from the first.
export const cycleFocusIndex = (current: number, count: number, backwards: boolean): number => {
  if (count === 0) return -1;
  if (current < 0 || current >= count) return backwards ? count - 1 : 0;
  if (backwards) return current === 0 ? count - 1 : current - 1;
  return current === count - 1 ? 0 : current + 1;
};
