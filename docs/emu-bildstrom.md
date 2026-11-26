# Bildstrom des Emulators (Kernarbeit K1)

Stand: K1. Für die Oberflächen-Sitzung (Tauri, Etappe E6) und alle, die das Emulatorbild fortlaufend brauchen. Code: `crates/vc-emu/src/framestream.rs`, Fake-Emulator `crates/vc-emu/src/fake.rs`, Tests `crates/vc-emu/tests/framestream.rs` (ohne Emulator) und `tests/vice_stream.rs` (echtes VICE, `VC_EMU_TESTS=1`).

## Was es ist

Ein Arbeitsfaden holt im festen Takt über `Backend::grab_frame` das Bild und liefert es als `StreamFrame`. Statt Einzelabfrage vom Webview (Spike: 16 Bilder/s) gibt es einen Takt im Kern: **49 bis 50 Bilder/s bei PAL**, mindestens 25 stabil. Der Strom ist allgemein: er arbeitet mit jedem Adapter, dessen `can_grab_frame()` wahr ist (VICE heute; weitere Emulatoren, sobald ihr Adapter `grab_frame` kann). Er teilt sich den Emulator mit Debugger und Rest (`SharedBackend` = `Arc<Mutex<Box<dyn Backend>>>`).

**Auslassen statt Aufstauen:** dauert ein Abruf länger als ein Takt oder holt der Verbraucher zu langsam ab, werden Bilder verworfen und gezählt (`dropped`). Es gibt keine Warteschlange und keinen Nachholstoß.

## Aufruf

```rust
use vc_emu::framestream::{share, FrameStream, StreamConfig, NextError};

let be = share(vc_emu::backend_for("vice-binmon", &launch)?);   // einmal teilen, dann überall dieselbe Kopie klonen
let stream = FrameStream::start(be.clone(), StreamConfig { rate: 50.0, deliver_unchanged: false })?;
loop {
    match stream.next(Duration::from_millis(200)) {
        Ok(f)  => paint(&f.data, f.width, f.height, f.stride),      // RGBA
        Err(NextError::Timeout) => continue,                          // Strom läuft weiter, nur nichts Neues
        Err(NextError::Ended(err)) => break,                          // Some(StreamError) bei Fehler, None nach stop()
    }
}
let stats = stream.stop();   // wartet auf den Faden; der Emulator läuft weiter
```

Mit Rückruf (für Tauri: `emit` je Bild): `FrameStream::start_with(be, cfg, |frame| …, |ende: Option<StreamError>| …)`; der Rückruf läuft auf dem Arbeitsfaden, ein langsamer Rückruf verzögert den nächsten Takt (verpasste Takte werden ausgelassen). Weitere Aufrufe: `try_latest()` (ohne Warten), `stats()`, `is_running()`. Wegwerfen des `FrameStream` beendet ihn ebenfalls (`Drop` wartet auf den Faden).

Der Emulator bleibt beim Beenden des Stroms am Leben; **zurückbleibende Emulatoren** verhindert weiter nur `Backend::stop()` (VICE-Prozess samt Kindprozessen) beim Beenden der Anwendung.

## Konfiguration

| Feld | Bedeutung |
|---|---|
| `rate` | Zielrate in Bildern pro Sekunde, `1.0 ..= 100.0` (PAL 50, NTSC 60); sonst Fehler `diag-emu-stream-bad-rate` |
| `deliver_unchanged` | `false` (Standard): Bilder ohne Änderung gegenüber dem zuletzt gelieferten (stehende Maschine, stehender Bildschirm) werden nicht geliefert, nur in `stats.unchanged` gezählt; das erste Bild kommt immer. `true`: alle Bilder, `changed` sagt es dann |

## Datenformat `StreamFrame`

