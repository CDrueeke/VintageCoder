# Messwerte E0 (Prüfpunkt G1)

Stand: 2026-11-20 · Commit `66f4aeb` und Vorgänger · Gerät: Mac mit Apple Silicon (arm64), macOS (WKWebView), Release-Bau mit Feature `selftest`. Rohdaten: `~/Development/abnahme-ui/e0/messung-macos/selftest-{1,2,3}.json` (drei Läufe, Lauf 1 kalt).

## Rundlauf und Zeichnen (echtes Fenster, Tauri-IPC)

| Messung | Mittelwert (ms) Lauf 1/2/3 | Median (ms)* | Höchstwert (ms) Lauf 1/2/3 |
|---|---|---|---|
| Befehl `ping`, kleine Nutzlast (1 000 Aufrufe) | 0.308 / 0.254 / 0.335 | 0 | 7 / 7 / 5 |
| Befehl `raster_apply` mit 3 Zellen, Delta-Antwort (1 000) | 0.336 / 0.218 / 0.332 | 0 | 2 / 5 / 3 |
| Binärantwort 64 KiB (200) | 0.270 / 0.180 / 0.315 | 0 | 1 / 1 / 1 |
| Zeigerereignis bis Strich, nur lokal (1 000) | 0.003 / 0.001 / 0.001 | 0 | 2 / 1 / 1 |
| Zeigerereignis bis Strich, gebündelt mit Rundlauf zu Rust (1 000) | 0.214 / 0.181 / 0.393 | 0 | 1 / 1 / 5 |
| Zeigerereignis bis gezeichnetes Bild (vsync), nur lokal (200) | 19.990 / 19.980 / 19.960 | 20 | 21 / 22 / 22 |
| Zeigerereignis bis gezeichnetes Bild (vsync), gebündelt (200) | 19.995 / 19.945 / 19.975 | 20 | 21 / 22 / 21 |

Start (Prozessstart bis Oberfläche fertig, Läufe 1/2/3): 1505 ms / 489 ms / 578 ms

\* **WebKit rundet `performance.now()` auf 1 ms** (Schutz gegen Zeitmessung). Einzelzeiten, Median und Höchstwert haben daher 1 ms Auflösung; der **Mittelwert** (Gesamtzeit der Schleife durch Anzahl) ist genau. Der Median „0“ heißt: weniger als 1 ms. Der Höchstwert 5 bis 7 ms tritt vereinzelt auf (erste Aufrufe, Speicherbereinigung).

**Lesart:** Ein Kernbefehl kostet im Mittel **0,2 bis 0,4 ms** Rundlauf (JSON-Nutzlast klein, Pixelraster-Delta, 64 KiB binär ähnlich). Der gebündelte Strich (lokaler Vorschaustrich plus Vorgang an Rust plus Zeichnen der Bestätigung) braucht im Mittel 0,2 bis 0,4 ms; der lokale Strich allein ist mit 0,001 bis 0,003 ms nicht messbar. Das liegt um mehr als eine Größenordnung unter einem Bild (16,7 ms bei 60 Hz); der Rundlauf ist für das Zeichnen kein Hindernis (Risiko R3 nach diesen Zahlen gering). „Bis gezeichnetes Bild“ misst gegen die Bildwiederholung und enthält zwei Bildwechsel (`requestAnimationFrame` zweimal): auf dem Messgerät 20 ms (Anzeige mit hoher Bildrate), also kein Unterschied zwischen lokal und gebündelt. **Synthetische Ereignisse** (ohne Maus) messen Verarbeitung, Rundlauf und Zeichnen, nicht die Latenz des Eingabegeräts.

## Start, Speicher, Größe

| Messung | Wert |
|---|---|
| Start: Prozessstart bis Oberfläche fertig (Sprache und Befehlstabelle geladen) | 489 bis 578 ms warm, 1 505 ms im ersten (kalten) Lauf |
| Speicher im Leerlauf nach 15 s, alle Prozesse (RSS) | App 92,4 MB + WebContent 46,9 MB + GPU 20,7 MB + Netzwerk 7,8 MB = **etwa 168 MB** (Spike: 225 MB mit weniger Funktionen im Kern) |
| Leerlauf-CPU | 0,0 % |
| Größe `VintageCoder.app` | 12,4 MiB (Binärdatei 12,9 MB, Frontend 140 KB: 99,9 kB JS + 10 kB CSS, ungepackt) |
| Frontend-Pakete (npm) | 109 Einträge in der Sperrdatei (alle Plattformen), davon 53 auf diesem System installiert; Budget „etwa 60 je System“ eingehalten |
| Rust-Pakete | `Cargo.lock` jetzt 662 Pakete für die GUI-Gruppe (Tauri-Baum rund 430, siehe `THIRD-PARTY-NOTICES.md`) |
| Schrift | System-UI-Schrift; Festbreitenschrift noch nicht gebündelt (F2) |

