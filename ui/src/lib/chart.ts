/* Daily playtime bar chart, drawn on a canvas. */

import type { DailyPoint } from "./types";

const PADDING = { top: 14, right: 14, bottom: 26, left: 54 };
const HEIGHT = 180;

export interface ChartBar {
  x: number;
  y: number;
  width: number;
  point: DailyPoint;
}

/** Compact axis label: minutes below an hour, then hours with a decimal. */
export function axisLabel(seconds: number): string {
  if (seconds <= 0) {
    return "0";
  }
  if (seconds < 3600) {
    return `${Math.round(seconds / 60)}m`;
  }
  const hours = seconds / 3600;
  return Number.isInteger(hours) ? `${hours}h` : `${hours.toFixed(1)}h`;
}

/** Draws the chart and returns the bars, so the caller can show a tooltip. */
export function drawChart(
  canvas: HTMLCanvasElement,
  points: DailyPoint[],
): { bars: ChartBar[]; width: number } {
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
        point.day.slice(5),
        PADDING.left + index * slot + slot / 2,
        PADDING.top + plotHeight + 6,
      );
    }
  });

  return { bars, width };
}
