<script lang="ts">
  import { onMount } from 'svelte';
  import type { TaskWithTags, Status } from '$lib/types';
  import { groups, statuses, sortedTasks, todayTasks, overdueTasks } from '$lib/stores';
  import { addDays, daysBetween, dayOfMonth, formatDate, formatMonth, overdueLabel, startOfDay, weekday } from '$lib/dateUtils';
  import TaskItem from './TaskItem.svelte';
  import Sun           from 'lucide-svelte/icons/sun';
  import TriangleAlert  from 'lucide-svelte/icons/triangle-alert';
  import ChevronLeft   from 'lucide-svelte/icons/chevron-left';
  import ChevronRight  from 'lucide-svelte/icons/chevron-right';
  import LocateFixed   from 'lucide-svelte/icons/locate-fixed';

  type Props = {
    onStatusChange: (id: number, status: Status) => void;
    onDelete:       (id: number) => void;
    onEdit:         (task: TaskWithTags) => void;
  };

  let { onStatusChange, onDelete, onEdit }: Props = $props();

  // ── タイムライン定義: 今日を基準に前後1年 ──────────────────────────────
  const DAY_W      = 34;   // 1日あたりの幅(px)
  const LEFT_W     = 300;  // 左のタスク列の幅(px)
  const PAST_DAYS  = 365;
  const FUTURE_DAYS = 365;

  const todayStart = startOfDay(Date.now());
  const rangeStart = addDays(todayStart, -PAST_DAYS);
  const totalDays  = PAST_DAYS + FUTURE_DAYS + 1;
  const timelineW  = totalDays * DAY_W;

  const days = Array.from({ length: totalDays }, (_, i) => addDays(rangeStart, i));

  const WEEKDAY_LABELS = ['日', '月', '火', '水', '木', '金', '土'];

  // 月ヘッダー: {start_index, day_count, label}
  const months = (() => {
    const out: { idx: number; count: number; label: string }[] = [];
    for (let i = 0; i < totalDays; i++) {
      if (i === 0 || dayOfMonth(days[i]) === 1) {
        out.push({ idx: i, count: 1, label: formatMonth(days[i]) });
      } else {
        out[out.length - 1].count++;
      }
    }
    return out;
  })();

  // 週末の縦帯（背景用）
  const weekends = days
    .map((d, i) => ({ i, wd: weekday(d) }))
    .filter(x => x.wd === 0 || x.wd === 6);

  function xOfDay(ts: number): number {
    return daysBetween(rangeStart, ts) * DAY_W;
  }

  // ── 行データ: 今日のタスク + 期限切れタスク + 全タスク（フィルター/ソート適用済み） ──
  type RowKind = 'today' | 'overdue' | 'all';
  const rows = $derived([
    ...$todayTasks.map(t => ({ task: t, kind: 'today' as RowKind })),
    ...$overdueTasks.map(t => ({ task: t, kind: 'overdue' as RowKind })),
    ...$sortedTasks.map(t => ({ task: t, kind: 'all' as RowKind })),
  ]);
  const rowKeyPrefix: Record<RowKind, string> = { today: 't', overdue: 'o', all: '' };

  // ── バー / マイルストン ────────────────────────────────────────────────
  const DEFAULT_STATUS_COLORS: Record<string, string> = {
    todo:    '#94A3B8',
    doing:   '#F59E0B',
    pending: '#8B5CF6',
    done:    '#22C55E',
  };

  function isOverdue(t: TaskWithTags): boolean {
    return t.status !== 'done' && t.due_at !== null && t.due_at < todayStart;
  }

  function barColor(t: TaskWithTags): string {
    if (isOverdue(t)) return '#EF4444';
    const def = $statuses.find(s => s.key === t.status);
    return def?.color ?? DEFAULT_STATUS_COLORS[t.status] ?? '#64748B';
  }

  function statusName(t: TaskWithTags): string {
    return $statuses.find(s => s.key === t.status)?.name ?? t.status;
  }

  // 期間バー: start_at〜due_at（両端の日を含む）
  function barRect(t: TaskWithTags): { left: number; width: number } | null {
    if (t.start_at === null || t.due_at === null) return null;
    const left  = xOfDay(t.start_at);
    const width = (daysBetween(t.start_at, t.due_at) + 1) * DAY_W;
    return { left: left + 2, width: Math.max(width - 4, DAY_W - 4) };
  }

  // マイルストン: due_at のみ（その日のセル中央）
  function milestoneX(t: TaskWithTags): number | null {
    if (t.due_at === null || t.start_at !== null) return null;
    return xOfDay(t.due_at) + DAY_W / 2;
  }

  // ── スクロール ─────────────────────────────────────────────────────────
  let scrollEl = $state<HTMLDivElement | null>(null);

  function scrollToToday(smooth = false) {
    // 今日が左寄り（1週間前が左端）に来る位置
    const left = (PAST_DAYS - 7) * DAY_W;
    scrollEl?.scrollTo({ left, behavior: smooth ? 'smooth' : 'auto' });
  }

  function scrollMonth(dir: -1 | 1) {
    scrollEl?.scrollBy({ left: dir * 30 * DAY_W, behavior: 'smooth' });
  }

  onMount(() => scrollToToday());

  // ── ツールチップ ───────────────────────────────────────────────────────
  let tooltip = $state<{ task: TaskWithTags; x: number; y: number } | null>(null);

  function showTooltip(e: MouseEvent, task: TaskWithTags) {
    tooltip = { task, x: e.clientX, y: e.clientY };
  }
  function moveTooltip(e: MouseEvent) {
    if (tooltip) tooltip = { ...tooltip, x: e.clientX, y: e.clientY };
  }
  function hideTooltip() {
    tooltip = null;
  }

  function periodLabel(t: TaskWithTags): string {
    if (t.start_at !== null && t.due_at !== null) {
      return `${formatDate(t.start_at)} 〜 ${formatDate(t.due_at)}`;
    }
    return t.due_at !== null ? `期限: ${formatDate(t.due_at)}` : '';
  }
