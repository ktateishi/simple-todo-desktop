<script lang="ts">
  import { onMount }                                        from 'svelte';
  import { isPermissionGranted, requestPermission }         from '@tauri-apps/plugin-notification';
  import { api } from '$lib/api';
  import {
    tasks, groups, tags, statuses, tasksByGroup, sortedTasks, todayTasks, overdueTasks,
    filterStatuses, filterGroupId, filterTagId, searchQuery, sortMode, viewMode,
    clipboardTask,
    loadAll, refreshTasks, refreshTags, refreshStatuses,
  } from '$lib/stores';
  import { overdueLabel } from '$lib/dateUtils';
  import type { TaskWithTags, Status, StatusDef, TaskUpdate } from '$lib/types';
  import QuickAdd      from '$lib/components/QuickAdd.svelte';
  import GroupSection  from '$lib/components/GroupSection.svelte';
  import TaskItem      from '$lib/components/TaskItem.svelte';
  import EditModal     from '$lib/components/EditModal.svelte';
  import ScheduleView  from '$lib/components/ScheduleView.svelte';
  import ThemeSwitcher from '$lib/components/ThemeSwitcher.svelte';

  import Search       from 'lucide-svelte/icons/search';
  import Pencil2      from 'lucide-svelte/icons/pencil';
  import List         from 'lucide-svelte/icons/list';
  import Circle       from 'lucide-svelte/icons/circle';
  import Timer        from 'lucide-svelte/icons/timer';
  import CircleCheck  from 'lucide-svelte/icons/circle-check';
  import CirclePause  from 'lucide-svelte/icons/circle-pause';
  import FolderOpen   from 'lucide-svelte/icons/folder-open';
  import Hash         from 'lucide-svelte/icons/hash';
  import Plus         from 'lucide-svelte/icons/plus';
  import X            from 'lucide-svelte/icons/x';
  import CheckSquare  from 'lucide-svelte/icons/square-check-big';
  import Sun          from 'lucide-svelte/icons/sun';
  import TriangleAlert from 'lucide-svelte/icons/triangle-alert';
  import ArrowUp      from 'lucide-svelte/icons/arrow-up';
  import ArrowDown    from 'lucide-svelte/icons/arrow-down';
  import ArrowUpDown  from 'lucide-svelte/icons/arrow-up-down';
  import Layers        from 'lucide-svelte/icons/layers';
  import LayoutList    from 'lucide-svelte/icons/layout-list';
  import ChartGantt    from 'lucide-svelte/icons/chart-gantt';
  import Clipboard     from 'lucide-svelte/icons/clipboard';
  import ClipboardX   from 'lucide-svelte/icons/clipboard-x';
  import { dndzone }   from 'svelte-dnd-action';
  import { check as checkUpdate, type Update } from '@tauri-apps/plugin-updater';
  import { relaunch } from '@tauri-apps/plugin-process';

  const PRESET_COLORS = [
    '#6366F1','#3B82F6','#06B6D4','#22C55E',
    '#EAB308','#F97316','#EF4444','#EC4899','#8B5CF6',
  ];

  let pendingUpdate   = $state<Update | null>(null);
  let updating        = $state(false);

  let editingTask     = $state<TaskWithTags | null>(null);
  let newGroupName    = $state('');
  let newGroupColor   = $state<string | null>(null);
  let newTagName      = $state('');
  let newTagColor     = $state<string | null>(null);
  let showGroupForm   = $state(false);
  let showTagForm     = $state(false);
  let editingGroupId  = $state<number | null>(null);
  let editGroupName   = $state('');
  let editGroupColor  = $state<string | null>(null);

  // ── カスタムステータス管理 ──
  let showStatusForm      = $state(false);
  let newStatusName       = $state('');
  let newStatusColor      = $state<string | null>(null);
  let newStatusShowToday  = $state(true);
  let editingStatusId     = $state<number | null>(null);
  let editStatusName      = $state('');
  let editStatusColor     = $state<string | null>(null);
  let editStatusShowToday = $state(true);

  // ── 今日のタスク DnD ───────────────────────────────────────────────────
  const TODAY_ORDER_KEY = 'today_task_order';
  const TODAY_DND_MS    = 150;
  let todayLocalItems   = $state<TaskWithTags[]>([]);

  // todayTasks が変わるたびに localItems を再構築（カスタム順を維持）
  $effect(() => {
    const current = $todayTasks;
    const saved: number[] = JSON.parse(localStorage.getItem(TODAY_ORDER_KEY) ?? '[]');
    todayLocalItems = [
      ...saved.map(id => current.find(t => t.id === id)).filter((t): t is TaskWithTags => !!t),
      ...current.filter(t => !saved.includes(t.id)),
    ];
  });

  function handleTodayConsider(e: CustomEvent<{ items: TaskWithTags[] }>) {
    todayLocalItems = e.detail.items;
  }

  async function handleTodayFinalize(e: CustomEvent<{ items: TaskWithTags[] }>) {
    todayLocalItems = e.detail.items;
    localStorage.setItem(TODAY_ORDER_KEY, JSON.stringify(todayLocalItems.map(t => t.id)));

    // 今日以外の期限を持つタスク（他セクションからドラッグされた）を今日の日付に更新
    const now        = new Date();
    const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const todayEnd   = todayStart + 86_400_000;

    const incoming = todayLocalItems.filter(
      t => t.due_at === null || t.due_at < todayStart || t.due_at >= todayEnd
    );

    if (incoming.length > 0) {
      await Promise.all(
        incoming.map(t =>
          api.updateTask(t.id, {
            title:            t.title,
            notes:            t.notes            ?? undefined,
            due_at:           todayStart,
            remind_at:        t.remind_at        ?? undefined,
            group_id:         t.group_id         ?? undefined,
            priority:         t.priority,
            recur_type:       t.recur_type       ?? undefined,
            recur_interval:   t.recur_interval   ?? undefined,
            recur_weekdays:   t.recur_weekdays   ?? undefined,
            recur_month_rule: t.recur_month_rule ?? undefined,
            recur_month_day:  t.recur_month_day  ?? undefined,
          })
        )
      );
      await refreshTasks();
    }
  }

  // ── 期限切れタスク DnD（並び替えのみ、他ゾーンからのドロップは不可）────────
  const OVERDUE_ORDER_KEY = 'overdue_task_order';
  const OVERDUE_DND_MS    = 150;
  let overdueLocalItems   = $state<TaskWithTags[]>([]);

  $effect(() => {
    const current = $overdueTasks;
    const saved: number[] = JSON.parse(localStorage.getItem(OVERDUE_ORDER_KEY) ?? '[]');
    overdueLocalItems = [
      ...saved.map(id => current.find(t => t.id === id)).filter((t): t is TaskWithTags => !!t),
      ...current.filter(t => !saved.includes(t.id)),
    ];
  });

  function handleOverdueConsider(e: CustomEvent<{ items: TaskWithTags[] }>) {
    overdueLocalItems = e.detail.items;
  }

  function handleOverdueFinalize(e: CustomEvent<{ items: TaskWithTags[] }>) {
    overdueLocalItems = e.detail.items;
    localStorage.setItem(OVERDUE_ORDER_KEY, JSON.stringify(overdueLocalItems.map(t => t.id)));
  }

  onMount(async () => {
    if (!(await isPermissionGranted())) {
      await requestPermission();
    }
    await loadAll();

    // Check for updates in background (non-blocking)
    checkUpdate().then(u => { if (u?.available) pendingUpdate = u; }).catch(() => {});
  });

  async function applyUpdate() {
    if (!pendingUpdate) return;
    updating = true;
    await pendingUpdate.downloadAndInstall();
    await relaunch();
  }

  // ── Quick Add ──────────────────────────────────────────────────────────
  async function handleAdd(data: { title: string; tagNames: string[]; groupId?: number }) {
    const { title, tagNames, groupId } = data;
    await api.addTask({
      title,
      group_id: groupId,
      new_tag_names: tagNames.length > 0 ? tagNames : undefined,
    });
    await Promise.all([refreshTasks(), refreshTags()]);
  }

  // ── Task events ────────────────────────────────────────────────────────
  async function handleStatusChange(id: number, status: Status) {
    await api.setTaskStatus(id, status);
    await refreshTasks();
  }

  async function handleDelete(id: number) {
    await api.deleteTask(id);
    await refreshTasks();
  }

  function handleEdit(task: TaskWithTags) {
    editingTask = task;
  }

  async function handleSave(p: { id: number; update: TaskUpdate; tagIds: number[] }) {
    await api.updateTask(p.id, p.update);
    await api.setTaskTags(p.id, p.tagIds);
    await refreshTasks();
    editingTask = null;
  }

  async function handleReorder(p: { movedId: number; beforeId?: number; afterId?: number }) {
    await api.reorderTask(p.movedId, p.beforeId, p.afterId);
    await refreshTasks();
  }

  function autoFocus(node: HTMLElement) { node.focus(); }

  // ── Groups / Tags ──────────────────────────────────────────────────────
  async function addGroup() {
    if (!newGroupName.trim()) return;
    await api.addGroup(newGroupName.trim(), newGroupColor ?? undefined);
    groups.set(await api.listGroups());
    newGroupName = ''; newGroupColor = null;
    showGroupForm = false;
  }

  async function addTag() {
    if (!newTagName.trim()) return;
    await api.addTag(newTagName.trim(), newTagColor ?? undefined);
    await refreshTags();
    newTagName = ''; newTagColor = null;
    showTagForm = false;
  }

  function handleGroupFormFocusout(e: FocusEvent) {
    if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) {
      if (newGroupName.trim()) addGroup();
      else { showGroupForm = false; newGroupColor = null; }
    }
  }

  function handleTagFormFocusout(e: FocusEvent) {
    if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) {
      if (newTagName.trim()) addTag();
      else { showTagForm = false; newTagColor = null; }
    }
  }

  function startEditGroup(g: { id: number; name: string; color: string | null }) {
    editingGroupId = g.id;
    editGroupName  = g.name;
    editGroupColor = g.color;
  }

  async function saveGroup() {
    if (!editGroupName.trim() || editingGroupId === null) return;
    await api.updateGroup(editingGroupId, editGroupName.trim(), editGroupColor ?? undefined);
    groups.set(await api.listGroups());
    editingGroupId = null;
  }

  async function deleteGroup(id: number) {
    await api.deleteGroup(id);
    groups.set(await api.listGroups());
    if ($filterGroupId === id) filterGroupId.set(null);
    await refreshTasks();
  }

  async function deleteTag(id: number) {
    await api.deleteTag(id);
    await refreshTags();
    if ($filterTagId === id) filterTagId.set(null);
  }

  const DEFAULT_STATUS_ICONS: Record<string, typeof Circle | undefined> = {
    todo:    Circle,
    doing:   Timer,
    pending: CirclePause,
    done:    CircleCheck,
  };

  // ── ステータス管理 ─────────────────────────────────────────────────────
  async function addStatus() {
    if (!newStatusName.trim()) return;
    await api.addStatus(newStatusName.trim(), newStatusColor, newStatusShowToday);
    await refreshStatuses();
    newStatusName = ''; newStatusColor = null; newStatusShowToday = true;
    showStatusForm = false;
  }

  function startEditStatus(s: StatusDef) {
    editingStatusId     = s.id;
    editStatusName      = s.name;
    editStatusColor     = s.color;
    editStatusShowToday = s.show_in_today;
  }

  async function saveStatus() {
    if (!editStatusName.trim() || editingStatusId === null) return;
    await api.updateStatus(editingStatusId, editStatusName.trim(), editStatusColor, editStatusShowToday);
    await refreshStatuses();
    editingStatusId = null;
  }

  async function deleteStatus(s: StatusDef) {
    await api.deleteStatus(s.id);
    filterStatuses.update(prev => {
      if (!prev.has(s.key)) return prev;
      const next = new Set(prev);
      next.delete(s.key);
      return next;
    });
    await Promise.all([refreshStatuses(), refreshTasks()]);
  }

  function handleStatusFormFocusout(e: FocusEvent) {
    if (!(e.currentTarget as HTMLElement).contains(e.relatedTarget as Node)) {
      if (newStatusName.trim()) addStatus();
      else { showStatusForm = false; newStatusColor = null; newStatusShowToday = true; }
    }
  }

  async function handlePaste() {
    const t = $clipboardTask;
    if (!t) return;
    await api.addTask({
      title:           t.title,
      notes:           t.notes           ?? undefined,
      due_at:          t.due_at          ?? undefined,
      remind_at:       t.remind_at       ?? undefined,
      group_id:        t.group_id        ?? undefined,
      tag_ids:         t.tags.map(tag => tag.id),
      recur_type:      t.recur_type      ?? undefined,
      recur_interval:  t.recur_interval  ?? undefined,
      recur_weekdays:  t.recur_weekdays  ?? undefined,
      recur_month_rule: t.recur_month_rule ?? undefined,
      recur_month_day:  t.recur_month_day  ?? undefined,
    });
    await Promise.all([refreshTasks(), refreshTags()]);
  }

  function toggleStatus(s: Status) {
    filterStatuses.update(prev => {
      const next = new Set(prev);
      next.has(s) ? next.delete(s) : next.add(s);
      return next;
    });
  }
