# Customize the layout

The window has a **fixed frame**: header bar with the main areas, below it the second row, then the work area and the status bar. In the area **Code**, the sidebar, the tray below the editor and the machine panel are added. You drag the sizes, collapse areas, make one area large or hide everything except the work area.

![The main window in the area 'Code'. At the top is the header with the project name and the main areas from 'Startseite' to 'Debug'. On the left is the sidebar 'PROJEKT' with the file tree, in the middle the editor with jumprun.asm, below it the drawer with the tab 'Ausgabe', on the right the panel 'Maschine', and at the bottom the status bar.](bild:ui-gesamt "The main window at a glance: header, sidebar, editor, drawer, machine panel and status bar.")

| Zone | Content | Show/hide |
|---|---|---|
| Header bar | Button for the sidebar, project name, the main areas (Project, Code, Graphics, Data, and Music and Debug where available), Build, Run, Stop, run target, language, color scheme, command palette | always visible |
| Second row | the subsections of the chosen main area (for example in Graphics: Sprites, Character set, Palette) | always visible |
| Sidebar "Project" | Files (project tree) | Cmd/Ctrl+B |
| Document area | the file or the editor of the chosen area; in the area Code all files as tabs | always visible |
| Tray below the editor | Problems, Output, Tests, Search, Outline | Cmd/Ctrl+J |
| Panel "Machine" | run target, state and result of the last build | Cmd/Ctrl+Alt+B |
| Status bar | project, language of the file, line ending, problems, line and column, result of the build; on the right the note "All commands" | always visible |

The sidebar, the tray and the machine panel exist only in the area **Code**. If the sidebar is closed, a narrow strip stays with the buttons for files and search.

With **View ▸ Areas ▸ Project area** (Cmd/Ctrl+Alt+0) you jump to the main area Project.

## Zen mode
**Cmd/Ctrl+Alt+Z** hides the sidebar, the tray, the machine panel and the status bar. The header bar, the second row and the work area stay. The same command brings everything back.

## Make areas large
Every area has a button **Maximize area** at the top. Maximizing collapses the other areas; **Restore area** or a double click on the header of the tray brings the layout back. **Cmd/Ctrl+Shift+Enter** toggles the area that has the focus. Every area also has its own command in **View ▸ Areas** and in the command palette.

## Change sizes
You can drag the dividers between the areas or move them with the arrow keys (Shift for large steps, Home and End for the smallest and largest size). A double click restores the default.

## Narrow windows
If the work area becomes very narrow, the sidebar and the panel float over the document. The editors then show fewer things side by side and arrange their cards one below the other.

**View ▸ Areas ▸ Reset layout** (Mac menu; otherwise the command palette) restores the default sizes. VintageCoder remembers sizes and visibility by itself; a maximized area is not remembered.

See also [Settings](help:settings), [Keyboard shortcuts](help:shortcuts) and [File tree](help:filetree).
