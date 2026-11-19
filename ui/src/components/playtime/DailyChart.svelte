<script lang="ts">
  import { t, formatDate } from "../../lib/i18n/i18n.svelte";

  import { formatSeconds } from "../../lib/api/api";
  import { drawChart, type ChartBarType } from "../../lib/playtime/chart";
  import type { DailyPointDto } from "../../lib/bindings/DailyPointDto";

  type DailyChartPropsType = { points: DailyPointDto[] };

  let { points }: DailyChartPropsType = $props();

  let canvas = $state<HTMLCanvasElement | null>(null);
  let bars = $state<ChartBarType[]>([]);
  let width = $state(0);
  let tooltip = $state({ text: "", left: 0, top: 0 });

  $effect(() => {
    const element = canvas;
    if (!element) {
      return;
    }
    const draw = (): void => {
      const drawn = drawChart(element, points);
      bars = drawn.bars;
      width = drawn.width;
    };
    draw();
    const observer = new ResizeObserver(draw);
    observer.observe(element);
    return () => observer.disconnect();
  });

  const hover = (event: MouseEvent): void => {
    if (!canvas) {
      return;
    }
    const bounds = canvas.getBoundingClientRect();
    const x = event.clientX - bounds.left;
    const hovered = bars.find((bar) => x >= bar.x - 2 && x <= bar.x + bar.width + 2);
    if (!hovered) {
      tooltip = { ...tooltip, text: "" };
      return;
    }
    tooltip = {
      text: t.report_chart_tooltip({
        date: formatDate(hovered.point.day, false),
        time: formatSeconds(hovered.point.seconds),
        sessions: t.report_sessions_count({ count: hovered.point.sessions }),
      }),
      left: Math.min(x + 12, Math.max(0, width - 190)),
      top: Math.max(6, hovered.y - 34),
    };
  };
</script>

<div class="relative">
  <canvas
    class="block h-[220px] w-full rounded-md border border-[var(--border)] bg-[var(--base)]"
    bind:this={canvas}
    onmousemove={hover}
    onmouseleave={() => (tooltip = { ...tooltip, text: "" })}
  ></canvas>
  {#if tooltip.text}
    <div
      class="pointer-events-none absolute z-[5] rounded-md border border-[var(--border-strong)] bg-[var(--panel)] px-2 py-1 text-[12px] text-[var(--text-strong)]"
      style="left:{tooltip.left}px;top:{tooltip.top}px"
    >
      {tooltip.text}
    </div>
  {/if}
</div>
