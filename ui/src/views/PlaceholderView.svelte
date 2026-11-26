<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import { section, type Mode } from "../lib/nav";
  import { ui } from "../lib/state.svelte";
  import { isTurbo } from "../lib/turbo.svelte";
  import Icon from "../lib/components/Icon.svelte";
  // Platzhalter für Bereiche, die in späteren Etappen gebaut werden; nennt die Etappe
  let { mode }: { mode: Mode } = $props();
  const s = $derived(section(mode, ui.sub[mode]));
</script>

<div class="empty">
  <Icon name={mode === "musik" ? "music" : mode === "debug" ? "bug" : mode === "daten" ? "memory" : mode === "grafik" ? "sprite" : "code"} size={28} />
  <h2>{t(s.title)}</h2>
  <p>{t("view-placeholder-text", { stage: s.stage })}</p>
  <p style="color:var(--faint)">{t("view-placeholder-hint")}</p>
  {#if mode === "debug" && isTurbo()}<p role="note" style="color:var(--warn)" data-testid="turbo-debug-grund">{t("turboui-debuggen-hinweis")}</p>{/if}
  {#if mode === "code" && ui.selected}<p class="mono" style="margin-top:8px;color:var(--dim)">{ui.selected}</p>{/if}
</div>
