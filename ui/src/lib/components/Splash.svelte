<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script module lang="ts">
  /** Dauer des Startbilds in Millisekunden (Ausblenden eingeschlossen); Klick oder Taste beendet es früher */
  export const SPLASH_MS = 2800;
</script>

<script lang="ts">
  // Startbild: etwa drei Sekunden (SPLASH_MS), Klick oder Taste schließt es früher. Abschaltbar unter Einstellungen > Allgemein; in der Prüfbrücke aus (außer ?splash=1).
  import { onMount } from "svelte";
  import { ui } from "../state.svelte";
  import { t } from "../i18n.svelte";
  import Wortmarke from "./Wortmarke.svelte";

  const q = typeof location !== "undefined" ? new URLSearchParams(location.search) : new URLSearchParams();
  let offen = $state(ui.splash && (!q.has("bridge") || q.has("splash")));
  const zu = () => { offen = false; };
  onMount(() => { if (!offen) return; const id = setTimeout(zu, SPLASH_MS); const k = () => zu(); window.addEventListener("keydown", k, { once: true }); window.addEventListener("pointerdown", k, { once: true }); return () => { clearTimeout(id); window.removeEventListener("keydown", k); window.removeEventListener("pointerdown", k); }; });
</script>

{#if offen}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="splash" data-splash role="presentation" onclick={zu} style={`--aus-start: ${SPLASH_MS - 350}ms`}>
    <h1 class="logo" data-splash-logo><Wortmarke hoehe={72} /></h1>
    <p>{t("splash-text")}</p>
  </div>
{/if}

<style>
  .splash { position: fixed; inset: 0; z-index: 9000; display: grid; place-content: center; justify-items: center; gap: 10px; background: var(--bg); animation: aus 0.35s ease var(--aus-start, 2450ms) forwards; }
  .logo { margin: 0; line-height: 0; max-width: 88vw; }
  p { margin: 0; color: var(--dim); }
  @keyframes aus { to { opacity: 0; } }
  @media (prefers-reduced-motion: reduce) { .splash { animation: none; } }
</style>
