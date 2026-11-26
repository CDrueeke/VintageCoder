<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import { lay, toggleMax } from "../layout.svelte";
  import type { Area } from "../layout";
  import { t } from "../i18n.svelte";
  import Icon from "./Icon.svelte";
  import Tooltip from "./Tooltip.svelte";
  // Knopf im Kopf eines Bereichs: maximieren und wiederherstellen (derselbe Weg wie Doppelklick auf den Kopf und die Befehle der Ansicht)
  let { area }: { area: Area } = $props();
  const on = $derived(lay.max === area);
  const label = $derived(t(on ? "bereich-wiederherstellen" : "bereich-maximieren"));
</script>

<Tooltip text={label}><button type="button" class="ibtn sm" aria-label={label} aria-pressed={on} data-max={area} onclick={() => toggleMax(area)}><Icon name={on ? "restore" : "max"} size={14} /></button></Tooltip>
