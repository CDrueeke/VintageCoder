// SPDX-License-Identifier: GPL-3.0-or-later
// Einziger Weg der Oberfläche in den Rust-Kern. Drei Wege, derselbe Befehlssatz:
//  1. Tauri (`invoke`) im echten Programm,
//  2. Prüfbrücke (WebSocket auf localhost, nur Entwicklung; Adresse über `?bridge=<port>`), damit die echte Oberfläche in Playwright/Browser gegen den echten Kern läuft,
//  3. kein Kern: jede Anfrage scheitert mit `keine-verbindung` (nur Entwurf im Browser ohne Kern).
// Die Typen sind von Hand geschrieben (tauri-specta ist nur als Vorabversion verfügbar, siehe docs/ui-aufbau.md); ein Rusttest vergleicht die Namen dieser Listen mit dem Kern.

/** Befehle des Kerns (Quelle: `crates/vc-shell/src/cmds.rs`, `BEFEHLE`) */
export const COMMANDS = [
  "i18n_ftl", "dom_info", "zahlen_werte", "lang_get", "lang_set", "commands_table", "registry_info", "features_get", "menu_context", "recent_list", "project_pick", "project_open", "project_close",
  "doc_list", "ping", "raster_apply", "raster_reset", "app_info", "app_quit",
  "file_read", "file_write", "file_create", "dir_create", "file_rename", "dir_rename", "file_delete", "syntax_color", "fold_ranges",
  "tree_move", "tree_undo", "tree_reload", "abs_path", "file_reveal", "file_external", "file_stats", "file_pick", "tree_can_trash",
  "doc_open", "doc_change", "doc_close", "lsp_hover", "lsp_complete", "lsp_definition", "lsp_outline", "lsp_fixes", "format_lines", "search_start", "search_cancel", "text_hover", "char_names",
  "build_start", "run_start", "run_stop", "build_info", "run_targets", "run_target_set", "build_cycles",
  "tests_start", "tests_stop", "tests_state",
  "disk_pick", "disk_open", "disk_view", "disk_apply", "disk_close", "disk_import", "disk_add_pick", "disk_save_pick", "disk_new", "palette_open", "kacheln_open", "karten_open", "bildschirm_open", "tracker_open", "audio_poll", "help_topics", "help_search", "help_open", "help_image", "wiz_stand", "wiz_start", "wiz_set", "wiz_system", "wiz_apply", "wiz_undo", "wiz_later", "wiz_news", "wiz_end", "info_about", "info_update_state", "info_update_check", "info_update_later", "info_update_skip", "info_feedback", "info_mail", "info_open", "settings_get", "args_info", "args_migrate", "settings_op", "ziele_info", "ziele_probe", "roms_info", "roms_open", "md_render", "md_edit", "md_url_open", "chips_info", "emu_pakete", "emu_install", "roms_pick", "roms_import_plan", "roms_import_run", "roms_pin", "settings_save", "settings_export", "settings_import_pick", "settings_import_apply", "settings_reset", "settings_reveal", "hex_list", "hex_page", "hex_find", "speicher_info", "debug_start", "debug_start_test", "debug_state", "debug_cmd", "debug_frame", "debug_stop", "song_quelle_pick", "song_import_run", "pixel_open", "pixel_view", "pixel_apply", "pixel_close", "export_info", "export_preview", "export_run", "png_pick", "media_profiles", "media_new", "recent_pin", "recent_remove", "recent_reveal", "examples_list", "tools_list", "vcprj_pick", "copy_suggest", "copy_check", "copy_plan", "copy_run",
  "drop_plan", "drop_copy", "include_resolve", "start_args", "newproj_info", "newproj_system", "newproj_preview", "newproj_create", "dir_pick",
  "projekt_info", "projekt_set", "basic_get", "basic_set", "turbo_state", "turbo_pick", "turbo_check", "turbo_provide", "turbo_reduce", "turbo_set",
  "session_get", "session_set", "restore_tabs_set", "scratch_get", "scratch_set", "scratch_pref",
  "basic_convert", "basic_info", "basic_codes",
] as const;
export type Command = (typeof COMMANDS)[number];

/** Ereignisse des Kerns (Quelle: `crates/vc-shell/src/events.rs`, `EREIGNISSE`) */
export const EVENTS = ["app:oeffnen", "lang:changed", "menu:cmd", "project:changed", "lsp:diagnostics", "search:hits", "search:done", "build:started", "build:done", "run:started", "run:failed", "run:stopped", "run:ended", "tests:update"] as const;
export type CoreEvent = (typeof EVENTS)[number];

export type LangCode = "de" | "en" | "xx";
export type LangChoice = "auto" | "de" | "en" | "xx";

export interface Failure { code: string; args: Record<string, unknown> }
export class CoreError extends Error {
  code: string;
  args: Record<string, unknown>;
  constructor(f: Failure) { super(f.code); this.code = f.code; this.args = f.args ?? {}; }
}