## Linux x86_64 und Windows

| System | Stand |
|---|---|
| Windows 11 Pro (Build 26300; Ryzen 5 3400G, 29,9 GB, Radeon Vega 11; WebView2 157.0.4341.60; Rust 1.100.0, Node 26.13.1) | **gemessen** (3 Läufe, Release mit `selftest`, Commit 66f4aeb, frischer Klon; Bau 10 min 17 s, `vintagecoder.exe` 11,6 MB). Start bereit seit Prozessstart 1443–1712 ms. Aufruf-Rundlauf (Median / p95): ping-small 3,2–3,3 / 5–5,5 ms; raster-delta 3,4–3,6 / 4,6–4,8 ms; binary 5,3–5,4 / 6,7–7,6 ms. Zeiger bis Bild: lokal und gebündelt je 62,5 ms (p95 63,6), nach Anschein fester Takt, nicht untersucht; synchron lokal 0 ms, gebündelt 3,8 ms. Leerlauf nach 20 s: Arbeitssätze 396 MB, privat 220 MB (vintagecoder.exe 33 MB, 5 WebView2-Prozesse zusammen 363 MB). Messbild der Tabelle liegt vor (Nachlieferung: kleine Nutzlast 3,1 ms, Pixelraster 3,2 ms, Binärantwort 4,8 ms; Bild 62,5 ms, Höchstwert 64,3); Hell/Dunkel nach System ungeprüft. Rohdaten: `Y:\austausch-g1\windows\` (auf dem Mac nicht erreichbar) |
| Linux x86_64 (Ubuntu 24.04.5, GNOME/X11, Hyper-V-VM mit 4 Kernen und 4 GB; **Software-Rendering** Mesa llvmpipe ohne GPU; libwebkit2gtk-4.1 2.52.6; Rust 1.100.0, Node 22.23.5) | **gemessen** (je 3 Läufe bei 1920x1080 und 1280x800, Release mit `selftest`, Commit 66f4aeb; Bau 5 min 27 s, Binärdatei 14,9 MB). Start bereit 1,12–1,22 s (1280x800: 1,16–1,20 s). Aufruf-Rundlauf (Median): kleine Nutzlast 1 ms, Pixelraster-Delta 1 ms, Binärantwort 2 ms. Zeiger bis Strich: lokal 0 ms, gebündelt 1 ms. Zeiger bis Bild (vsync): 1920x1080 lokal 38–48 ms (p95 51–54), gebündelt 48 ms (p95 84); 1280x800 beide 33 ms (p95 35–67); durch die Software-Grafik der VM nicht auf echte Hardware übertragbar. Speicher mit Messansicht: 661 MB RSS, 425 MB PSS (vintagecoder 186, WebKitWebProcess 440, WebKitNetworkProcess 50 MB RSS). Ein offener Willkommensassistent von Ubuntu ließ die erste Messung hängen (schwarzes Fenster); danach fehlerfrei. Rohdaten und Bilder: `Y:\austausch-g1\linux\` (auf dem Mac nicht erreichbar) |

## Bildtakt (Frage e, geklärt mit der Sonde `frameProbe`)
Die Messung „Zeiger bis Bild“ wartet absichtlich **zwei** Bilder (Ereignis, Zeichnen, nächstes Bild). Der feste Wert ist daher zweimal das Bildintervall der Anzeige, keine Drosselung durch WebKit oder WebView2:

| System | rAF-Intervall (Sonde) | Anzeige | Wert „Zeiger bis Bild“ |
|---|---|---|---|
| macOS arm64 | 10 ms (100 Hz) | Bildschirm 3440x1440 mit 100 Hz | 20 ms |
| Windows 11 | 31,25 ms (32 Hz) | Remotesitzung: „Microsoft Remote Display Adapter“ 3440x1440 mit 32 Hz (die echte Grafik mit 59 Hz liegt daneben); sichtbar, mit Fokus | 62,5 ms |
| Linux (VM, Software-Rendering) | nicht gesondert gemessen | llvmpipe | 33–48 ms |

Folgerung für das Zeichnen (ab E3): Die Bildrate folgt der Anzeige des Fensters; unter Windows per Remotesitzung ist sie ein Artefakt der Sitzung. Eine Gegenprobe am physischen Monitor steht aus (nur lokal am Rechner möglich). Das Zeichnen selbst muss nicht auf Bilder warten: der lokale Vorschaustrich braucht 0 ms (`pointer-local-sync`), die Bestätigung aus Rust 0–4 ms.
