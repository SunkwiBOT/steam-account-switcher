import { t, locale } from "../i18n/i18n.svelte";
import { formatSeconds } from "../api/api";

import type { DailyPointType } from "../types";

const PADDING = { top: 14, right: 14, bottom: 26, left: 54 };
const HEIGHT = 180;

export type ChartBarType = {
  x: number;
  y: number;
  width: number;
  point: DailyPointType;
};

export const axisLabel = (seconds: number): string => {
  if (seconds <= 0) {
    return "0";
  }

  if (seconds < 3600) {
    return formatSeconds(seconds);
  }

  const hours = seconds / 3600;

  return t.duration_hours({
    hours: new Intl.NumberFormat(locale.current, { maximumFractionDigits: 1 }).format(hours),
  });
};

export const drawChart = (
  canvas: HTMLCanvasElement,
  points: DailyPointType[],
): { bars: ChartBarType[]; width: number } => {
  const ratio = window.devicePixelRatio || 1;
  const width = canvas.clientWidth || 900;
  canvas.width = width * ratio;
  canvas.height = HEIGHT * ratio;

  const context = canvas.getContext("2d");
  if (!context) {
    return { bars: [], width };
  }
  context.scale(ratio, ratio);

  const plotWidth = width - PADDING.left - PADDING.right;
  const plotHeight = HEIGHT - PADDING.top - PADDING.bottom;
  const max = Math.max(1, ...points.map((point) => point.seconds));
  const slot = plotWidth / Math.max(1, points.length);
  const barWidth = Math.max(2, Math.min(26, slot - 3));

  context.fillStyle = "#0e141b";
  context.fillRect(0, 0, width, HEIGHT);
  context.strokeStyle = "#223348";
  context.fillStyle = "#8f98a0";
  context.font = "11px 'Noto Sans', 'Segoe UI', sans-serif";
  context.textAlign = "right";
  context.textBaseline = "middle";

  const steps = 4;
  for (let step = 0; step <= steps; step += 1) {
    const value = (max / steps) * step;
    const y = PADDING.top + plotHeight - (plotHeight / steps) * step;
    context.beginPath();
    context.moveTo(PADDING.left, y);
    context.lineTo(width - PADDING.right, y);
    context.stroke();
    context.fillText(axisLabel(value), PADDING.left - 8, y);
  }

  const bars = points.map((point, index) => {
    const x = PADDING.left + index * slot + (slot - barWidth) / 2;
    const barHeight = (point.seconds / max) * plotHeight;
    const y = PADDING.top + plotHeight - barHeight;
    context.fillStyle = point.seconds > 0 ? "#66c0f4" : "#1b2838";
    context.fillRect(x, y, barWidth, Math.max(1, barHeight));
    return { x, y, width: barWidth, point };
  });

  const labelEvery = Math.max(1, Math.ceil(points.length / 8));
  context.textAlign = "center";
  context.textBaseline = "top";
  points.forEach((point, index) => {
    if (index % labelEvery === 0) {
      context.fillText(
        new Intl.DateTimeFormat(locale.current, { month: "2-digit", day: "2-digit" }).format(
          new Date(`${point.day}T12:00:00`),
        ),
        PADDING.left + index * slot + slot / 2,
        PADDING.top + plotHeight + 6,
      );
    }
  });

  return { bars, width };
};