export interface FtlSources { lang: LangCode; de: string; en: string | null }
export interface LangState { choice: LangChoice; lang: LangCode; available: LangCode[] }
/** Marke einer Datei: Zeile (ab 1) und Zeilentext zum Wiederfinden nach Änderungen */
export interface MarkEntry { line: number; text: string }
export interface FileMarks { breakpoints: MarkEntry[]; bookmarks: MarkEntry[] }
export interface CommandInfo { id: string; menu: string; submenu: string; keys: string; keysDisplay: string; needs: "none" | "project" | "editor"; /** Paket, aus dem ein Befehl kommt (fest eingebaute: leer) */ source?: string | null; /** Titel dynamischer Befehle (Erweiterungen) */ title?: string | null }
/** Auskunft der Registry (Quellen, Pakete, Befehle, Meldungen) */
export interface RegistryInfo { sources: { kind: string; origin: string; path: string }[]; packages: { id: string; version: string; name: string; kind: string }[]; commands: { id: string; origin: string; feature: string | null }[]; problems: { origin: string; file: string; text: string }[]; hasExtensions: boolean }
/** Release-Auswahl der Editoren */
export interface FeaturesInfo { all: boolean; studios: { id: string; on: boolean }[] }
export interface RecentProject { path: string; pretty: string; name: string; pinned: boolean; missing: boolean; systems: [string, string][]; langs: string[]; emulator?: string | null; ago: string; image?: string | null }
export interface DiskFile { i: number; name: string; kind: string; blocks: number; start: string; locked: boolean; auto: boolean; size: string | null; bad: boolean; checked: boolean; basic: boolean }
export interface DiskHexRow { off: number; bytes: [number, boolean][]; text: string }
/** Anzeige-DTO des Disk-Editors (`vc-shell::disk::ansicht`); Zellen: −1 frei, −2 Kopf/BAM, −3 Verzeichnis, −4 verwaist, −5 Konflikt, ≥0 Datei */
/** Antwort von `zahlen_werte` (Zahlenwerkzeuge): bei `ok` alle Schreibweisen, sonst `error` */
export interface ZahlenWerte { ok: boolean; text: string; error?: string; value?: number; dec?: string; hex?: string; bin?: string; signed?: number; bits?: number; lo?: number; hi?: number; char?: string | null; bitList?: number[] }
export interface DiskView {
  format: string; atari: boolean; title: string; path: string | null; dirty: boolean; geometry: string; size: string | null;
  files: DiskFile[]; cells: number[][]; free: number; used: number; total: number; interleave: number | null; problems: string[]; report: string[] | null;
  selFile: number | null; chainOn: boolean; chain: [number, number][]; chainPos: [number, number] | null;
  cur: [number, number]; curText: string; title2: string; owner: string | null; pointer: string | null; hex: DiskHexRow[] | null;
  next: { addr: [number, number]; text: string } | null; bytesInSector: number | null; canBack: boolean; checkedCount: number; canUndo: boolean; canRedo: boolean; canOrder: boolean;
  note: { ok: boolean; text: string } | null; fs: boolean; opts: { dir: string; basic: boolean; raw: boolean; asm: boolean; dirBad: boolean }; importResult: { ok: boolean; text: string }[] | null;
}
/** Anzeige-DTO des Sprite-Editors (`vc-shell::pixel`); `pixels` ist das gewählte Bild zeilenweise, `valueColors` die Farbe je Pixelwert (null = durchsichtig) */
export interface PixelView {
  rel: string; kind: "sprite" | "pmg" | "charset" | string; profile: string; target: string; title: string; width: number; height: number; bpp: number; aspect: number;
  frames: number; fi: number; frameNames: string[]; fixedFrames: boolean; multicolor: boolean; pixels: number[]; pv: number; tool: string;
  values: { v: number; name: string; color: string | null; role: string | null }[]; valueColors: (string | null)[]; slots: { key: string; label: string; index: number; color: string | null }[];
  palette: { i: number; color: string }[]; norm: string; normActive: string; projectNorm: string; normChoice: boolean;
  anims: { name: string; mode: "loop" | "pingpong" | "once"; steps: [number, number][]; expanded: [number, number][] }[] | null; animError: string | null; selAnim: number;
  thumbs: { w: number; aspect: number; mc: boolean; pixels: number[]; colors: (string | null)[] }[]; modes: { label: string; hint: string }[] | null;
  flags: { x2: boolean; y2: boolean; bg: boolean } | null; pmg: { sizes: number[]; prior: number; priorChoices: number[]; sizeChoices: number[] } | null; scene: { sky: string | null; ground: string | null }; blockBytes: number;
  sel: [number, number, number, number] | null; hasClip: boolean; canUndo: boolean; canRedo: boolean; dirty: boolean; status: string; note: { ok: boolean; text: string } | null;
  /** nur Zeichensätze: Vorlagen, aktuelle Vorlage, Herkunft, je Zeichen Abweichung von der Vorlage (null ohne Vorlage) */
  charset: { vorlagen: { ref: string; label: string; missing: boolean }[]; vorlage: string; herkunft: { label: string; lines: string[] } | null; geaendert: boolean[] | null } | null;
}
/** Anzeige-DTO des Trackers (`vc-shell::tracker`) */
export interface TrackerView {
  rel: string; kind: "song" | "play"; title: string; name: string; target: string; chip: string; pokey: boolean; channels: number; rows: number; hz: number; pattern: number; patternCount: number; grid: string[][];
  cursor: [number, number]; sel: [number, number, number, number] | null; order: number[]; ordSel: number; loop: number; tempo: number; volume: number; octave: number; instrument: number; step: number;
  instruments: { name: string; used: boolean; wave?: string; pw?: number; a?: number; d?: number; s?: number; r?: number; dist?: string; env?: number[]; loop?: number }[]; insSel: number; maxInstruments: number;
  mute: boolean[]; solo: boolean[]; hasClip: boolean; canUndo: boolean; canRedo: boolean; dirty: boolean; status: string; audition: null | { note: number; instrument: number }; spielt: boolean; spielModus: null | "song" | "pattern";
}
/** Quelle für Konvertieren/Import (`song_quelle_pick`): Datei, Art, Namensvorschlag, bei SID-Dateien Beschreibung, Zahl der Subtunes und Startsong; `error` bei unlesbarer Datei */
export interface SongQuelle { path?: string; file: string; art?: string; name?: string; sid?: { info: string; songs: number; start: number } | null; error?: string }
/** Entwurf des Einrichtungsassistenten (`wiz_*`): Schritte, Einstellungen, Darstellung, Beispiele, Änderungsliste */
export interface WizData {
  mode: string; firstRun: boolean; applied: boolean; steps: { id: string; title: string; group: string }[]; settings: Record<string, any>; view: { scheme: string; zoom: number; font: number }; // eslint-disable-line @typescript-eslint/no-explicit-any
  examples: { copy: boolean; excluded: string[]; source: string | null; dest: string; items: { id: string; system: string; name: string }[]; destEffective: string }; projects: string;
  changes: { key: string; area: string; kind: "new" | "changed" | "removed" | "same"; label: string; old: string; new: string; selected: boolean; sensitive: boolean; note: string; topic: string }[]; problems: { topic: string; text: string }[]; later: string[];
}
/** Stand der Update-Anzeige (`info_update_*`): letzte Prüfung, Hinweis für die Statusleiste, gemerkte Bewertung */
export interface UpdateStand { lastCheck: string | null; lastResult: string; badge: string | null; enabled: boolean; source: string; eval: null | { latest: string; yanked: boolean; update: null | { version: string; url: string; notes: string; formatWarning: boolean; published: string } } }
export interface Danksagung { id: string; name: string; license: string; links: { url: string; label: string }[] }
export interface AboutInfo { name: string; version: string; license: string; website: string; repository: string; notices: string; credits: Danksagung[]; update: UpdateStand }
export interface FeedbackMail { subject: string; body: string; bodyFull: string; url: string; dropped: string[]; tooLong: boolean; clipboard: string }
/** Einstellungsdatei der IDE (`settings_get`): Werte, Ort, persönlicher Ordner, wirksamer Projektordner; BASIC-Dialekte je System */
export interface SettingsReply { settings: Record<string, any>; file: string | null; home: string | null; effectiveProjects: string; mac: boolean; dialects?: { id: string; name: string; dialect: string | null }[] } // eslint-disable-line @typescript-eslint/no-explicit-any
/** Parametereditor (`args_info`): Katalog der Sorte, Platzhalter, Prüfmeldungen, fertige Kommandozeile */
export interface ArgParam { id: string; group: string; type: "switch" | "choice" | "text" | "number" | "file"; flag: string; off_flag?: string | null; label: string; hint: string; choices: { value: string; label: string }[]; default?: string | null; example?: string | null; min?: number | null; max?: number | null; conflicts: string[]; requires: string[] }
export interface ArgsInfo { catalog: null | { program: string; label: string; verified: string; groups: { id: string; label: string }[]; reserved: string[]; params: ArgParam[] }; placeholders: { name: string; label: string; hint: string; example: string | null }[]; issues: { level: "error" | "warn" | "info"; text: string }[]; preview: string; changed: number; migratable: number }
/** ROM-Stand (`roms_info`): je System und Rolle Status, Datei und Prüfsummen; durchsuchte Orte */
/** Markdown-Vorschau (Kern `markdown.rs`): Blockbaum mit Quellzeilen */
export type MdInline = { t: "text" | "code"; s: string } | { t: "em" | "strong" | "strike"; k: MdInline[] } | { t: "link"; title: string; to: MdZiel; k: MdInline[] } | { t: "image"; alt: string; to: MdZiel; img: string | null } | { t: "soft" | "br" };
export type MdZiel = { kind: "web"; url: string } | { kind: "anchor"; anchor: string } | { kind: "file"; rel: string; line: number | null } | { kind: "blocked"; why: string };
export type MdNode = { b: "heading"; level: number; anchor: string; k: MdInline[]; lines: [number, number] } | { b: "para"; k: MdInline[]; lines: [number, number] } | { b: "quote"; nodes: MdNode[]; lines: [number, number] }
  | { b: "list"; start: number | null; items: { task: boolean | null; nodes: MdNode[] }[]; lines: [number, number] } | { b: "code"; lang: string; text: string; lines: [number, number] }
  | { b: "table"; align: string[]; head: MdInline[][]; rows: MdInline[][][]; lines: [number, number] } | { b: "rule"; lines: [number, number] } | { b: "html"; text: string; lines: [number, number] };
