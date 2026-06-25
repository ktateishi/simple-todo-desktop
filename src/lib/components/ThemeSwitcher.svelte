<script lang="ts">
  const themes = [
    { id: 'airy',  label: 'Airy',  bg: '#FFFFFF', accent: '#5753E8' },
    { id: 'bloom', label: 'Bloom', bg: '#FFFFFF', accent: '#F43F5E' },
    { id: 'noir',  label: 'Noir',  bg: '#0E0E11', accent: '#5B8AF5' },
    { id: 'dusk',  label: 'Dusk',  bg: '#130F0C', accent: '#D97706' },
    { id: 'ash',   label: 'Ash',   bg: '#0A0A0A', accent: '#737373' },
  ] as const;

  let current = $state(
    typeof localStorage !== 'undefined'
      ? (localStorage.getItem('theme') ?? 'airy')
      : 'airy'
  );

  function setTheme(id: string) {
    current = id;
    document.documentElement.setAttribute('data-theme', id);
    localStorage.setItem('theme', id);
  }
</script>

<div class="theme-wrap">
  <span class="label">テーマ</span>
  <div class="dots">
    {#each themes as t}
      <button
        class="dot"
        class:active={current === t.id}
        onclick={() => setTheme(t.id)}
        title={t.label}
        aria-label="{t.label}テーマ"
        style="--bg:{t.bg}; --ac:{t.accent}"
      >
        <span class="dot-bg"></span>
        <span class="dot-accent"></span>
      </button>
    {/each}
  </div>
</div>

<style>
.theme-wrap {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 14px 12px;
}
.label {
  font-size: 0.7rem;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: 0.08em;
  user-select: none;
}
.dots { display: flex; gap: 6px; }

.dot {
  position: relative;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  overflow: hidden;
  border: 1.5px solid transparent;
  padding: 0;
  transition: transform 0.12s, border-color 0.12s;
}
.dot:hover { transform: scale(1.15); }
.dot.active { border-color: var(--text); }

.dot-bg {
  position: absolute;
  inset: 0;
  background: var(--bg);
}
.dot-accent {
  position: absolute;
  bottom: 0; right: 0;
  width: 10px; height: 10px;
  background: var(--ac);
  border-radius: 10px 0 0 0;
}
</style>
