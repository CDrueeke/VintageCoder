# Back up and reset the configuration

Under **Settings > Backup** (card **Backup**) you back up the configuration and bring it back. The configuration includes target systems, emulators, machines, the help choice and the update choice. **Passwords of the machines are never included.** With the configuration you can quickly set up a second computer like the first.

![The settings with the list on the left (from 'Allgemein' to 'Info und Update'), with 'Sicherung' selected. On the right is the card 'SICHERUNG' with a short explanatory text and the buttons 'Konfiguration exportieren ...', 'Konfiguration importieren ...' and 'Auf Vorgabe zuruecksetzen ...'.](bild:einstellungen-sicherung "The 'Sicherung' card in the settings with export, import and reset.")

The card has three buttons:

- **Export configuration …** writes the configuration to a JSON file that you name in the file dialog. The message names the location ("Exported to … (without passwords)").
- **Import configuration …** reads such a file. Before that, a dialog shows what is in it ("The file contains … target systems and … machines. The current settings are replaced."). Only **Apply** replaces your current settings. Passwords of machines you already have are kept.
- **Reset to default …** sets all settings back to the default, including target systems, emulators, machines and their passwords. A dialog asks first; this cannot be undone.

The setup assistant (see [Start page, setup assistant and new project](help:wizard)) is not part of this; it makes a backup of its own before writing and then offers **Undo**. Projects are not touched by the configuration. Color scheme and zoom of the user interface are not part of the configuration, and neither is the arrangement of the areas. See [Settings](help:settings).