</script>

<!-- ── コントロールバー ─────────────────────────────────────────────── -->
<div class="sched-controls">
  <button class="ctrl-btn" onclick={() => scrollMonth(-1)} title="前月へ">
    <ChevronLeft size={13} strokeWidth={2.5} />前月
  </button>
  <button class="ctrl-btn today-btn" onclick={() => scrollToToday(true)} title="今日へ戻る">
    <LocateFixed size={13} strokeWidth={2} />今日
  </button>
  <button class="ctrl-btn" onclick={() => scrollMonth(1)} title="翌月へ">
    翌月<ChevronRight size={13} strokeWidth={2.5} />
  </button>
</div>

<!-- ── ガントチャート本体 ───────────────────────────────────────────── -->
<div class="sched-scroll" bind:this={scrollEl}>
  <div class="sched-inner" style="width:{LEFT_W + timelineW}px">

    <!-- ヘッダー（月 + 日） -->
    <div class="sched-header">
      <div class="header-corner" style="width:{LEFT_W}px">タスク</div>
      <div class="header-timeline" style="width:{timelineW}px">
        <div class="month-row">
          {#each months as m (m.idx)}
            <div class="month-cell" style="left:{m.idx * DAY_W}px; width:{m.count * DAY_W}px">
              {m.label}
            </div>
          {/each}
        </div>
        <div class="day-row">
          {#each days as d, i (d)}
            <div
              class="day-cell"
              class:weekend={weekday(d) === 0 || weekday(d) === 6}
              class:sun={weekday(d) === 0}
              class:sat={weekday(d) === 6}
              class:today={d === todayStart}
              style="left:{i * DAY_W}px; width:{DAY_W}px"
            >
              <span class="day-wd">{WEEKDAY_LABELS[weekday(d)]}</span>
              <span class="day-num">{dayOfMonth(d)}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <!-- 行エリア -->
    <div class="sched-body">
      <!-- 背景レイヤー（週末帯 + 今日ライン + 日グリッド） -->
      <div class="grid-layer" style="left:{LEFT_W}px; width:{timelineW}px; --day-w:{DAY_W}px">
        {#each weekends as w (w.i)}
          <div class="weekend-stripe" style="left:{w.i * DAY_W}px; width:{DAY_W}px"></div>
        {/each}
        <div class="today-line" style="left:{xOfDay(todayStart) + DAY_W / 2}px"></div>
      </div>

      {#if rows.length === 0}
        <div class="sched-empty" style="width:{LEFT_W}px">タスクがありません</div>
      {/if}

      {#each rows as row, i (rowKeyPrefix[row.kind] + row.task.id)}
        {@const t = row.task}
        {@const g = $groups.find(gr => gr.id === t.group_id)}
        {@const bar = barRect(t)}
        {@const mx = milestoneX(t)}

        {#if i === 0 || rows[i - 1].kind !== row.kind}
          {#if row.kind === 'today'}
            <div class="today-section-header" style="width:{LEFT_W}px">
              <span class="today-sun"><Sun size={13} strokeWidth={2} /></span>
              今日のタスク
            </div>
          {:else if row.kind === 'overdue'}
            <div class="overdue-section-header" style="width:{LEFT_W}px">
              <span class="overdue-warn"><TriangleAlert size={13} strokeWidth={2} /></span>
              期限切れ
            </div>
          {:else}
            <div class="all-section-header" style="width:{LEFT_W}px">すべてのタスク</div>
          {/if}
        {/if}

        <div class="sched-row">
          <div class="row-left" style="width:{LEFT_W}px">
            <TaskItem
              task={t}
              {onStatusChange}
              {onDelete}
              {onEdit}
              groupLabel={g?.name}
              groupColor={g?.color ?? null}
              extraBadge={row.kind === 'overdue' && t.due_at ? overdueLabel(t.due_at) : undefined}
            />
          </div>
          <div class="row-timeline" style="width:{timelineW}px">
            {#if bar}
              <button
                class="gantt-bar"
                class:overdue={isOverdue(t)}
                class:done={t.status === 'done'}
                style="left:{bar.left}px; width:{bar.width}px; --bar-c:{barColor(t)}"
                onclick={() => onEdit(t)}
                onmouseenter={(e) => showTooltip(e, t)}
                onmousemove={moveTooltip}
                onmouseleave={hideTooltip}
                aria-label="{t.title} の期間バー"
              >
                <span class="bar-label">{t.title}</span>
              </button>
            {:else if mx !== null}
              <button
                class="milestone"
                class:overdue={isOverdue(t)}
                style="left:{mx}px; --bar-c:{barColor(t)}"
                onclick={() => onEdit(t)}
                onmouseenter={(e) => showTooltip(e, t)}
                onmousemove={moveTooltip}
                onmouseleave={hideTooltip}
                aria-label="{t.title} のマイルストン"
              >◆</button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  </div>
</div>

<!-- ── ツールチップ ─────────────────────────────────────────────────── -->
{#if tooltip}
  <div class="gantt-tooltip" style="left:{tooltip.x + 14}px; top:{tooltip.y + 14}px">
    <p class="tt-title">{tooltip.task.title}</p>
    {#if periodLabel(tooltip.task)}
      <p class="tt-line">{periodLabel(tooltip.task)}</p>
    {/if}
    <p class="tt-line">ステータス: {statusName(tooltip.task)}</p>
    {#if isOverdue(tooltip.task)}
      <p class="tt-line tt-overdue">期限超過</p>
    {/if}
  </div>
{/if}

<style>
/* ── コントロール ── */
.sched-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 5px 14px;
  border-bottom: 1px solid var(--border-light);
  background: var(--bg);
  flex-shrink: 0;
}
.ctrl-btn {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 3px 9px;
  border-radius: 6px;
  border: 1px solid var(--border);
  font-size: 0.72rem;
  color: var(--text-muted);
  background: var(--surface);
  cursor: pointer;
  transition: background 0.1s, color 0.1s;
}
.ctrl-btn:hover { background: var(--hover); color: var(--text); }
.today-btn { color: var(--accent); border-color: var(--accent); }
.today-btn:hover { background: var(--accent-soft); color: var(--accent); }

/* ── スクロール領域 ── */
.sched-scroll {
  flex: 1;
  overflow: auto;
  position: relative;
}
.sched-inner { position: relative; }

/* ── ヘッダー ── */
.sched-header {
  display: flex;
  position: sticky;
  top: 0;
  z-index: 30;
  background: var(--bg);
  border-bottom: 1px solid var(--border);
}
.header-corner {
  position: sticky;
  left: 0;
  z-index: 31;
  flex-shrink: 0;
  display: flex;
  align-items: flex-end;
  padding: 6px 12px;
  font-size: 0.72rem;
  font-weight: 600;
  color: var(--text-muted);
  background: var(--bg);
  border-right: 1px solid var(--border);
}
.header-timeline { position: relative; flex-shrink: 0; }
.month-row {
  position: relative;
  height: 22px;
  border-bottom: 1px solid var(--border-light);
}
.month-cell {
  position: absolute;
  top: 0;
  height: 100%;
  display: flex;
  align-items: center;
  padding-left: 8px;
  font-size: 0.72rem;
  font-weight: 600;
  color: var(--text-muted);
  border-left: 1px solid var(--border);
  white-space: nowrap;
  overflow: hidden;
}
.day-row { position: relative; height: 32px; }
.day-cell {
  position: absolute;
  top: 0;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 1px;
  color: var(--text-muted);
  border-left: 1px solid var(--border-light);
  font-variant-numeric: tabular-nums;
}
.day-wd  { font-size: 0.56rem; line-height: 1; opacity: 0.85; }
.day-num { font-size: 0.68rem; line-height: 1; }
.day-cell.weekend { background: var(--hover); }
.day-cell.sun .day-wd { color: var(--danger); }
.day-cell.sat .day-wd { color: #3B82F6; }
.day-cell.today {
  background: var(--accent);
  color: #fff;
  font-weight: 700;
  border-radius: 4px;
}
.day-cell.today .day-wd { color: #fff; opacity: 0.9; }

/* ── 行エリア ── */
.sched-body { position: relative; }
.grid-layer {
  position: absolute;
  top: 0;
  bottom: 0;
  z-index: 0;
  pointer-events: none;
  background-image: repeating-linear-gradient(
    to right,
    var(--border-light) 0 1px,
    transparent 1px var(--day-w)
  );
}
.weekend-stripe {
  position: absolute;
  top: 0;
  bottom: 0;
  background: var(--hover);
  opacity: 0.55;
}
.today-line {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  margin-left: -1px;
  background: var(--accent);
  opacity: 0.85;
  z-index: 1;
}

.sched-row {
  display: flex;
  position: relative;
}
.row-left {
  position: sticky;
  left: 0;
  z-index: 20;
  flex-shrink: 0;
  background: var(--surface);
  border-right: 1px solid var(--border);
}
/* ステータスピッカー展開中は、後続行の row-left (同じ z-index の
   別スタッキングコンテキスト) に覆われないよう自身を最前面に引き上げる */
.row-left:has(:global(.status-picker)) {
  z-index: 50;
}
.row-timeline {
  position: relative;
  flex-shrink: 0;
  border-bottom: 1px solid var(--border-light);
}

/* ── セクション見出し ── */
.today-section-header,
.overdue-section-header,
.all-section-header {
  position: sticky;
  left: 0;
  z-index: 20;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px 4px;
  font-size: 0.74rem;
  font-weight: 700;
  color: var(--text);
  background: var(--bg);
  border-right: 1px solid var(--border);
  border-bottom: 1px solid var(--border-light);
}
.all-section-header { color: var(--text-muted); }
.overdue-section-header { color: var(--danger); }
.today-sun { color: #F59E0B; display: flex; align-items: center; }
.overdue-warn { color: var(--danger); display: flex; align-items: center; }

.sched-empty {
  position: sticky;
  left: 0;
  padding: 24px 12px;
  color: var(--text-muted);
  font-size: 0.82rem;
}

/* ── ガントバー ── */
.gantt-bar {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  height: 18px;
  border-radius: 5px;
  background: var(--bar-c);
  opacity: 0.9;
  border: none;
  cursor: pointer;
  z-index: 2;
  display: flex;
  align-items: center;
  padding: 0 6px;
  overflow: hidden;
  transition: opacity 0.1s, transform 0.1s;
}
.gantt-bar:hover { opacity: 1; transform: translateY(-50%) scaleY(1.12); }
.gantt-bar.done { opacity: 0.45; }
.gantt-bar.overdue {
  background: repeating-linear-gradient(
    -45deg,
    var(--bar-c) 0 8px,
    color-mix(in srgb, var(--bar-c) 70%, #000) 8px 16px
  );
}
.bar-label {
  font-size: 0.68rem;
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  text-shadow: 0 1px 2px rgba(0,0,0,0.35);
}

/* ── マイルストン ── */
.milestone {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%);
  background: transparent;
  border: none;
  color: var(--bar-c);
  font-size: 1rem;
  line-height: 1;
  cursor: pointer;
  z-index: 2;
  padding: 2px;
  transition: transform 0.1s;
}
.milestone:hover { transform: translate(-50%, -50%) scale(1.3); }
.milestone.overdue { color: #EF4444; }

/* ── ツールチップ ── */
.gantt-tooltip {
  position: fixed;
  z-index: 1000;
  pointer-events: none;
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 8px;
  box-shadow: 0 6px 20px rgba(0,0,0,0.18);
  padding: 8px 12px;
  max-width: 280px;
}
.tt-title {
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--text);
  margin-bottom: 3px;
}
.tt-line {
  font-size: 0.72rem;
  color: var(--text-muted);
}
.tt-overdue { color: var(--danger); font-weight: 600; }
</style>
