// VintageCoder – Anbindung des Language Servers (vc-lsp) an einen Editor mit Erweiterungsschnittstelle. Nicht automatisch getestet.
const vscode = require("vscode");
const { LanguageClient } = require("vscode-languageclient/node");
let client;
exports.activate = function () {
  const cfg = vscode.workspace.getConfiguration("vintagecoder");
  const command = cfg.get("serverPath") || "vc-lsp";
  const initializationOptions = {};
  if (cfg.get("dataDir")) initializationOptions.dataDir = cfg.get("dataDir");
  initializationOptions.target = cfg.get("target") || "c64";
  client = new LanguageClient("vintagecoder", "VintageCoder", { command, args: [] },
    { documentSelector: [{ language: "vcasm" }], initializationOptions });
  client.start();
};
exports.deactivate = function () { return client ? client.stop() : undefined; };
