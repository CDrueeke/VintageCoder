<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { t } from "../lib/i18n.svelte";
  import type { Combo } from "../lib/keys";
  import Card from "../lib/components/Card.svelte";
  import DataTable from "../lib/components/DataTable.svelte";
  import Dialog from "../lib/components/Dialog.svelte";
  import Icon from "../lib/components/Icon.svelte";
  import Kbd from "../lib/components/Kbd.svelte";
  import KeyCapture from "../lib/components/KeyCapture.svelte";
  import Menu from "../lib/components/Menu.svelte";
  import Segment from "../lib/components/Segment.svelte";
  import Slider from "../lib/components/Slider.svelte";
  import Switch from "../lib/components/Switch.svelte";
  import Tooltip from "../lib/components/Tooltip.svelte";
  // Entwicklungsansicht: alle Bausteine der Komponentenbibliothek an einem Ort (Handprüfung, Bilder, Layout in de/en/xx)
  let mode = $state("mc");
  let grid = $state(true);
  let zoom = $state(12);
  let row = $state(0);
  let dlg = $state(false);
  let combo = $state<Combo | null>(null);
  let picked = $state("");
  const rows = $derived([
    { name: "jumprun.asm", size: "2 KB", kind: t("gallery-row-asm") },
    { name: "title.vcsong", size: "1 KB", kind: t("gallery-row-song") },
    { name: "hero.vcspr", size: "1 KB", kind: t("gallery-row-sprite") },
  ]);
</script>

<div style="overflow:auto;padding:14px;display:flex;flex-direction:column;gap:12px;min-height:0">
  <div><h2 style="font-size:18px">{t("gallery-title")}</h2><p style="color:var(--dim);max-width:720px">{t("gallery-intro")}</p></div>
  <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:12px">
    <Card title={t("gallery-card-title")}><div class="c"><p style="color:var(--dim)">{t("gallery-card-body")}</p>
      <p style="margin-top:10px;display:flex;gap:8px"><button type="button" class="btn pri">{t("gallery-button-primary")}</button><button type="button" class="btn">{t("gallery-button-plain")}</button></p></div></Card>
    <Card title={t("gallery-segment-label")}><div class="c" style="display:flex;flex-direction:column;gap:14px">
      <Segment small label={t("gallery-segment-label")} value={mode} onselect={(v) => (mode = v)} items={[{ id: "mc", label: t("gallery-segment-a") }, { id: "hi", label: t("gallery-segment-b") }]} />
      <Switch bind:checked={grid} label={t("gallery-switch-label")} />
      <Slider bind:value={zoom} min={4} max={40} label={t("gallery-slider-label")} />
    </div></Card>
    <Card title={t("gallery-kbd-label")}><div class="c" style="display:flex;flex-direction:column;gap:12px">
      <Kbd parts={["⌘", "⇧", "S"]} /><KeyCapture bind:combo label={t("gallery-keycapture-label")} />
      <Tooltip text={t("gallery-tooltip-text")}><button type="button" class="btn wrap">{t("gallery-tooltip-target")}</button></Tooltip>
    </div></Card>
    <Card title={t("gallery-table-label")}><div class="c" style="padding:0"><DataTable bind:selected={row} label={t("gallery-table-label")} cols={[{ id: "name", label: t("gallery-table-col-name") }, { id: "size", label: t("gallery-table-col-size") }, { id: "kind", label: t("gallery-table-col-kind") }]} {rows} /></div></Card>
    <Card title={t("gallery-dialog-title")}><div class="c" style="display:flex;gap:10px;align-items:center">
      <button type="button" class="btn" onclick={() => (dlg = true)}>{t("gallery-dialog-open")}</button>
      <Menu label={t("gallery-menu-open")} radio onselect={(id) => (picked = id)} items={[{ id: "a", label: t("gallery-menu-a"), checked: picked === "a" }, { id: "b", label: t("gallery-menu-b"), checked: picked === "b" }, { id: "c", label: t("gallery-menu-c"), disabled: true }]}>
        {#snippet button()}<Icon name="chevd" size={16} />{/snippet}
      </Menu>
    </div></Card>
  </div>
</div>
<Dialog bind:open={dlg} title={t("gallery-dialog-title")}>
  {t("gallery-dialog-body")}
  {#snippet footer()}<button type="button" class="btn pri" onclick={() => (dlg = false)}>{t("dialog-ok")}</button>{/snippet}
</Dialog>
