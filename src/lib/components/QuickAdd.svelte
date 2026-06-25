<script lang="ts">
  import { createVoiceInput, parseInput } from '$lib/voice';
  import { groups } from '$lib/stores';
  import Mic    from 'lucide-svelte/icons/mic';
  import MicOff from 'lucide-svelte/icons/mic-off';
  import Plus   from 'lucide-svelte/icons/plus';

  type AddPayload = { title: string; tagNames: string[]; groupId: number | undefined };

  type Props = {
    onAdd: (data: AddPayload) => void;
  };

  let { onAdd }: Props = $props();

  const voice = createVoiceInput();
  let raw = $state('');
  let listening = $state(false);
  let selectedGroupId = $state<number | undefined>(undefined);

  function submit(e: SubmitEvent) {
    e.preventDefault();
    const trimmed = raw.trim();
    if (!trimmed) return;
    const { title, tagNames } = parseInput(trimmed);
    if (!title) return;
    onAdd({ title, tagNames, groupId: selectedGroupId });
    raw = '';
  }

  function toggleVoice() {
    if (listening) {
      voice.stop();
      listening = false;
      return;
    }
    listening = true;
    voice.start({
      onResult: (text) => { raw = text; listening = false; },
      onError:  ()     => { listening = false; },
      onEnd:    ()     => { listening = false; },
    });
  }

  function onKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) { e.preventDefault(); }
  }
</script>

<form class="quick-add" onsubmit={submit}>
  <select bind:value={selectedGroupId} class="group-select" aria-label="グループ選択">
    <option value={undefined}>グループなし</option>
    {#each $groups as g (g.id)}
      <option value={g.id}>{g.name}</option>
    {/each}
  </select>

  <input
    bind:value={raw}
    onkeydown={onKeydown}
    placeholder="タスクを追加… (#タグ で分類)"
    class="task-input"
    aria-label="タスク入力"
  />

  <button
    type="button"
    onclick={toggleVoice}
    class="voice-btn"
    class:active={listening}
    title={listening ? '録音停止' : '音声入力'}
    aria-label="音声入力"
  >
    {#if listening}
      <MicOff size={15} strokeWidth={2} />
    {:else}
      <Mic size={15} strokeWidth={2} />
    {/if}
  </button>

  <button type="submit" class="add-btn" disabled={!raw.trim()}>
    <Plus size={15} strokeWidth={2.5} />
    追加
  </button>
</form>

<style>
.quick-add {
  display: flex;
  gap: 7px;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--surface);
}
.group-select {
  flex: 0 0 160px;
  min-width: 0;
  padding: 6px 8px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  color: var(--text-muted);
  font-size: 0.82rem;
}
.group-select:focus { outline: none; border-color: var(--accent); }

.task-input {
  flex: 1;
  padding: 7px 12px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  color: var(--text);
  font-size: 0.9rem;
  transition: border-color 0.12s;
}
.task-input::placeholder { color: var(--text-muted); opacity: 0.7; }
.task-input:focus { outline: none; border-color: var(--accent); }

.voice-btn {
  padding: 6px 9px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg);
  color: var(--text-muted);
  display: flex; align-items: center; justify-content: center;
  transition: background 0.12s, border-color 0.12s, color 0.12s;
}
.voice-btn:hover { background: var(--hover); color: var(--text); }
.voice-btn.active {
  background: var(--danger-soft);
  border-color: var(--danger);
  color: var(--danger);
}

.add-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 7px 13px;
  border-radius: 8px;
  background: var(--accent);
  color: #fff;
  font-size: 0.85rem;
  font-weight: 500;
  letter-spacing: -0.01em;
  transition: opacity 0.12s;
  white-space: nowrap;
}
.add-btn:disabled { opacity: 0.35; cursor: not-allowed; }
.add-btn:not(:disabled):hover { opacity: 0.85; }
</style>