export interface MdRender { nodes: MdNode[]; total: number; front: [number, number] | null; outline: { level: number; text: string; line: number; anchor: string }[] }
export interface EmuPakete { known: boolean; id?: string; name?: string; homepage?: string; license?: string; packages?: { version: string; sizeMb: number; host: string; installed: boolean; to: string | null }[]; manual?: string | null; installed?: string[] }
export interface RomImportItem { i: number; name: string; src: string; system: string | null; role: string | null; label: string | null; size: number; crc32: string | null; action: "copy" | "present" | "skip"; reason: string; dest: string | null; source: string | null }
export interface RomsInfo { systems: { id: string; label: string; missingRequired: string[]; roles: { id: string; label: string; required: boolean; file: string; status: "recognized" | "unknown" | "replacement" | "missing" | "unusable"; statusLabel: string; path: string | null; size: number | null; crc32: string | null; sha1: string | null; revision: string | null; place: string | null; expected: number[]; others: string[] }[] }[]; searched: { stage: string; path: string; exists: boolean; files: number }[]; useForeign: boolean }
/** Eine Seite der Hex-Ansicht (`hex_page`, Kern: `daten.rs`) */
export interface HexPage { rel: string; total: number; loaded: number; truncated: boolean; pages: number; page: number; pageSize: number; width: number; rows: { off: number; bytes: number[]; text: string }[]; load: number | null }
/** Speicherkarte (`speicher_info`): je Seite [Art, Kategorie, belegte Bytes] */
export interface SpeicherInfo {
  built: boolean; target?: string; pages?: [string, string, number][]; sel?: number; legend?: { key: string; label: string }[];
  banking?: null | { register: { name: string; addr: number }; presets: { id: string; label: string; note: string }[]; selected: string | null };
  detail?: { page: number; from: number; to: number; kind: string; category: string; used: number; symbols: [string, number][]; sources: [string, number, number][] };
  checks?: { level: "ok" | "info" | "warn" | "err"; text: string }[]; program?: number; free?: number; segments?: { start: number; len: number; bank: number; name: string }[];
}
/** Zustand der Debug-Sitzung (`debug_state`, Kern: `dbgsitzung.rs`) */
export interface DebugDis { addr: number; bytes: number[]; text: string; illegal: boolean; cycles: number; extra: boolean; label: string | null; line: { file: string; line: number } | null; bp: boolean }
export interface DebugPoint { id: number; kind: "exec" | "read" | "write" | "access"; start: number; end: number; cond: string | null; ignore: number; hits: number }
export interface DebugState {
  active: boolean; starting?: boolean; ended?: boolean; running?: boolean; haveRegs?: boolean; backend?: string; canPause?: boolean; stepBlock?: string | null; error?: string | null; condError?: string | null;
  caps?: { registers: boolean; step: boolean; breakpoints: boolean; watch: boolean };
  regs?: { pc: number; a: number; x: number; y: number; sp: number; flags: number; extra: { name: string; value: number }[]; prev: number[] | null };
  ort?: { file: string; line: number } | null; disasm?: DebugDis[]; stack?: number[]; points?: DebugPoint[]; mem?: { addr: number; bytes: number[]; prev: number[] };
  watch?: { expr: string; value?: number; error?: string }[]; chips?: { addr: number; bytes: number[] }[];
  detected?: null | { error?: string; system?: string; mode?: string; screen?: number; charset?: number | null; charsetMulticolor?: boolean; romOff?: number | null; supported?: boolean; sprites?: number; notes?: string[] };
  frame?: { can: boolean; seq: number; error: string | null; sources: string[]; canText: boolean }; meldungen?: { ok: boolean; text: string }[]; test?: boolean;
}
/** Zustand der Wiedergabe (`audio_poll`): Position, Spitzenpegel je Stimme, klingende Noten */
export interface AudioStand { playing: boolean; ord: number; row: number; peaks: number[]; master: number; notes: (number | null)[]; error: string | null }
/** Anzeige-DTO des Bildschirm-Editors (`vc-shell::bildschirm`) */
export interface BildschirmView {
  rel: string; kind: "screen"; title: string; target: string; w: number; h: number; farbRam: boolean; atari: boolean; img: string; imgW: number; imgH: number; rand: number; codes: number[]; farben: number[];
  tafel: { img: string; n: number }; tool: "char" | "text" | "fill" | "rect" | "pick" | "erase"; brush: { code: number; color: number }; cursor: [number, number]; cursorCode: number;
  roles: { bg: number; border: number; fg: number }; palette: { i: number; color: string }[]; charset: string | null; charsets: string[]; norm: string; projectNorm: string;
  zerlegen: null | { ok: boolean; text: string }; zerlegt: null | { set: string; bank: string; info: string }; canUndo: boolean; canRedo: boolean; dirty: boolean; status: string;
}
/** Anzeige-DTO des Karten-Editors (`vc-shell::karten`): Bank mit mehreren Karten (Screens), Kachelraster, Zeichentafel */
export interface KartenView {
  rel: string; kind: "bank"; title: string; target: string; norm: string; projectNorm: string; layers: { tiles: boolean; cells: boolean }; cur: number; cols: number; rows: number; tw: number; th: number; rand: number;
  screens: { i: number; name: string; w: number; h: number; img: string }[];
  map: { img: string; w: number; h: number; name: string; cellsW: number; cellsH: number; origin: [number, number]; colors: Record<string, number>; patches: number; grid: number[][] } | null; set: number | null;
  mode: "tile" | "cell"; tool: string; filled: boolean; tools: { id: string; label: string }[];
  tiles: { i: number; name: string; col: number; props: number; img: string }[];
  brush: { tile: number; recent: number[]; cell: { code: number; cs: number }; color: number; prop: number | null }; stamp: [number, number] | null;
  tafel: { img: string; n: number; w: number; h: number }; reiter: number; charsets: string[]; colorMode: "tile" | "cell" | "none"; propDefs: { name: string; bit: number }[];
  sets: string[]; setRel: string | null; setError: string | null; palette: { i: number; color: string }[];
  selection: [number, number, number, number] | null; textPos: [number, number];
  ask: { at: [number, number]; dup: number | null; aligned: boolean; replace: boolean; name: string } | null;
  bankFiles: string[]; outputBytes: number; canUndo: boolean; canRedo: boolean; dirty: boolean; status: string;
}
/** Anzeige-DTO des Kachel-Editors (`vc-shell::kacheln`) */
export interface KachelView {
  rel: string; kind: "tileset"; title: string; name: string; target: string; atari: boolean; tw: number; th: number; count: number; sel: number; cur: number;
  tiles: { i: number; name: string; col: number; props: number; uses: number; double: number | null; img: string }[];
  tile: { img: string; w: number; h: number; name: string; col: number; props: number } | null; cells: { code: number; cs: number; col: number | null; sel: boolean }[];
  tafel: { img: string; n: number; w: number; h: number }; reiter: number; charsets: string[]; angebot: string[]; usedCharset: boolean[];
  colorMode: "tile" | "cell" | "none"; brush: { code: number; cs: number; sel: boolean; mode: "char" | "color" | "both"; color: number }; bg: number; palette: { i: number; color: string }[];
  propDefs: { name: string; bit: number }[]; fill: { used: number; max: number; doubles: number; err: string | null } | null; banks: { rel: string; screens: number }[];
  ask: null | { kind: "delete"; i: number; uses: number } | { kind: "size"; w: number; h: number }; hasClip: boolean; canUndo: boolean; canRedo: boolean; dirty: boolean; status: string;
}
/** Anzeige-DTO des Palette-Editors (`vc-shell::palette`) */
export interface PaletteView {
  rel: string; kind: "palette"; title: string; name: string; profile: string; count: number; entries: string[]; sel: number; rgb: [number, number, number]; hex: string;
  uses: { file: string; kind: string }[]; bytes: number; dirty: boolean; canUndo: boolean; canRedo: boolean; status: string;
}
/** Zustand der Projektdatei für den Bereich Projekt */
export interface ProjektInfo {
  file: string; name: string; description: string; main: string | null; mainEffective: string | null; sources: { file: string; kind: string }[]; groups: number; missing: string[];
  targets: { id: string; system: string; variant: string; variants: string[]; emulator: string; startOn: string; illegals: string; defaultEmulator: string }[]; sourceGroups: { kind: string; targets: string[]; dialect: string; files: { file: string; missing: boolean }[] }[]; disk: { format: string; name: string; file: string; files: string[] } | null; turbo: boolean; restoreTabs: boolean; errors: string[]; warnings: string[];
}
export interface TreeNode { kind: "dir" | "file"; name: string; rel: string; tag?: string; /** Art-Marke, z. B. `turbo-disk` für die Diskette des Anwenders */ mark?: string; children?: TreeNode[] }
/** BASIC-Einstellungen des Projekts (Abschnitt `basic` der Projektdatei) */
export interface BasicState { present: boolean; start: number; step: number; compact: boolean; check: boolean; disabled: string[]; maxLine: number; error: string | null }
/** Umwandlung von BASIC im Editor (Kern: `vc_ide::basicconv`): `mode` fragt nur den Modus ab */
/** Sprachdienste für natives BASIC (Kern: `vc_ide::basicinfo`) */
export interface BasicInfo { dialect: { id: string; name: string; family: string } | null; system: string; defines: { name: string; value: string; line: number }[]; excluded: number[]; outline: { name: string; line: number; kind: "label" | "const" }[] }
export type BasicConvKind = "labels" | "numbers" | "renumber" | "mode";
export interface BasicConvDiag { file: string; line: number; severity: "error" | "warning"; code: string; message: string; key?: string; args?: Record<string, string | number> }
export interface BasicConvResult {
  ok: boolean; mode?: "numbers" | "labels" | "mixed" | "undetermined"; numbering?: { start: number; step: number }; includes?: number; media?: number; maxLine?: number;
  changes?: { file: string; line: number; old: string; new: string }[]; newLabels?: { name: string; file: string; line: number }[]; kept?: number;
  warnings?: BasicConvDiag[]; diagnostics?: BasicConvDiag[]; files?: { file: string; changed: boolean }[]; text?: string; written?: string[]; error?: string;
}
export interface OpenedProject { root: string; pretty: string; name: string; systems: string[]; tree: TreeNode[]; /** Turbo-BASIC-XL-Projekt (Atari) */ turbo?: boolean }
export interface NewProjCard { i: number; name: string; description: string; basic: boolean; systems: string[]; files: string[] }
/** Plan für eine auf das Fenster gezogene Datei: was damit geschieht */
export interface DropItem { path: string; name: string; action: "project" | "open" | "disk" | "copy" | "none"; rel?: string; reason?: "no-project" | "type" | "folder" | "missing" }
export interface NewProjInfo { systems: { id: string; name: string; cpu: string }[]; default: string; cards: NewProjCard[]; structures: { id: string; name: string; description: string }[]; dir: string; dirPretty: string }
export interface NewProjSystem {
  valid: boolean; cpu: string; chips: string; dialects: { id: string; label: string }[]; disks: string[]; norm: boolean; memory: { id: string; label: string }[];
  emulators: { id: string; name: string; global: boolean }[]; globalEmulatorKind: string; globalEmulator: string; machines: { id: string; name: string }[]; globalStart: string;
}
export interface NewProjForm {
  system: string; extra: string[]; basic: boolean; card: number | null; name: string; dir: string; dialect: string; norm: string; memory: string; emulator: string; startOn: string;
  disk: string; charset: string; structure: number; readme: boolean; git: boolean;
}
export interface RasterDelta { cells: [number, number, number][]; undoDepth: number }
export type Eol = "lf" | "crlf" | "gemischt";
export type TextLang = "asm" | "basic" | "markdown" | "plain";
export interface FileContent { rel: string; text: string; eol: Eol; lang: TextLang; bytes: number }
/** Färbung: je Zeile (ab 0) Liste `[von, bis, Art]` in UTF-16-Einheiten */
export type ColorLines = [number, [number, number, string][]][];
export interface TreeResult { rel?: string; tree: TreeNode[]; moved?: [string, string][]; rewritten?: [string, number][]; trashed?: boolean }
export interface MoveResult { moved: [string, string][]; rewritten: [string, number][]; tree: TreeNode[] }
export interface UndoResult { moves: [string, string][]; n: number; tree: TreeNode[] }
/** Meldung der Sprachdienste: Katalogschlüssel und Parameter (I2); `line`/`endLine` ab 0, Spalten in UTF-16-Einheiten, `from` 0 und `to` am Zeilenende = ganze Zeile */
export interface LspDiag { line: number; endLine: number; from: number; to: number; severity: 1 | 2 | 3 | 4; code: string; key: string; args: Record<string, unknown>; message: string }
export interface LspDiagEvent { rel: string; version: number | null; items: LspDiag[] }
export interface LspHover { line: number; endLine: number; from: number; to: number; markdown: string; lineMode?: Record<string, unknown> }
export interface LspItem { label: string; kind?: number; detail?: string; documentation?: string; sortText?: string }
export interface LspFix { title: string; edits: { line: number; endLine: number; from: number; to: number; text: string }[] }
export interface LspSymbol { name: string; line: number; kind: "label" | "const" | "macro" }
export interface SearchHit { rel: string; line: number; text: string; from: number; to: number }
export interface SearchDone { id: number; files: number; truncated: boolean; cancelled: boolean; limit: number }
export interface BuildDiag { file: string; line: number; severity: "error" | "warning" | "info"; code: string; message: string }
export interface BuildStage { title: string; status: string; ms: number; lines: string[] }
export interface BuildResult { ok: boolean; file?: string; target?: string; artifact?: string; size?: number; error?: string; diagnostics: BuildDiag[]; segments?: { start: number; length: number }[]; log?: { ms: number; status: string; stages: BuildStage[] }; disk?: { file: string; size: number; programs: string[] }; disk_error?: string; disk_skipped?: string }
export interface BuildDone { ok: boolean; rel: string; result: BuildResult; then: string; /** Nur bei Turbo-BASIC-XL-Projekten: `ok`, `missing` (nur die Diskette fehlt) oder `error` */ turbo?: "ok" | "missing" | "error" }
export interface BuildInfo { rel: string; target: string; artifact: string; size: number; run: number | null; segments: [number, number][] }
export interface RunTargets { system: string; current: string; name: string; targets: { id: string; name: string }[] }
export interface TestItem { name: string; state: "passed" | "failed" | "timeout" | "skipped"; level: "error" | "hint"; detail: string; measured: string | null; location: [string, number] | null; regs: { pc: number; a: number; x: number; y: number; sp: number; flags: number } | null; file: string }
export interface TestGroup { title: string; file: string | null; backend: string | null; mailbox: boolean; millis: number; expected: number; items: TestItem[] }
export interface TestSuite { running: boolean; error: string | null; millis: number | null; counts: { passed: number; failed: number; timedOut: number; skipped: number; hints: number }; groups: TestGroup[]; backends?: { id: string; name: string }[]; files?: string[] }
export interface TurboEntry { name: string; sectors: number; locked: boolean }
export interface TurboReport {
  sha256: string; size: number; known: string | null; label: string | null; name: string; dos2: boolean; turbo: boolean; version: string | null; program_present: boolean; programLocked: boolean;
  free_sectors: number; kb: string; files: { name: string; sectors: number; locked: boolean }[]; entries: TurboEntry[]; problems: string[]; hints: string[]; lines: string[];
  reduce: { removes: string[]; free: number; kb: string } | null;
  /** Nur bei `turbo_check` mit Datei: Ziel der Kopie, ob dort schon eine liegt, ob die Quelle die Kopie selbst ist, aktuelle Kopie des Projekts */
  dest?: string; destExists?: boolean; sameAsCopy?: boolean; file?: string; current?: string | null;
}
export interface TurboState {
  turbo: boolean; disk: string; present: boolean; reduced: boolean; program: string; programIsDefault: boolean; files: string[]; changed: boolean; free: number | null;
  report: TurboReport | null; errors: string[]; warnings: string[];
}
export interface TurboProvided { disk: string; removed: string[]; free: number; sha256: string; gitignoreAdded: boolean; lines: string[]; state: TurboState; tree: TreeNode[] }
export interface AppInfo { pid: number; os: string; arch: string; version: string; testbridge: boolean; debug: boolean }

