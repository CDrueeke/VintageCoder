<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { cfg, einzeln, feedbackKopieren, feedbackMail, inf } from "../settings.svelte";
  import Dialog from "./Dialog.svelte";
  import Switch from "./Switch.svelte";
  import { t } from "../i18n.svelte";
  // Feedback (E12.4): bereitet eine Mail vor; gesendet wird erst im Mailprogramm. Der Text enthält nur technische Angaben (Version, System), nie Dateien oder Namen.
  let offen = $state(false);
  $effect(() => { offen = inf.fbOpen; }); $effect(() => { if (!offen && inf.fbOpen) inf.fbOpen = false; });
</script>
{#if offen && inf.fb}
  <Dialog bind:open={offen} title={t("info-feedback-titel")}>
    {#snippet children()}
      <div class="fb" data-fb="dialog">
        <p class="small dim">{t("info-feedback-hinweis")}</p>
        <label class="small dim">{t("info-betreff")}<input class="inp" readonly value={inf.fb?.subject ?? ""} /></label>
        <label class="small dim">{t("info-text")}<textarea class="inp" readonly rows="9" data-fb="text">{inf.fb?.bodyFull}</textarea></label>
        {#if inf.fb?.tooLong || inf.fb?.dropped.length}<p class="small warn">{t("info-gekuerzt")}</p>{/if}
        <Switch checked={!!cfg.saved?.debug_log} label={t("einst-debuglog")} onchange={(on) => void einzeln((c) => { c.debug_log = on; })} />
        {#if inf.fbMsg}<p class="small ok" role="status" data-fb="msg">{inf.fbMsg}</p>{/if}
      </div>
    {/snippet}
    {#snippet footer()}
      <button type="button" class="btn" onclick={() => (inf.fbOpen = false)}>{t("dialog-abbrechen")}</button>
      <button type="button" class="btn" data-fb="kopieren" onclick={() => void feedbackKopieren()}>{t("info-kopieren")}</button>
      <button type="button" class="btn pri" data-fb="mail" onclick={() => void feedbackMail()}>{t("info-mail-oeffnen")}</button>
    {/snippet}
  </Dialog>
{/if}
<style>
  .fb { display: flex; flex-direction: column; gap: 8px; min-width: 420px; } label { display: flex; flex-direction: column; gap: 4px; } .inp { width: 100%; font: inherit; } textarea.inp { font-family: var(--font-mono, monospace); font-size: 12px; resize: vertical; }
  .small { font-size: 12px; margin: 0; } .dim { color: var(--dim); } .warn { color: var(--warn); } .ok { color: var(--ok); }
</style>