| Feld | Inhalt |
|---|---|
| `width`, `height` | Pixel (VICE x64sc PAL: 504 × 312; je nach Rechner/Bildquelle anders, bei jedem Bild prüfen) |
| `format` | `PixelFormat::Rgba8`: R, G, B, A je 8 Bit, A = 255 |
| `stride` | Bytes je Zeile (`width * 4`, keine Füllbytes; die Oberfläche soll `stride` trotzdem verwenden) |
| `data` | `stride * height` Bytes (PAL: 630 KB) |
| `seq` | Nummer des Taktes (ab 1). **Lücken zeigen ausgelassene Takte**; bei `deliver_unchanged: false` entstehen zusätzlich Lücken durch nicht gelieferte unveränderte Bilder |
| `timestamp` | Zeit seit Strombeginn, zu der der Abruf fertig war (Dauer, monoton) |
| `dropped` | Seit dem zuletzt gelieferten Bild **ausgelassene Bilder** wegen zu langsamen Abrufs oder Verbrauchers (0 = lückenlos); Nummernlücken mit `deliver_unchanged: true` sind genau `dropped` |
| `changed` | Inhalt anders als beim zuletzt gelieferten Bild (erstes Bild: `true`) |

Für Tauri: das Bild als `tauri::ipc::Response` mit 8-Byte-Kopf (Breite, Höhe als u32 little endian) wie im Spike, `seq`, `dropped` im Ereignis daneben; `Uint8ClampedArray` → `ImageData` → `putImageData`.

## Fehler (Katalogschlüssel)

`StreamError { key, args, message }`: `key` und `args` maschinenlesbar, `message` in der Sprache beim Entstehen (`t!`); die Oberfläche kann mit `key` und `args` selbst übersetzen. Schlüssel in `crates/vc-i18n/catalog/{de,en}/emu.ftl`:

| Schlüssel | Parameter | Wann |
|---|---|---|
| `diag-emu-stream-bad-rate` | `rate`, `min`, `max` | `start` mit Rate außerhalb des Bereichs |
| `diag-emu-stream-no-frames` | – | der Adapter kann kein Bild (`can_grab_frame()` falsch) |
| `diag-emu-stream-thread` | `detail` | Arbeitsfaden ließ sich nicht starten |
| `diag-emu-stream-grab-failed` | `detail` | Strom endet: **drei** Abrufe hintereinander schlugen fehl (ein einzelner Fehler wird überstanden und in `stats.errors` gezählt). Beendet auch den Strom, wenn der Emulator-Prozess weg ist |
| `diag-emu-vice-resume-lost` | – | aus `grab_frame`/`cont` (VICE): VICE hat das Fortsetzen auch nach Wiederholung nicht bestätigt |

## Wichtig für die Oberfläche

- **Haltepunkte und Schritte:** hält der Emulator an (Haltepunkt, `pause`, Einzelschritt), liefert der Strom weiter Abrufe, setzt die Maschine aber **nicht** fort (ein stehender Emulator bleibt stehen). Mit `deliver_unchanged: false` kommen dann keine Bilder; die Oberfläche zeigt das letzte.
- **Gemeinsamer Zugriff:** jeder andere Befehl (`registers`, `read_mem`, …) nimmt dieselbe Sperre; bei VICE hält **jeder** Monitorbefehl die Maschine an. Ein Befehl `registers()` lässt VICE angehalten (VICE-Verhalten); wer die Maschine danach weiterlaufen lassen will, ruft `cont()`. Der Strom setzt eine von ihm selbst angehaltene Maschine immer fort.
- **Tempo:** siehe unten (Tempoausgleich). Wer `Vice::set_tempo_hold(Duration::ZERO)` setzt, bekommt schnellere Abrufe, aber der C64 läuft dann mit 130 bis 160 % Tempo.
- Andere Rechner (C128 mit VDC, Plus/4, VC-20): `frame_sources()` und `set_frame_source()` wirken auch im Strom; die Größe steht in jedem Bild.

## Anlaufstörung (geklärt)

Beobachtung im Spike: VICE „startet nach `autostart` angehalten und verschluckt das erste `cont`“ (schwarzes Bild, PC im Reset-Code). Befund:

