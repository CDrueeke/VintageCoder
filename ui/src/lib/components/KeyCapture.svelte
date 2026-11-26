<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { codeLabel, comboFromEvent, comboParts, learnLayout, type Combo } from "../keys";
  import { t } from "../i18n.svelte";
  import Kbd from "./Kbd.svelte";
  // Tastenfang: erfasst die nächste Taste nach `KeyboardEvent.code` (physisch), Esc bricht ab, Rücktaste setzt zurück
  let { combo = $bindable(null), label }: { combo?: Combo | null; label: string } = $props();
  let listening = $state(false);
  const names = $derived({ ctrl: t("key-ctrl"), shift: t("key-shift"), alt: "Alt" });
  function down(e: KeyboardEvent) {
    if (!listening) return;
    e.preventDefault(); e.stopPropagation();
    learnLayout(e);
    if (e.code === "Escape") { listening = false; return; }
    const c = comboFromEvent(e);
    if (!c) return;
    combo = c; listening = false;
  }
  const parts = $derived(combo ? comboParts(combo, names) : []);
  void codeLabel;
</script>

<span class="row" style="display:inline-flex;align-items:center;gap:8px">
  <button type="button" class="btn wrap" aria-label={label} aria-pressed={listening} onclick={() => (listening = !listening)} onkeydown={down} onblur={() => (listening = false)}>
    {#if listening}{t("keycapture-active")}{:else if combo}<Kbd {parts} />{:else}{t("keycapture-idle")}{/if}
  </button>
  {#if combo}<button type="button" class="btn" onclick={() => (combo = null)}>{t("keycapture-clear")}</button>{/if}
  <span class="sr-only" aria-live="polite">{listening ? t("keycapture-active") : parts.join(" ")}</span>
</span>
