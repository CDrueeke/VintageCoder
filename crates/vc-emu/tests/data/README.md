# Aufgezeichnete Ultimate-Pakete

Echte UDP-Pakete einer Ultimate 64 II (Firmware 3.14d, LAN, Unicast, PAL), aufgezeichnet am 2026-11-08 mit
`vcc streams probe --host … --record ORDNER` (dieser Empfänger reicht Nutzpakete weiter, nicht die 2-Byte-Startpakete).
Dateiformat: Folge von Paketen, je `u16` Länge (little-endian) und die Paketbytes. Leser: `read_packets` in `crates/vc-emu/tests/streams.rs`.

| Datei | Inhalt |
|---|---|
| `video.bin` | 72 Pakete ab dem ersten Paket eines Bilds (`line` = 0); die ersten 68 sind ein vollständiges Bild (das letzte mit Bit 15) |
| `audio.bin` | 49 Pakete (rund 196 ms) |
| `debug.bin` | 61 Pakete „6510 Only“; ein Paket ist kürzer als 1444 Byte (716 Byte, das erste nach dem Start) |

Gemessen: Jeder Strom beginnt mit einem Paket von 2 Byte (Folgenummer 0, ohne Nutzdaten). Im Debug-Strom kommen neben vollen Paketen
regelmäßig kürzere (868 Byte etwa alle 55 Pakete); die Folgenummern bleiben lückenlos.