</script>

<div class="layout">

  <!-- ── Sidebar ─────────────────────────────────────────────────────── -->
  <aside class="sidebar">
    <div class="sidebar-header">
      <CheckSquare size={16} strokeWidth={2} class="app-icon" />
      <span class="app-name">Simple TODO</span>
    </div>

    <div class="search-wrap">
      <div class="search-inner">
        <Search size={13} strokeWidth={2} class="search-icon" />
        <input
          class="search-input"
          placeholder="検索…"
          bind:value={$searchQuery}
          aria-label="タスク検索"
        />
      </div>
    </div>

    <!-- Status filter (multi-select) + custom status management -->
    <nav class="nav-section">
      <p class="nav-label">
        ビュー
        <button class="mini-btn" onclick={() => (showStatusForm = !showStatusForm)} title="ステータス追加">
          <Plus size={13} strokeWidth={2.5} />
        </button>
      </p>
      <button
        class="nav-item"
        class:active={$filterStatuses.size === 0}
        onclick={() => filterStatuses.set(new Set())}
      >
        <span class="nav-icon"><List size={14} strokeWidth={1.75} /></span>
        すべて
      </button>
      {#each $statuses as st (st.id)}
        {@const Icon = DEFAULT_STATUS_ICONS[st.key]}
        {#if editingStatusId === st.id}
          <form class="inline-form" onsubmit={(e) => { e.preventDefault(); saveStatus(); }}>
            <div class="color-swatches">
              {#each PRESET_COLORS as c}
                <button
                  type="button"
                  class="swatch"
                  class:sel={editStatusColor === c}
                  style="--c:{c}"
                  onclick={() => editStatusColor = editStatusColor === c ? null : c}
                  aria-label={c}
                ></button>
              {/each}
            </div>
            <label class="today-toggle">
              <input type="checkbox" bind:checked={editStatusShowToday} />
              今日のタスクに表示
            </label>
            <div class="inline-form-row">
              <input class="mini-input" bind:value={editStatusName} />
              <button type="submit" class="mini-btn-accent">保存</button>
              <button type="button" class="mini-btn" onclick={() => editingStatusId = null}>
                <X size={12} strokeWidth={2.5} />
              </button>
            </div>
          </form>
        {:else}
          <div class="nav-item-row">
            <button
              class="nav-item status-filter-btn flex1"
              class:active={$filterStatuses.has(st.key)}
              onclick={() => toggleStatus(st.key)}
            >
              <span class="nav-icon">
                {#if Icon}
                  <Icon size={14} strokeWidth={1.75} />
                {:else}
                  <span class="dot" style="background:{st.color ?? 'var(--accent)'}"></span>
                {/if}
              </span>
              <span class="nav-name" title={st.name}>{st.name}</span>
              {#if $filterStatuses.has(st.key)}
                <span class="filter-check">✓</span>
              {/if}
            </button>
            {#if st.is_custom}
              <button class="icon-btn" onclick={() => startEditStatus(st)} title="編集">
                <Pencil2 size={11} strokeWidth={2.5} />
              </button>
              <button class="icon-btn danger" onclick={() => deleteStatus(st)} title="削除">
                <X size={11} strokeWidth={2.5} />
              </button>
            {/if}
          </div>
        {/if}
      {/each}
      {#if showStatusForm}
        <form class="inline-form" onsubmit={(e) => { e.preventDefault(); addStatus(); }} onfocusout={handleStatusFormFocusout}>
          <div class="color-swatches">
            {#each PRESET_COLORS as c}
              <button
                type="button"
                class="swatch"
                class:sel={newStatusColor === c}
                style="--c:{c}"
                onclick={() => newStatusColor = newStatusColor === c ? null : c}
                aria-label={c}
              ></button>
            {/each}
          </div>
          <label class="today-toggle">
            <input type="checkbox" bind:checked={newStatusShowToday} />
            今日のタスクに表示
          </label>
          <input class="mini-input" bind:value={newStatusName} placeholder="ステータス名" use:autoFocus />
        </form>
      {/if}
    </nav>

    <!-- Groups -->
    <nav class="nav-section">
      <p class="nav-label">
        <span class="label-with-icon">
          <FolderOpen size={12} strokeWidth={2} />
          グループ
        </span>
        <button class="mini-btn" onclick={() => (showGroupForm = !showGroupForm)} title="グループ追加">
          <Plus size={13} strokeWidth={2.5} />
        </button>
      </p>
      <button
        class="nav-item"
        class:active={$filterGroupId === null}
        onclick={() => filterGroupId.set(null)}
      >
        <span class="nav-icon"><List size={14} strokeWidth={1.75} /></span>
        すべて
      </button>
      {#each $groups as g (g.id)}
        {#if editingGroupId === g.id}
          <form class="inline-form" onsubmit={(e) => { e.preventDefault(); saveGroup(); }}>
            <div class="color-swatches">
              {#each PRESET_COLORS as c}
                <button
                  type="button"
                  class="swatch"
                  class:sel={editGroupColor === c}
                  style="--c:{c}"
                  onclick={() => editGroupColor = editGroupColor === c ? null : c}
                  aria-label={c}
                ></button>
              {/each}
            </div>
            <div class="inline-form-row">
              <input class="mini-input" bind:value={editGroupName} />
              <button type="submit" class="mini-btn-accent">保存</button>
              <button type="button" class="mini-btn" onclick={() => editingGroupId = null}>
                <X size={12} strokeWidth={2.5} />
              </button>
            </div>
          </form>
        {:else}
          <div class="nav-item-row">
            <button
              class="nav-item flex1"
              class:active={$filterGroupId === g.id}
              onclick={() => filterGroupId.set($filterGroupId === g.id ? null : g.id)}
            >
              <span class="dot" style="background:{g.color ?? 'var(--accent)'}"></span>
              <span class="nav-name" title={g.name}>{g.name}</span>
            </button>
            <button class="icon-btn" onclick={() => startEditGroup(g)} title="編集">
              <Pencil2 size={11} strokeWidth={2.5} />
            </button>
            <button class="icon-btn danger" onclick={() => deleteGroup(g.id)} title="削除">
              <X size={11} strokeWidth={2.5} />
            </button>
          </div>
        {/if}
      {/each}
      {#if showGroupForm}
        <form class="inline-form" onsubmit={(e) => { e.preventDefault(); addGroup(); }} onfocusout={handleGroupFormFocusout}>
          <div class="color-swatches">
            {#each PRESET_COLORS as c}
              <button
                type="button"
                class="swatch"
                class:sel={newGroupColor === c}
                style="--c:{c}"
                onclick={() => newGroupColor = newGroupColor === c ? null : c}
                aria-label={c}
              ></button>
            {/each}
          </div>
          <input class="mini-input" bind:value={newGroupName} placeholder="グループ名" use:autoFocus />
        </form>
      {/if}
    </nav>

    <!-- Tags -->
    <nav class="nav-section">
      <p class="nav-label">
        <span class="label-with-icon">
          <Hash size={12} strokeWidth={2} />
          タグ
        </span>
        <button class="mini-btn" onclick={() => (showTagForm = !showTagForm)} title="タグ追加">
          <Plus size={13} strokeWidth={2.5} />
        </button>
      </p>
      {#each $tags as tag (tag.id)}
        <div class="nav-item-row">
          <button
            class="nav-item flex1"
            class:active={$filterTagId === tag.id}
            onclick={() => filterTagId.set($filterTagId === tag.id ? null : tag.id)}
          >
            <span class="tag-dot" style="background:{tag.color ?? 'var(--accent)'}"></span>
            <span class="nav-name" title="#{tag.name}">#{tag.name}</span>
          </button>
          <button class="icon-btn" onclick={() => deleteTag(tag.id)} title="削除">
            <X size={11} strokeWidth={2.5} />
          </button>
        </div>
      {/each}
      {#if showTagForm}
        <form class="inline-form" onsubmit={(e) => { e.preventDefault(); addTag(); }} onfocusout={handleTagFormFocusout}>
          <div class="color-swatches">
            {#each PRESET_COLORS as c}
              <button
                type="button"
                class="swatch"
                class:sel={newTagColor === c}
                style="--c:{c}"
                onclick={() => newTagColor = newTagColor === c ? null : c}
                aria-label={c}
              ></button>
            {/each}
          </div>
          <input class="mini-input" bind:value={newTagName} placeholder="タグ名" use:autoFocus />
        </form>
      {/if}
    </nav>

    <div class="sidebar-footer">
      <ThemeSwitcher />
    </div>
  </aside>

  <!-- ── Main ────────────────────────────────────────────────────────── -->
  <main class="main">

    <!-- Update banner -->
    {#if pendingUpdate}
      <div class="update-banner">
        <span class="update-msg">
          🎉 新しいバージョン <strong>v{pendingUpdate.version}</strong> が利用可能です
        </span>
        <div class="update-actions">
          <button class="update-btn" onclick={applyUpdate} disabled={updating}>
            {updating ? 'ダウンロード中…' : '今すぐ更新'}
          </button>
          <button class="update-dismiss" onclick={() => pendingUpdate = null} title="後で">
            <X size={13} strokeWidth={2.5} />
          </button>
        </div>
      </div>
    {/if}

    <QuickAdd onAdd={handleAdd} />

    <!-- Paste bar -->
    {#if $clipboardTask}
      <div class="paste-bar">
        <Clipboard size={13} strokeWidth={2} />
        <span class="paste-preview">
          コピー中: <strong>{$clipboardTask.title}</strong>
        </span>
        <button class="paste-btn" onclick={handlePaste}>貼り付け</button>
        <button class="paste-clear" onclick={() => clipboardTask.set(null)} title="クリア">
          <ClipboardX size={14} strokeWidth={2} />
        </button>
      </div>
    {/if}

    <!-- Sort + view toolbar -->
    <div class="sort-bar">
      <span class="sort-label">
        <ArrowUpDown size={12} strokeWidth={2} />
        並び替え
      </span>
      <div class="sort-btns">
        <button
          class="sort-btn"
          class:active={$sortMode === 'manual'}
          onclick={() => sortMode.set('manual')}
        >手動</button>
        <button
          class="sort-btn"
          class:active={$sortMode === 'due_asc'}
          onclick={() => sortMode.set('due_asc')}
        ><ArrowUp size={11} strokeWidth={2.5} />近い順</button>
        <button
          class="sort-btn"
          class:active={$sortMode === 'due_desc'}
          onclick={() => sortMode.set('due_desc')}
        ><ArrowDown size={11} strokeWidth={2.5} />遠い順</button>
        <button
          class="sort-btn"
          class:active={$sortMode === 'priority_desc'}
          onclick={() => sortMode.set('priority_desc')}
        >優先度順</button>
      </div>

      <div class="view-toggle">
        <button
          class="sort-btn"
          class:active={$viewMode === 'grouped'}
          onclick={() => viewMode.set('grouped')}
          title="グループ表示"
        ><Layers size={12} strokeWidth={2} />グループ</button>
        <button
          class="sort-btn"
          class:active={$viewMode === 'flat'}
          onclick={() => viewMode.set('flat')}
          title="一覧表示"
        ><LayoutList size={12} strokeWidth={2} />一覧</button>
        <button
          class="sort-btn"
          class:active={$viewMode === 'schedule'}
          onclick={() => viewMode.set('schedule')}
          title="スケジュール表示"
        ><ChartGantt size={12} strokeWidth={2} />スケジュール</button>
      </div>
    </div>

    {#if $viewMode === 'schedule'}
      <!-- ── スケジュール表示（ガントチャート） ─────────────── -->
      <ScheduleView
        onStatusChange={handleStatusChange}
        onDelete={handleDelete}
        onEdit={handleEdit}
      />
    {:else}
    {#snippet overdueSection()}
      {#if overdueLocalItems.length > 0}
        <div class="overdue-section">
          <div class="overdue-header">
            <span class="overdue-icon"><TriangleAlert size={14} strokeWidth={2} /></span>
            <span class="overdue-title">期限切れ</span>
            <span class="overdue-count">{overdueLocalItems.length}</span>
          </div>
          <div
            use:dndzone={{ items: overdueLocalItems, flipDurationMs: OVERDUE_DND_MS, type: 'tasks', dropFromOthersDisabled: true }}
            onconsider={handleOverdueConsider}
            onfinalize={handleOverdueFinalize}
            class="overdue-dnd-zone"
          >
            {#each overdueLocalItems as task (task.id)}
              {@const g = $groups.find(gr => gr.id === task.group_id)}
              <TaskItem
                {task}
                onStatusChange={handleStatusChange}
                onDelete={handleDelete}
                onEdit={handleEdit}
                groupLabel={g?.name}
                groupColor={g?.color ?? null}
                extraBadge={task.due_at ? overdueLabel(task.due_at) : undefined}
              />
            {/each}
          </div>
        </div>
      {/if}
    {/snippet}

    <div class="task-scroll">

      {#if $viewMode === 'grouped'}
        <!-- ── グループ表示 ─────────────────────────────── -->

        <!-- Today's tasks (grouped mode only) -->
        {#if todayLocalItems.length > 0}
          <div class="today-section">
            <div class="today-header">
              <span class="today-sun"><Sun size={14} strokeWidth={2} /></span>
              <span class="today-title">今日のタスク</span>
              <span class="today-count">{todayLocalItems.length}</span>
            </div>
            <div
              use:dndzone={{ items: todayLocalItems, flipDurationMs: TODAY_DND_MS, type: 'tasks' }}
              onconsider={handleTodayConsider}
              onfinalize={handleTodayFinalize}
              class="today-dnd-zone"
            >
              {#each todayLocalItems as task (task.id)}
                {@const g = $groups.find(gr => gr.id === task.group_id)}
                <TaskItem
                  {task}
                  onStatusChange={handleStatusChange}
                  onDelete={handleDelete}
                  onEdit={handleEdit}
                  groupLabel={g?.name}
                  groupColor={g?.color ?? null}
                />
              {/each}
            </div>
          </div>
        {/if}

        {@render overdueSection()}

        {#each $tasksByGroup.groups as group (group.id)}
          {#if ($tasksByGroup.map.get(group.id)?.length ?? 0) > 0 || $filterGroupId === group.id}
            <GroupSection
              {group}
              items={$tasksByGroup.map.get(group.id) ?? []}
              onReorder={handleReorder}
              onStatusChange={handleStatusChange}
              onDelete={handleDelete}
              onEdit={handleEdit}
            />
          {/if}
        {/each}

        {#if ($tasksByGroup.map.get(null)?.length ?? 0) > 0 || ($filterGroupId === null && $tasksByGroup.groups.length > 0)}
          <GroupSection
            group={null}
            items={$tasksByGroup.map.get(null) ?? []}
            onReorder={handleReorder}
            onStatusChange={handleStatusChange}
            onDelete={handleDelete}
            onEdit={handleEdit}
          />
        {/if}

      {:else}
        <!-- ── 一覧表示（フラット） ───────────────────────── -->
        {@render overdueSection()}

        {#each $sortedTasks as task (task.id)}
          {@const g = $groups.find(gr => gr.id === task.group_id)}
          <TaskItem
            {task}
            onStatusChange={handleStatusChange}
            onDelete={handleDelete}
            onEdit={handleEdit}
            groupLabel={g?.name}
            groupColor={g?.color ?? null}
          />
        {/each}
      {/if}

      {#if $tasks.length === 0}
        <div class="empty-state">
          <CheckSquare size={48} strokeWidth={1} class="empty-icon" />
          <p class="empty-title">タスクなし</p>
          <p class="empty-hint">上のフォームからタスクを追加しましょう</p>
        </div>
      {/if}
    </div>
    {/if}
  </main>
</div>

{#if editingTask}
  <EditModal
    task={editingTask}
    onSave={handleSave}
    onCancel={() => (editingTask = null)}
  />
{/if}

<style>
:global(.app-icon) { color: var(--accent); }
:global(.search-icon) { color: var(--text-muted); flex-shrink: 0; }
:global(.empty-icon) { color: var(--border); }

.layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* ── Sidebar ── */
.sidebar {
  width: 216px;
  flex-shrink: 0;
  background: var(--sidebar-bg);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  overflow-y: auto;
  overflow-x: hidden;
}
.sidebar-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 13px 14px 11px;
  border-bottom: 1px solid var(--border);
}
.app-name {
  font-weight: 600;
  font-size: 0.95rem;
  letter-spacing: -0.02em;
  color: var(--accent);
}
.sidebar-footer {
  margin-top: auto;
  border-top: 1px solid var(--border-light);
}

/* ── Search ── */
.search-wrap {
  padding: 9px 12px;
  border-bottom: 1px solid var(--border);
}
.search-inner {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 5px 10px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  transition: border-color 0.12s;
}
.search-inner:focus-within { border-color: var(--accent); }
.search-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text);
  font-size: 0.85rem;
  outline: none;
}
.search-input::placeholder { color: var(--text-muted); opacity: 0.7; }

/* ── Nav ── */
.nav-section {
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}
.nav-label {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 2px 12px 5px;
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.07em;
}
.label-with-icon {
  display: flex;
  align-items: center;
  gap: 4px;
}
.nav-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  text-align: left;
  padding: 5px 12px;
  font-size: 0.87rem;
  color: var(--text);
  cursor: pointer;
  border-radius: 0;
  transition: background 0.1s;
}
.nav-item:hover { background: var(--hover); }
.filter-check {
  margin-left: auto;
  font-size: 0.7rem;
  color: var(--accent);
  font-weight: 700;
}

.nav-item.active {
  background: var(--accent-soft);
  color: var(--accent);
  font-weight: 500;
}
.nav-icon {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  opacity: 0.7;
}
.nav-item.active .nav-icon { opacity: 1; }

.nav-item-row { display: flex; align-items: center; }
.flex1 { flex: 1; min-width: 0; overflow: hidden; }
.nav-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.dot {
  width: 7px; height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
}
.tag-dot {
  width: 7px; height: 7px;
  border-radius: 50%;
  flex-shrink: 0;
  background: var(--accent);
}
.icon-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 5px 6px;
  color: var(--text-muted);
  cursor: pointer;
  opacity: 0;
  border-radius: 4px;
  transition: color 0.1s, background 0.1s, opacity 0.1s;
}
.nav-item-row:hover .icon-btn { opacity: 1; }
.icon-btn:hover { background: var(--hover); color: var(--text); }
.icon-btn.danger:hover { color: var(--danger); background: var(--danger-soft); }

.mini-btn {
  display: flex;
  align-items: center;
  color: var(--text-muted);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  transition: color 0.1s, background 0.1s;
}
.mini-btn:hover { color: var(--accent); background: var(--accent-soft); }

.inline-form {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 5px 10px 8px;
}
.inline-form .mini-input {
  width: 100%;
}
.inline-form-row {
  display: flex;
  align-items: center;
  gap: 5px;
}
.today-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 0.75rem;
  color: var(--text-muted);
  cursor: pointer;
  user-select: none;
  padding: 0 2px;
}
.today-toggle input { accent-color: var(--accent); }
.color-swatches {
  display: flex;
  gap: 5px;
  padding: 0 2px;
  flex-wrap: wrap;
}
.swatch {
  width: 14px; height: 14px;
  border-radius: 50%;
  background: var(--c);
  border: 2px solid transparent;
  flex-shrink: 0;
  transition: transform 0.1s, border-color 0.1s;
  outline: none;
}
.swatch:hover { transform: scale(1.2); }
.swatch.sel { border-color: var(--text); }

.mini-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg);
  color: var(--text);
  font-size: 0.83rem;
}
.mini-input:focus { outline: none; border-color: var(--accent); }
.mini-btn-accent {
  padding: 4px 10px;
  border-radius: 6px;
  background: var(--accent);
  color: #fff;
  font-size: 0.8rem;
  cursor: pointer;
  font-weight: 500;
}

/* ── Main ── */
.main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--bg);
}
/* ── Paste bar ───────────────────────────────── */
.update-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 9px 16px;
  background: rgba(99,102,241,.12);
  border-bottom: 1px solid rgba(99,102,241,.3);
  font-size: 0.85rem;
}
.update-msg { color: var(--text); }
.update-actions { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }
.update-btn {
  padding: 4px 14px;
  border-radius: 8px;
  background: var(--accent);
  color: #fff;
  font-size: 0.82rem;
  font-weight: 500;
  cursor: pointer;
  transition: opacity .12s;
}
.update-btn:disabled { opacity: .55; cursor: default; }
.update-btn:not(:disabled):hover { opacity: .85; }
.update-dismiss {
  display: flex; align-items: center; justify-content: center;
  width: 24px; height: 24px;
  border-radius: 6px;
  color: var(--text-muted);
  background: transparent;
  transition: background .1s, color .1s;
}
.update-dismiss:hover { background: var(--hover); color: var(--text); }

.paste-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 14px;
  background: var(--accent-soft);
  border-bottom: 1px solid var(--accent);
  color: var(--accent);
  flex-shrink: 0;
  font-size: 0.78rem;
}
.paste-preview {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.paste-preview strong { font-weight: 600; }
.paste-btn {
  padding: 3px 12px;
  border-radius: 6px;
  background: var(--accent);
  color: #fff;
  font-size: 0.75rem;
  font-weight: 600;
  cursor: pointer;
  border: none;
  white-space: nowrap;
  transition: opacity 0.1s;
}
.paste-btn:hover { opacity: 0.85; }
.paste-clear {
  display: flex; align-items: center; justify-content: center;
  background: transparent;
  border: none;
  color: var(--accent);
  cursor: pointer;
  opacity: 0.65;
  border-radius: 4px;
  padding: 2px;
  transition: opacity 0.1s;
}
.paste-clear:hover { opacity: 1; }

/* ── Sort toolbar ─────────────────────────────── */
.sort-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 14px;
  border-bottom: 1px solid var(--border-light);
  flex-shrink: 0;
  background: var(--bg);
}
.sort-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 0.72rem;
  color: var(--text-muted);
  user-select: none;
}
.sort-btns { display: flex; gap: 2px; }
.view-toggle {
  display: flex;
  gap: 2px;
  margin-left: auto;
  padding-left: 10px;
  border-left: 1px solid var(--border-light);
}
.sort-btn {
  display: flex;
  align-items: center;
  gap: 3px;
  padding: 3px 9px;
  border-radius: 6px;
  border: 1px solid transparent;
  font-size: 0.72rem;
  color: var(--text-muted);
  background: transparent;
  cursor: pointer;
  transition: background 0.1s, color 0.1s, border-color 0.1s;
}
.sort-btn:hover { background: var(--hover); color: var(--text); }
.sort-btn.active {
  background: var(--accent-soft);
  color: var(--accent);
  border-color: var(--accent);
  font-weight: 600;
}

/* ── Today section ────────────────────────────── */
.today-section { border-bottom: 2px solid var(--border-light); }
.today-dnd-zone { outline: none; }
.today-header {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 14px 6px;
  background: var(--bg);
}
.today-sun { color: #F59E0B; }
.today-title {
  font-size: 0.78rem;
  font-weight: 700;
  color: var(--text);
  letter-spacing: 0.01em;
  flex: 1;
}
.today-count {
  font-size: 0.7rem;
  font-weight: 600;
  background: rgba(245,158,11,0.15);
  color: #B45309;
  padding: 1px 7px;
  border-radius: 10px;
}

/* ── Overdue section ──────────────────────────── */
.overdue-section { border-bottom: 2px solid var(--border-light); }
.overdue-dnd-zone { outline: none; }
.overdue-header {
  display: flex;
  align-items: center;
  gap: 7px;
  padding: 8px 14px 6px;
  background: var(--bg);
}
.overdue-icon { color: var(--danger); display: flex; align-items: center; }
.overdue-title {
  font-size: 0.78rem;
  font-weight: 700;
  color: var(--danger);
  letter-spacing: 0.01em;
  flex: 1;
}
.overdue-count {
  font-size: 0.7rem;
  font-weight: 600;
  background: var(--danger-soft);
  color: var(--danger);
  padding: 1px 7px;
  border-radius: 10px;
}

/* ── Main scroll area ─────────────────────────── */
.task-scroll {
  flex: 1;
  overflow-y: auto;
}
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 60%;
  gap: 10px;
  color: var(--text-muted);
}
.empty-title { font-size: 0.95rem; font-weight: 500; }
.empty-hint { font-size: 0.82rem; opacity: 0.7; }
</style>
