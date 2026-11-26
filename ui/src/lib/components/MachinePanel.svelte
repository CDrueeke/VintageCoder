<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import AreaButton from "./AreaButton.svelte";
  import { bau, buildRun, loadTargets, runStop } from "../bau.svelte";
  import { t } from "../i18n.svelte";
  import { ui } from "../state.svelte";
  import { aktiv, bildHolen, dbg, hex as hx } from "../debug.svelte";
  // Maschine: Ziel, Zustand und Bauergebnis (Ladeadresse, Start, Segmente). Register, Flags und Bild kommen mit der Debug-Sitzung (E6).
  $effect(() => { if (ui.project) void loadTargets(); });
  const hex = (n: number) => "$" + n.toString(16).toUpperCase().padStart(4, "0");
  const info = $derived(bau.info);
  const state = $derived(bau.started ? t("machine-laeuft") : bau.running ? t("bar-baut") : t("machine-bereit"));
</script>

<div class="m">
  <div class="mh"><h2>{t("machine-title")}</h2><AreaButton area="panel" /></div>
  <dl>
    <dt>{t("machine-system")}</dt><dd>{bau.targets?.system ?? "–"}</dd>
    <dt>{t("machine-ziel")}</dt><dd>{bau.targets?.name ?? "–"}</dd>
    <dt>{t("machine-zustand")}</dt><dd role="status">{state}</dd>
  </dl>
  <div class="act">
    <button type="button" class="btn" disabled={bau.running} onclick={() => void buildRun()}>{t("bar-bauen")}</button>
    <button type="button" class="btn" disabled={bau.running || bau.starting} onclick={() => void buildRun({ then: "run" })}>{t("bar-starten")}</button>
    <button type="button" class="btn" disabled={!bau.started} onclick={() => void runStop()}>{t("bar-stopp")}</button>
  </div>
  {#if aktiv() && dbg.s}
    {@const d = dbg.s}
    <h3>{t("machine-debug")}</h3>
    <div data-mp="debug">
      {#if d.regs}<dl class="mono" data-mp="regs"><dt>PC</dt><dd>${hx(d.regs.pc, 4)}</dd><dt>A X Y</dt><dd>{hx(d.regs.a, 2)} {hx(d.regs.x, 2)} {hx(d.regs.y, 2)}</dd><dt>SP</dt><dd>{hx(d.regs.sp, 2)}</dd><dt>{t("machine-flags")}</dt><dd>{"NV-BDIZC".split("").map((f, i) => (d.regs!.flags & (0x80 >> i) ? f : "·")).join("")}</dd></dl>
      {:else}<p class="none">{d.running ? t("dbgui-status-laeuft") : t("dbgui-status-startet")}</p>{/if}
      {#if d.frame?.can}
        <div class="bild" data-mp="bild">{#if dbg.frame}<img src={dbg.frame.img} alt={t("dbgui-bild-alt")} width={Math.min(dbg.frame.w, 320)} />{:else}<span class="none">{t("dbgui-bild-leer")}</span>{/if}</div>
        <button type="button" class="btn" data-mp="bild-holen" onclick={() => void bildHolen()}>{t("dbgui-bild-aktualisieren")}</button>
      {/if}
    </div>
  {/if}
  <h3>{t("machine-bau")}</h3>
  {#if info}
    <dl>
      <dt>{t("machine-quelle")}</dt><dd class="mono">{info.rel}</dd>
      <dt>{t("machine-datei")}</dt><dd class="mono">{info.artifact.split("/").slice(-2).join("/")}</dd>
      <dt>{t("machine-groesse")}</dt><dd>{t("machine-bytes", { n: info.size })}</dd>
      <dt>{t("machine-start")}</dt><dd class="mono">{info.run !== null ? hex(info.run) : "–"}</dd>
    </dl>
    <table>
      <thead><tr><th>{t("machine-segment")}</th><th>{t("machine-von")}</th><th>{t("machine-bis")}</th><th>{t("machine-laenge")}</th></tr></thead>
      <tbody>{#each info.segments as [s, l], i (i)}<tr class="mono"><td>{i + 1}</td><td>{hex(s)}</td><td>{hex(s + l - 1)}</td><td>{l}</td></tr>{/each}</tbody>
    </table>
  {:else}<p class="none">{t("machine-kein-bau")}</p>{/if}
  {#if !aktiv()}<p class="note">{t("machine-spaeter")}</p>{/if}
</div>

<style>
  .mh { display: flex; align-items: center; justify-content: space-between; }
  .m { padding: 10px 14px; font-size: 12.5px; overflow: auto; height: 100%; box-sizing: border-box; }
  h2 { font-size: 13px; margin: 0 0 8px; } h3 { font-size: 12px; margin: 14px 0 6px; color: var(--dim); }
  dl { display: grid; grid-template-columns: auto 1fr; gap: 3px 12px; margin: 0; } dt { color: var(--dim); } dd { margin: 0; word-break: break-all; }
  .act { display: flex; gap: 6px; margin-top: 10px; flex-wrap: wrap; }
  table { border-collapse: collapse; width: 100%; margin-top: 8px; font-size: 12px; } th, td { text-align: left; padding: 2px 8px 2px 0; border-bottom: 1px solid var(--line); } th { color: var(--dim); font-weight: 600; }
  .none, .note { color: var(--dim); } .note { margin-top: 16px; font-size: 11.5px; }
</style>
