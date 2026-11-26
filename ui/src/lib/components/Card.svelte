<!-- SPDX-License-Identifier: GPL-3.0-or-later -->
<script lang="ts">
  import type { Snippet } from "svelte";
  // Schwebende Fläche mit Kopfzeile (Großbuchstaben, gedämpft); als Bereich mit Namen für die Sprachausgabe
  let { title, header, children, width, class: cls = "" }: { title: string; header?: Snippet; children: Snippet; width?: number; class?: string } = $props();
  const id = `card-${Math.random().toString(36).slice(2, 8)}`;
</script>

<section class="card {cls}" aria-labelledby={id} style={width ? `width:${width}px;flex:none` : undefined}>
  <div class="h"><span {id}>{title}</span><span class="sp"></span>{@render header?.()}</div>
  {@render children()}
</section>