interface CommandMap {
  i18n_ftl: [{ lang: LangCode }, FtlSources];
  zahlen_werte: [{ text: string; bits?: number; toggle?: number }, ZahlenWerte];
  dom_info: [{ info: Record<string, unknown> }, null];
  lang_get: [undefined, LangState];
  lang_set: [{ choice: LangChoice }, { lang: LangCode }];
  commands_table: [undefined, CommandInfo[]];
  registry_info: [undefined, RegistryInfo];
  features_get: [undefined, FeaturesInfo];
  menu_context: [{ project: boolean; editor: boolean; turbo?: boolean }, null];
  recent_list: [undefined, RecentProject[]];
  project_pick: [undefined, string | null];
  project_open: [{ path: string }, OpenedProject];
  project_close: [undefined, null];
  doc_list: [undefined, unknown[]];
  ping: [unknown, unknown];
  raster_apply: [{ cells: [number, number, number][] }, RasterDelta];
  raster_reset: [undefined, null];
  app_info: [undefined, AppInfo];
  app_quit: [undefined, null];
  file_read: [{ rel: string }, FileContent];
  file_write: [{ rel: string; text: string; eol: Eol }, null];
  file_create: [{ rel: string }, TreeResult];
  dir_create: [{ rel: string }, TreeResult];
  file_rename: [{ old: string; new: string }, TreeResult];
  dir_rename: [{ old: string; new: string }, TreeResult];
  file_delete: [{ rel: string; trash: boolean }, TreeResult];
  syntax_color: [{ text: string; lang: TextLang; rel?: string }, ColorLines];
  fold_ranges: [{ text: string; lang: TextLang; rel?: string }, [number, number][]];
  tree_move: [{ from: string; folder: string }, MoveResult];
  tree_undo: [undefined, UndoResult];
  tree_reload: [undefined, { tree: TreeNode[] }];
  abs_path: [{ rel: string }, string];
  file_reveal: [{ rel: string }, null];
  file_external: [{ rel: string; confirmed?: boolean }, null];
  file_stats: [{ rels: string[] }, ([number, number] | null)[]];
  file_pick: [undefined, { rel?: string; outside?: string } | null];
  tree_can_trash: [undefined, boolean];
  doc_open: [{ rel: string; text: string }, null];
  doc_change: [{ rel: string; version: number; changes: { from: number; to: number; insert: string }[] }, null];
  doc_close: [{ rel: string }, null];
  lsp_hover: [{ rel: string; line: number; col: number }, LspHover | null];
  lsp_complete: [{ rel: string; line: number; col: number }, LspItem[]];
  lsp_definition: [{ rel: string; line: number; col: number }, { rel: string; line: number; col: number } | null];
  lsp_outline: [{ rel: string }, LspSymbol[]];
  lsp_fixes: [{ rel: string; from: number; to: number }, LspFix[]];
  search_start: [{ text: string; case: boolean; overrides: Record<string, string> }, { id: number }];
  search_cancel: [undefined, null];
  text_hover: [{ rel: string; lang: TextLang; text: string; line: number; col: number }, LspHover | null];
  char_names: [{ rel: string; lang: TextLang; line_text: string; col: number }, [string, string][]];
  build_start: [{ active?: string | null; onlyActive?: boolean; then?: "none" | "run" }, { rel: string }];
  run_start: [undefined, null];
  run_stop: [undefined, boolean];
  build_info: [undefined, BuildInfo | null];
  build_cycles: [{ rel: string }, [number, number, number][]];
  tests_start: [{ only?: "all" | "failed" | "file"; file?: string; backend?: string | null }, null];
  tests_stop: [undefined, null];
  tests_state: [undefined, TestSuite];
  recent_pin: [{ path: string }, RecentProject[]];
  recent_remove: [{ path: string }, RecentProject[]];
  recent_reveal: [{ path: string }, null];
  disk_pick: [undefined, string | null];
  disk_open: [{ rel?: string; path?: string }, { docId: number; kind: "disk"; view: DiskView }];
  disk_view: [{ docId: number }, DiskView];
  disk_import: [{ docId: number }, DiskView];
  disk_add_pick: [undefined, string[]];
  disk_save_pick: [{ name?: string }, string | null];
  disk_new: [{ format: string; name: string; id: string }, { docId: number; kind: "disk"; view: DiskView }];
  disk_apply: [{ docId: number; op: Record<string, unknown> }, DiskView];
  disk_close: [{ docId: number }, boolean];
  audio_poll: [undefined, AudioStand];
  help_topics: [undefined, { topics: { id: string; group: string; title: string }[] }];
  help_search: [{ q: string; target?: string }, { id: string; title: string; snippet: string }[]];
  help_open: [{ key: string; target?: string; lang?: string }, null | { title: string; markdown: string; id: string }];
  help_image: [{ id: string; fetch?: boolean }, { state: string; img?: string; error?: string }];
  wiz_stand: [undefined, { start: "first" | "offer" | "normal"; suppressed: boolean; incomplete: boolean; later?: string[]; news?: { kind: string; label: string }[] }];
  wiz_news: [{ mode: "seen" | "off" }, null];
  wiz_start: [{ mode: string }, WizData];
  wiz_set: [Record<string, unknown>, WizData];
  wiz_system: [{ id: string; on: boolean; deselected?: string[] }, WizData];
  wiz_apply: [{ deselected: string[] }, WizData & { report: { notes: string[]; copied: string[]; backup: string | null } }];
  wiz_undo: [{ deselected?: string[] }, WizData];
  wiz_later: [{ topic: string }, null];
  wiz_end: [undefined, null];
  info_about: [undefined, AboutInfo];
  info_update_state: [undefined, UpdateStand];
  info_update_check: [{ auto?: boolean }, UpdateStand & { error?: string | null }];
  info_update_later: [undefined, UpdateStand];
  info_update_skip: [{ version: string }, UpdateStand];
  info_feedback: [{ system?: string } | undefined, FeedbackMail];
  info_mail: [{ url: string }, null];
  info_open: [{ which: "web" | "repo" | "release" | "credit"; version?: string; id?: string; index?: number }, null];
  settings_get: [undefined, SettingsReply];
  args_info: [{ kind: string; opts: Record<string, string>; free: string; target?: string }, ArgsInfo];
  args_migrate: [{ kind: string; opts: Record<string, string>; free: string }, { opts: Record<string, string>; free: string }];
  settings_op: [{ settings: Record<string, unknown>; op: Record<string, unknown> }, { settings: Record<string, any> }]; // eslint-disable-line @typescript-eslint/no-explicit-any
  ziele_info: [undefined, { systems: { id: string; name: string; group: string; cpu: string; kinds: { id: string; label: string }[]; hardware: { id: string; label: string }[] }[] }];
  ziele_probe: [Record<string, unknown>, any]; // eslint-disable-line @typescript-eslint/no-explicit-any
  roms_info: [{ useForeign?: boolean } | undefined, RomsInfo];
  md_render: [{ rel: string; text: string; from?: number; count?: number }, MdRender];
  md_url_open: [{ url: string }, null];
  md_edit: [{ op: string; text: string; a: number; b: number; arg?: string }, { text: string; a: number; b: number } | null];
  chips_info: [{ target: string }, { chips: { id: string; name: string; start: number; len: number; regs: { name: string; addr: number; access: string; doc: string }[] }[] }];
  emu_pakete: [{ kind: string }, EmuPakete];
  emu_install: [{ kind: string; version: string }, string];
  roms_pick: [undefined, string[]];
  roms_import_plan: [{ paths?: string[]; system?: string; foreign?: boolean }, { items: RomImportItem[] }];
  roms_import_run: [{ paths?: string[]; system?: string; foreign?: boolean; pick: number[] }, { copied: string[]; errors: string[] }];
  roms_pin: [{ system: string; role: string; file?: string }, null];
  roms_open: [undefined, null];
  settings_save: [{ settings: Record<string, unknown> }, SettingsReply];
  settings_export: [undefined, null | { path: string }];
  settings_import_pick: [undefined, null | { path: string; targets: number; machines: number; projectsDir: string }];
  settings_import_apply: [{ path: string }, SettingsReply];
  settings_reset: [undefined, SettingsReply];
  settings_reveal: [undefined, null];
  hex_list: [undefined, { rel: string; size: number }[]];
  hex_page: [{ rel: string; page: number; width: number }, HexPage];
  hex_find: [{ rel: string; query: string; from: number }, null | { off: number; len: number }];
  speicher_info: [{ preset?: string; page?: number } | undefined, SpeicherInfo];
  debug_start_test: [{ file: string; test: string }, { ok: boolean }];
  debug_start: [{ bps: Record<string, number[]>; watches: string[] }, { ok: boolean }];
  debug_state: [undefined, DebugState];
  debug_cmd: [Record<string, unknown>, null];
  debug_frame: [{ seq: number }, null | { seq: number; w: number; h: number; img: string }];
  debug_stop: [undefined, boolean];
  song_quelle_pick: [{ rmt: boolean }, null | SongQuelle];
  song_import_run: [{ path: string; rmt: boolean; name: string; ordner: string; ziel?: string; subtune?: number; sekunden?: number }, { rel: string; warnings: string[] }];
  tracker_open: [{ rel: string }, { docId: number; kind: "song"; view: TrackerView }];
  bildschirm_open: [{ rel: string }, { docId: number; kind: "screen"; view: BildschirmView }];
  kacheln_open: [{ rel: string }, { docId: number; kind: "tileset"; view: KachelView }];
  karten_open: [{ rel: string }, { docId: number; kind: "bank"; view: KartenView }];
  palette_open: [{ rel: string }, { docId: number; kind: "palette"; view: PaletteView }];
  pixel_open: [{ rel: string }, { docId: number; kind: "pixel"; view: PixelView }];
  pixel_view: [{ docId: number }, PixelView];
  pixel_apply: [{ docId: number; op: Record<string, unknown> }, PixelView];
  pixel_close: [{ docId: number }, boolean];
  png_pick: [undefined, string | null];
  media_profiles: [{ kinds: string[] }, { id: string; name: string; kind: string; target: string }[]];
  media_new: [{ name: string; profile: string; dir: string }, { path: string }];
  export_info: [{ docId: number }, { rel: string; kind: string; name: string; addr: string; formats: string[] }];
  export_preview: [{ docId: number; format: string; name: string; addr: string; routine: boolean }, { files?: [string, number][]; text?: string; hex?: string; error?: string | null }];
  export_run: [{ docId: number; action: "save" | "copy" | "include"; format?: string; name: string; addr: string; routine?: boolean; targetBasic?: boolean; target?: string; text?: string }, { files?: string[]; text?: string; line?: string }];
  examples_list: [undefined, { name: string; system: string; title: string; description: string; path: string; readonly: boolean; featured: boolean; source?: string | null }[]];
  tools_list: [undefined, { id: string; label: string; found: boolean; text: string }[]];
  vcprj_pick: [undefined, string | null];
  copy_suggest: [{ src: string; dir: string }, string];
  copy_check: [{ src: string; dir: string; name: string }, string | null];
  copy_plan: [{ src: string; withBuild: boolean; withGit: boolean }, { files: number; bytes: number } | null];
  include_resolve: [{ from: string; path: string }, { status: "ok" | "legacy" | "missing"; rel?: string; medium?: boolean }];
  copy_run: [{ src: string; dir: string; name: string; withBuild: boolean; withGit: boolean; /** Zielsystem der Kopie wird Normalziel (leer = wie das Original) */ system?: string }, { root: string; files: number; bytes: number; skipped: string[]; /** Die Kopie enthält eine Diskette des Anwenders (turbo/*.atr) */ turboDisk?: boolean }];
  start_args: [undefined, { paths: string[]; open: string | null; view: string | null }];
  drop_plan: [{ paths: string[] }, DropItem[]];
  drop_copy: [{ path: string }, string];
  newproj_info: [undefined, NewProjInfo];
  newproj_system: [{ system: string }, NewProjSystem];
  newproj_preview: [NewProjForm, string[]];
  newproj_create: [NewProjForm, OpenedProject & { notes: string[] }];
  dir_pick: [{ start?: string }, string | null];
  session_get: [undefined, { enabled: boolean; open: string[]; active: string | null; marks: Record<string, FileMarks> }];
  session_set: [{ open: string[]; active: string | null; marks?: Record<string, FileMarks> }, null];
  restore_tabs_set: [{ on: boolean }, null];
  scratch_get: [undefined, { enabled: boolean; entries: { name: string; text: string; note: boolean; savedAt: number }[] }];
  scratch_set: [{ entries: { name: string; text: string; note: boolean }[] }, null];
  scratch_pref: [{ on?: boolean }, boolean];
  basic_info: [{ rel: string; text: string }, BasicInfo];
  basic_codes: [undefined, Record<string, { kind: "error" | "warning" | "hint"; disable: boolean; text: string }>];
  basic_convert: [{ kind: BasicConvKind; rel: string; text: string; includes?: boolean; from?: number; to?: number; apply?: boolean }, BasicConvResult];
  basic_get: [undefined, BasicState];
  basic_set: [{ start: number; step: number; compact: boolean; check: boolean; disabled: string[] }, BasicState];
  projekt_info: [undefined, ProjektInfo];
  projekt_set: [{ name?: string; description?: string; main?: string | null; restoreTabs?: boolean; variant?: { id: string; variant: string }; move?: { file: string; dir: number }; addTarget?: string; removeTarget?: string; groupAdd?: { kind: string }; groupRemove?: number; groupSet?: { i: number; kind?: string; dialect?: string; targets?: string[] }; fileAdd?: { group: number; file: string }; fileRemove?: string; targetSet?: { id: string; illegals?: string; startOn?: string; defaultEmulator?: string }; disk?: { format?: string; name?: string; files?: string[] } | null }, ProjektInfo];
  turbo_state: [undefined, TurboState];
  turbo_pick: [undefined, string | null];
  turbo_check: [{ path?: string }, TurboReport | TurboState];
  turbo_provide: [{ src: string; reduce: boolean; anyway: boolean; replace: boolean }, TurboProvided];
  turbo_reduce: [undefined, { removed: string[]; state: TurboState }];
  turbo_set: [{ program: string; files: string[] }, TurboState];
  run_targets: [{ rel?: string }, RunTargets];
  run_target_set: [{ id: string }, null];
  format_lines: [{ text: string; rel: string; lang: TextLang; from: number; to: number }, [number, string][]];
}