1. **Nach dem Verbinden steht VICE im Monitor** (Ereignis 0x62 „angehalten“ auf den ersten Befehl). Der Autostart-Befehl (0xDD) antwortet sofort; **das Fortsetzen meldet VICE erst kurz danach selbst** (Ereignis 0x63). In dieser Lücke sagte `Vice::state()` „angehalten“.
2. `Vice::cont()` setzte den Zustand **sofort** auf „läuft“, ohne VICEs Bestätigung. Traf ein Befehl VICE in der Anlaufphase, ließ er die Maschine im Monitor stehen; der eigene Merker sagte weiter „läuft“, und jede spätere Prüfung („lief schon“, im Strom wie im Spike `grab_frame` und `emu_regs`) übersprang das Fortsetzen. Das erste `cont` war also nicht „verschluckt“, sondern der **Zustandsmerker log**. Die Umgehung des Spikes (`cont` beim Registerabfragen) traf das Symptom.
3. **Behoben in `vc-emu`** (nicht nur umgangen): der Zustand der Maschine ändert sich jetzt nur durch die Ereignisse von VICE (0x62/0x63); `cont()` wartet auf die Bestätigung (0x63), wiederholt 0xAA einmal und meldet sonst `diag-emu-vice-resume-lost`; `Vice::launch` wartet nach dem Autostart auf das Fortsetzen (höchstens 3 s, dann eigenes `cont`). Ergebnis: nach `launch` mit Programm ist `state()` verlässlich „läuft“, ein `cont` von außen ist nicht nötig.
4. **Nicht reproduziert:** das schwarze Bild selbst ließ sich in 10 Läufen (Wartezeit 0 bis 2 s, mit und ohne `cont`, 16-Byte-Testprogramm) nicht auslösen; die Spike-Programmgröße (16 KB, längerer Autostart) war nicht nachstellbar. Die Ursachenkette 1 bis 2 ist aus dem Monitorverlauf (`VC_VICE_TRACE=1`) und dem Code belegt, ein Beweis durch Fehlerauslösung steht aus. Der Test `vice_stream` startet dreimal hintereinander mit sofort laufendem Strom und verlangt, dass das Programm anläuft.

Wer ohne Programm startet (`launch("")`), bekommt weiter eine angehaltene Maschine und muss `cont()` rufen (nichts zu laden, VICE wartet im Monitor).

## Tempoausgleich (VICE-Verhalten, wichtig)

Jedes Anhalten und Fortsetzen über den Binärmonitor lässt VICE das laufende Bild **ohne Wartezeit** zu Ende rechnen (der Tempobegrenzer setzt zurück): je Zyklus rund 11 bis 13 ms zusätzliche Emulationszeit. Gemessen (Jiffy-Zähler, VICE 3.10 x64sc, macOS, M4), reiner Zyklus ohne Bildabruf: Takt 20 ms ohne Haltezeit 160 %, mit 8 ms 111 %, mit **10 ms 101 %**, mit 12 ms 91 %; Takt 100 ms ohne Haltezeit 111 %, 500 ms 102 %, 1 s 101 %. Ohne Gegenmaßnahme liefe der Rechner mit dem alten Einzelabruf (und jedem naiven Strom) bei 50 Abrufen/s mit **bis 165 %**, Musik und Spiele zu schnell (Spike: 16 Abrufe/s, etwa 105 bis 110 %).

