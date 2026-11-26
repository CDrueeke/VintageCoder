<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../i18n.svelte";
  import { openProvide, tb } from "../turbo.svelte";
  import Icon from "./Icon.svelte";
  // Banner „Diskette fehlt“ über dem Editor (Konzept 9): erscheint, wenn ein Turbo-BASIC-XL-Projekt ohne Diskette gestartet wird; „Bereitstellen …“ öffnet den Assistenten
</script>

{#if tb.banner && tb.state?.turbo && !tb.state.present}
  <div class="tbn" role="region" aria-label={t("turboui-banner-label")} data-testid="turbo-banner">
    <Icon name="disk" size={16} /><span class="txt">{t("turboui-banner-text")}</span>
    <button type="button" class="btn pri" onclick={() => openProvide(tb.state?.disk ? "replace" : "provide")} data-testid="turbo-banner-bereitstellen">{t("turboui-banner-bereitstellen")}</button>
    <button type="button" class="ibtn" aria-label={t("turboui-banner-schliessen")} onclick={() => (tb.banner = false)}><Icon name="close" size={14} /></button>
  </div>
{/if}

<style>
  .tbn { display: flex; align-items: center; gap: 10px; padding: 7px 12px; margin: 0 0 6px; border: 1px solid var(--warn); border-radius: 8px; background: var(--raise); font-size: 12.5px; }
  .txt { flex: 1; min-width: 0; }
  @media (max-width: 560px) { .tbn { flex-wrap: wrap; } }
  @media (forced-colors: active) { .tbn { border: 2px solid CanvasText; } }
</style>