type Handler = (payload: unknown) => void;
type Unlisten = () => void;

interface Transport {
  call(cmd: string, args: unknown): Promise<unknown>;
  listen(ev: string, h: Handler): Promise<Unlisten>;
  readonly kind: "tauri" | "bridge" | "none";
}

const noneTransport: Transport = {
  kind: "none",
  call: () => Promise.reject(new CoreError({ code: "keine-verbindung", args: {} })),
  listen: () => Promise.resolve(() => {}),
};

async function tauriTransport(): Promise<Transport> {
  const { invoke } = await import("@tauri-apps/api/core");
  const { listen } = await import("@tauri-apps/api/event");
  return {
    kind: "tauri",
    call: async (cmd, args) => {
      try { return await invoke("vc", { cmd, args: args ?? null }); }
      catch (e) { throw new CoreError(isFailure(e) ? e : { code: "unbekannt", args: { detail: String(e) } }); }
    },
    listen: async (ev, h) => await listen(ev, (e) => h(e.payload)),
  };
}

function isFailure(e: unknown): e is Failure { return typeof e === "object" && e !== null && typeof (e as Failure).code === "string"; }

function bridgeTransport(port: string): Promise<Transport> {
  return new Promise((resolve, reject) => {
    const ws = new WebSocket(`ws://127.0.0.1:${port}`);
    let next = 1;
    const pending = new Map<number, { ok: (v: unknown) => void; err: (e: CoreError) => void }>();
    const handlers = new Map<string, Set<Handler>>();
    ws.onmessage = (m) => {
      const v = JSON.parse(String(m.data)) as { id?: number; ok?: unknown; err?: Failure; event?: string; payload?: unknown };
      if (v.event) { handlers.get(v.event)?.forEach((h) => h(v.payload)); return; }
      const p = v.id !== undefined ? pending.get(v.id) : undefined;
      if (!p) return;
      pending.delete(v.id!);
      if (v.err) p.err(new CoreError(v.err)); else p.ok(v.ok ?? null);
    };
    ws.onerror = () => reject(new Error("test bridge not reachable"));
    ws.onopen = () => resolve({
      kind: "bridge",
      call: (cmd, args) => new Promise((ok, err) => { const id = next++; pending.set(id, { ok, err }); ws.send(JSON.stringify({ id, cmd, args: args ?? null })); }),
      listen: (ev, h) => { const s = handlers.get(ev) ?? new Set(); s.add(h); handlers.set(ev, s); return Promise.resolve(() => s.delete(h)); },
    });
  });
}