Gegenmaßnahme in `Vice::grab_frame`: nach dem Abruf bleibt die Maschine bis mindestens `TEMPO_HOLD` (10 ms, ab dem Eintreffen des Bildes) angehalten, dann `cont`. Das gleicht den Gewinn aus (100 ± 3 %), und aufeinanderfolgende Bilder liegen im Mittel ein Rechnerbild auseinander. Kosten: ein Einzelabruf dauert mindestens 10 ms länger. Stufen der Messung: Haltezeit 0/3/6/10 ms bei 50 Abrufen/s: Tempo 163/146/127/**102 %**. Einstellbar mit `Vice::set_tempo_hold`; für Messungen `VC_VICE_HOLD_MS`.

## Messbericht (VICE 3.10 x64sc PAL, Apple M4, macOS 27.2.1, Release-Bau)

Aufbau: `examples/bench_stream.rs` (Testprogramm: Rahmenfarbe wechselt ständig, damit jedes Bild neu ist), 15 s je Lauf, Verbraucher in demselben Prozess, **ohne** Weiterleitung an ein Webview (die UI-Sitzung misst ihren Teil getrennt).

| Ziel | geliefert | ausgelassen | Abstand Mittel / Streuung / p95 / max | Verzögerung Abruf→Verbraucher | Abruf (VICE angehalten) | Tempo des Rechners | CPU VintageCoder / VICE | Speicher VintageCoder / VICE |
|---|---|---|---|---|---|---|---|---|
| 25 B/s | 25,0 B/s | 0 | 40,0 / 10,0 / 59 / 87 ms | p50 0,02, p95 0,13 ms | 25,0 ms | 101 % | 2,0 % / 26 % | 5 MB / 126 MB |
| 50 B/s (Lauf 1) | 49,4 B/s | 9 | 20,3 / 2,9 / 21,9 / 58 ms | p50 0,02, p95 0,18 ms | 19,7 ms | 100 % | 3,4 % / 29 % | 6 MB / 123 MB |
| 50 B/s (Lauf 2) | 49,2 B/s | 12 | 20,3 / 4,0 / 23,3 / 58 ms | p50 0,02, p95 0,67 ms | 19,7 ms | 102 % | 2,9 % / 26 % | 6 MB / 127 MB |
| 1 B/s (Grundlast) | 1,1 B/s | 0 | – | – | – | 99 % | 0,2 % / 30,5 % | 12 MB / 162 MB |

CPU in Prozent eines Kerns. VICE verbraucht auch ohne Strom rund 30 %; der Strom kostet ihn nichts Messbares. Speicher je Bild: VICE liefert 157 KB Farbindizes über die lokale Verbindung; daraus werden 630 KB RGBA (504 × 312 × 4), die genau einmal zugewiesen und an den Verbraucher übergeben werden (keine Kopie im Strom; Pufferwiederverwendung steht aus). Die Palette wird gemerkt (alle 2 s neu), das spart einen Monitorbefehl je Bild.

**Vergleich mit dem Spike:** der Spike hatte 16,2 Bilder/s (60 ms Sollabstand plus Webview-Einzelabfrage) und Rechnertempo etwa 105 bis 110 %; jetzt 49 bis 50 Bilder/s im Kern bei richtigem Tempo. Das Webview-Ende (IPC, `putImageData`, 630 KB je Bild = 31 MB/s bei 50 B/s) misst die Oberfläche in E6.

Einzelne Ausreißer: der Abruf läuft bis 40 ms (VICE bearbeitet Monitorbefehle in der Bildschleife, ein Befehl wartet bis zu einem Bild); das zeigt sich als Streuung des Abstands (3 bis 4 ms bei 50, 10 ms bei 25 Bildern/s) und als 9 bis 12 ausgelassene Bilder je 750. Die Oberfläche sollte Bilder bei `dropped > 0` einfach zeichnen (kein Nachholen).

## Grenzen und offene Punkte

- **Nur VICE gemessen.** atari800, Altirra, MEGA65 haben noch kein `grab_frame` im Adapter (`can_grab_frame()` falsch → `diag-emu-stream-no-frames`); der Strom ist dafür fertig, der Adapter fehlt. Die Ultimate 64 liefert ihr Bild über das UDP-Modul `streams` (`FrameAssembler`), nicht über `Backend`; eine Anbindung als `grab_frame` (neuestes vollständiges Bild) wäre klein, ist hier nicht gebaut.
- NTSC (60 Hz): nicht gemessen; Tempoausgleich-Konstante gilt für PAL (Ausgleichs-Gewinn je Zyklus hängt an der Bilddauer).
- Linux und Windows nicht gemessen; VICE unter Windows 3.9 hat den bekannten Absturz beim Einzelschritt (nicht Strom).
- Die Konstante `TEMPO_HOLD` gilt für VICE 3.10 unter macOS; bei anderen Versionen mit `examples/probe_cycle.rs` nachmessen.