let transport: Promise<Transport> | undefined;

/** Weg wählen (einmal): Tauri, sonst Prüfbrücke, sonst kein Kern */
export function connect(): Promise<Transport> {
  transport ??= (async () => {
    if (typeof window !== "undefined" && "__TAURI_INTERNALS__" in window) return tauriTransport();
    // Die Prüfbrücke gibt es nur im Entwicklungsbau (Vite-Server); im Auslieferungsbau entfällt dieser Zweig samt Brückencode ganz
    if (import.meta.env.DEV) {
      const port = typeof location !== "undefined" ? new URLSearchParams(location.search).get("bridge") : null;
      if (port) { try { return await bridgeTransport(port); } catch { return noneTransport; } }
    }
    return noneTransport;
  })();
  return transport;
}

export async function transportKind(): Promise<Transport["kind"]> { return (await connect()).kind; }

/** Befehl an den Kern; Fehler kommen als `CoreError` mit `code` und `args` */
export async function call<C extends Command>(cmd: C, ...args: CommandMap[C][0] extends undefined ? [] : [CommandMap[C][0]]): Promise<CommandMap[C][1]> {
  const t = await connect();
  return (await t.call(cmd, args[0])) as CommandMap[C][1];
}

/** Befehl ohne Typprüfung (nur Selbstmessung der Abnahme, Feature `selftest` des Kerns) */
export async function callAny(cmd: string, args: unknown): Promise<unknown> { return (await connect()).call(cmd, args); }

/** Auf ein Ereignis des Kerns hören; liefert die Abmeldung */
export async function on(ev: CoreEvent, h: Handler): Promise<Unlisten> { return (await connect()).listen(ev, h); }

/** Binärer Weg (nur Tauri): `n` Bytes ohne JSON-Umweg (Messung) */
export async function callBinary(n: number): Promise<ArrayBuffer> {
  const t = await connect();
  if (t.kind !== "tauri") throw new CoreError({ code: "keine-verbindung", args: {} });
  const { invoke } = await import("@tauri-apps/api/core");
  return (await invoke("vc_bin", { n })) as ArrayBuffer;
}
