# SPDX-License-Identifier: GPL-3.0-or-later
"""Linux-Pakete (.deb, .rpm, tar.gz) aus dem FHS-Staging von `release.py stage --os linux` (Konzept Release-Auswahl, Abschnitt 8.4).

Wird von tools/release.py (Unterbefehl `package`) benutzt. Nur Python-Standardbibliothek; gebaut wird mit `dpkg-deb` und `rpmbuild`
direkt (kein nfpm, kein cargo-deb). Die reinen Funktionen (Versionsabbildung, Dateilisten, control/spec, Abhaengigkeiten) laufen
ohne diese Werkzeuge und sind in tools/test_linux_pkg.py getestet.
"""
import gzip, hashlib, io, json, os, re, shutil, subprocess, sys, tarfile, tempfile, time

ROOT = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
SHARE = "usr/share/vintagecoder"
DOC = "usr/share/doc/vintagecoder"
DISTROS = ["debian", "ubuntu", "fedora", "opensuse"]
FORMATS = ["deb", "rpm", "tar"]
REPRODUCIBLE_EPOCH = 1767225600          # 2026-01-01T00:00:00Z fuer --reproducible
SEMVER = re.compile(r"^(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+([0-9A-Za-z.-]+))?$")


class PackageError(Exception):
    pass


# ---------------------------------------------------------------- Versionen

def split_version(v):
    """'0.2.0-beta.1+b5' -> ('0.2.0', 'beta.1', 'b5'); Fehler bei Nicht-SemVer."""
    m = SEMVER.match(v)
    if not m:
        raise PackageError("Version '%s' ist keine SemVer-Version (MAJOR.MINOR.PATCH[-vorab][+bau])" % v)
    return "%s.%s.%s" % m.group(1, 2, 3), m.group(4), m.group(5)


def deb_version(v):
    """Debian/Ubuntu: Vorabkennung mit Tilde (0.1.0~alpha sortiert vor 0.1.0), Baukennung mit Plus. Ohne Revision (natives Paket)."""
    core, pre, build = split_version(v)
    return core + ("~" + pre.replace("-", ".") if pre else "") + ("+" + build.replace("-", ".") if build else "")


def rpm_version_release(v):
    """Fedora-Schema: stabil Version=0.1.0 Release=1; Vorabversion Version=0.1.0 Release=0.1.<vorab> (sortiert vor 1)."""
    core, pre, build = split_version(v)
    rel = "0.1." + pre.replace("-", ".") if pre else "1"
    if build:
        rel += "." + re.sub(r"[^0-9A-Za-z.]", ".", build)
    return core, rel


def tar_version(v):
    split_version(v)
    return v


def deb_arch(machine):
    return {"x86_64": "amd64", "amd64": "amd64", "aarch64": "arm64", "arm64": "arm64", "armv7l": "armhf", "i686": "i386"}.get(machine, machine)


def rpm_arch(machine):
    return {"amd64": "x86_64", "arm64": "aarch64"}.get(machine, machine)


def glibc_version():
    """Version der glibc dieses Rechners (Mindestversion fuer libc6 in .deb); None ausserhalb von glibc-Linux."""
    try:
        m = re.match(r"glibc (\d+\.\d+)", os.confstr("CS_GNU_LIBC_VERSION"))
        return m.group(1) if m else None
    except (AttributeError, ValueError, OSError):
        return None


# ---------------------------------------------------------------- Konfiguration

def load_config(path=None):
    path = path or os.path.join(ROOT, "release", "linux-packages.json")
    with open(path, encoding="utf-8") as f:
        cfg = json.load(f)
    for d, c in cfg["distros"].items():
        if c["format"] not in ("deb", "rpm"):
            raise PackageError("%s: unbekanntes Format %s" % (d, c["format"]))
    return cfg


def detect_distro(os_release_text):
    """Schluessel (debian, ubuntu, fedora, opensuse) aus dem Inhalt von /etc/os-release; None, wenn unbekannt."""
    kv = {}
    for line in os_release_text.splitlines():
        if "=" in line and not line.startswith("#"):
            k, v = line.split("=", 1)
            kv[k.strip()] = v.strip().strip('"')
    ident = kv.get("ID", "").lower()
    like = kv.get("ID_LIKE", "").lower().split()
    if ident in ("debian", "ubuntu", "fedora"):
        return ident
    if ident.startswith("opensuse") or ident == "sles" or "suse" in like:
        return "opensuse"
    if "ubuntu" in like:
        return "ubuntu"
    if "debian" in like:
        return "debian"
    if "fedora" in like or "rhel" in like:
        return "fedora"
    return None


# ---------------------------------------------------------------- Staging lesen

def read_selection(stage):
    """Liest data/release.json des Staging: (ide, links, version)."""
    p = os.path.join(stage, SHARE, "data", "release.json")
    if not os.path.isfile(p):
        raise PackageError("%s fehlt: --stage muss ein Ergebnis von 'release.py stage --os linux' sein" % p)
    with open(p, encoding="utf-8") as f:
        rj = json.load(f)
    return bool(rj.get("ide")), list(rj.get("links", [])), rj.get("version")


def stage_files(stage):
    """Alle Dateien unter usr/ (relativ, sortiert). Alles andere im Staging ausser release-report.md ist ein Fehler (nichts Fremdes)."""
    top = sorted(os.listdir(stage))
    extra = [t for t in top if t not in ("usr", "release-report.md")]
    if extra:
        raise PackageError("Unerwartete Eintraege im Staging (kein FHS-Baum?): " + ", ".join(extra))
    out = []
    for d, dirs, files in os.walk(os.path.join(stage, "usr")):
        dirs.sort()
        for f in files:
            p = os.path.join(d, f)
            if os.path.islink(p):
                raise PackageError("Symbolische Verknuepfung im Staging: %s" % os.path.relpath(p, stage))
            out.append(os.path.relpath(p, stage).replace(os.sep, "/"))
    if not out:
        raise PackageError("Staging enthaelt keine Dateien unter usr/")
    return sorted(out)


def is_ide_file(rel):
    """Gehoert die Datei zum Paket 'vintagecoder' (IDE) statt zu 'vintagecoder-cli'?"""
    return (rel == "usr/bin/vintagecoder" or rel == "usr/share/man/man1/vintagecoder.1"
            or rel.startswith(("usr/share/applications/", "usr/share/icons/", DOC + "/licenses/")))


def split_components(files, ide):
    """{'cli': [...], 'ide': [...]}: ohne IDE-Auswahl darf keine IDE-Datei im Staging liegen."""
    cli = [f for f in files if not is_ide_file(f)]
    ide_files = [f for f in files if is_ide_file(f)]
    if ide_files and not ide:
        raise PackageError("Staging enthaelt IDE-Dateien, obwohl release.json ide=false sagt: " + ", ".join(ide_files[:3]))
    if ide and not ide_files:
        raise PackageError("release.json sagt ide=true, im Staging liegen aber keine IDE-Dateien (usr/bin/vintagecoder fehlt)")
    return {"cli": cli, "ide": ide_files} if ide else {"cli": cli}


def installed_path(rel):
    """Pfad im Paket: Manpages werden gzip-komprimiert (Debian-Richtlinie, rpm laesst .gz unberuehrt)."""
    p = "/" + rel
    if re.match(r"^/usr/share/man/man\d/.+\.\d$", p):
        p += ".gz"
    return p


def installed_paths(files):
    return sorted(installed_path(f) for f in files)


# ---------------------------------------------------------------- Abhaengigkeiten und Metadaten

def dependencies(cfg, distro, which, links, version, fmt, libc_min=None):
    """Beziehungen eines Pakets: {'depends': [...], 'recommends': [...], 'suggests': [...]} als Textzeilen im Format der Distribution.

    cli: nur libc (deb: mit Mindestversion der Bau-glibc; rpm erzeugt die ELF-Abhaengigkeiten selbst) und die gewaehlten Anbindungen
    (vice, atari800) als Recommends/Suggests laut Distribution. ide: GUI-Bibliotheken, vintagecoder-cli in gleicher Version, Empfehlungen."""
    dc = cfg["distros"][distro]
    rel = {"depends": [], "recommends": [], "suggests": []}
    if which == "cli":
        if fmt == "deb":
            rel["depends"].append("libc6 (>= %s)" % libc_min if libc_min else "libc6")
        for link in links:
            spec = dc.get("links", {}).get(link)
            if spec:
                rel[spec["level"]].append(spec["package"])
    else:
        cli = cfg["packages"]["cli"]["name"]
        if fmt == "deb":
            rel["depends"].append("%s (= %s)" % (cli, deb_version(version)))
        else:
            core, release = rpm_version_release(version)
            rel["depends"].append("%s = %s-%s" % (cli, core, release))
        ide = dc["ide"]
        rel["depends"] += ide["depends"]
        rel["recommends"] += ide.get("recommends", [])
        rel["suggests"] += ide.get("suggests", [])
    return rel


def installed_size_kib(stage, files):
    return max(1, (sum(os.path.getsize(os.path.join(stage, f)) for f in files) + 1023) // 1024)


def render_control(cfg, which, version, arch, rel, size_kib):
    """Inhalt der DEBIAN/control-Datei."""
    p = cfg["packages"][which]
    L = ["Package: " + p["name"], "Version: " + deb_version(version), "Architecture: " + arch,
         "Maintainer: " + cfg["maintainer"], "Installed-Size: %d" % size_kib]
    for key, field in (("depends", "Depends"), ("recommends", "Recommends"), ("suggests", "Suggests")):
        if rel[key]:
            L.append("%s: %s" % (field, ", ".join(rel[key])))
    L += ["Section: " + p["section"], "Priority: optional", "Homepage: " + cfg["homepage"], "Description: " + p["summary"]]
    L += [" " + (line if line.strip() else ".") for line in p["description"]]
    return "\n".join(L) + "\n"


def rpm_quote(path):
    if re.search(r"[*?\[\]{}\n]", path):
        raise PackageError("Dateiname mit Platzhalterzeichen im Paket nicht moeglich: " + path)
    path = path.replace("%", "%%")
    return '"%s"' % path if " " in path else path


def owned_dirs(paths):
    """Verzeichnisse, die das Paket besitzt (alles unter .../vintagecoder), von aussen nach innen sortiert."""
    dirs = set()
    for p in paths:
        d = os.path.dirname(p)
        while d and d != "/":
            if "vintagecoder" in d.split("/") and os.path.basename(d) != "":
                dirs.add(d)
            d = os.path.dirname(d)
    return sorted(dirs)


def rpm_date(epoch):
    days = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
    months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"]
    t = time.gmtime(epoch)
    return "%s %s %02d %d" % (days[t.tm_wday], months[t.tm_mon - 1], t.tm_mday, t.tm_year)


def render_spec(cfg, which, version, rel, paths, epoch):
    """Inhalt der .spec-Datei; %{payload} ist der vorbereitete Dateibaum (Makro per rpmbuild --define)."""
    p = cfg["packages"][which]
    core, release = rpm_version_release(version)
    L = ["%global debug_package %{nil}",
         "%global _build_id_links none",
         "Name:           " + p["name"],
         "Version:        " + core,
         "Release:        " + release,
         "Summary:        " + p["summary"],
         "License:        " + cfg["license"],
         "URL:            " + cfg["homepage"]]
    for key, field in (("depends", "Requires"), ("recommends", "Recommends"), ("suggests", "Suggests")):
        L += ["%-15s %s" % (field + ":", r) for r in rel[key]]
    L += ["", "%description"] + p["description"] + ["", "%prep", "", "%build", "", "%install",
          "mkdir -p %{buildroot}", "cp -a %{payload}/. %{buildroot}/", "", "%files", "%defattr(-,root,root,-)"]
    L += ["%dir " + rpm_quote(d) for d in owned_dirs(paths)]
    L += [rpm_quote(x) for x in paths]
    L += ["", "%changelog", "* %s %s - %s-%s" % (rpm_date(epoch), cfg["maintainer"], core, release), "- Paket aus der Release-Auswahl (tools/release.py package)", ""]
    return "\n".join(L)


# ---------------------------------------------------------------- Dateibaum fuer ein Paket

def build_payload(stage, files, dest, epoch=None):
    """Kopiert die Dateien nach dest (Modi 755/644, Manpages gzip -9n); gibt die Liste der installierten Pfade zurueck."""
    paths = []
    for rel in files:
        src = os.path.join(stage, rel)
        ip = installed_path(rel)
        dst = os.path.join(dest, ip.lstrip("/"))
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        if ip.endswith(".gz") and not rel.endswith(".gz"):
            with open(src, "rb") as f:
                data = f.read()
            with open(dst, "wb") as raw:
                with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0, compresslevel=9) as gz:
                    gz.write(data)
        else:
            shutil.copyfile(src, dst)
        os.chmod(dst, 0o755 if rel.startswith("usr/bin/") else 0o644)
        if epoch is not None:
            os.utime(dst, (epoch, epoch))
        paths.append(ip)
    for d, dirs, _ in os.walk(dest):
        os.chmod(d, 0o755)
        if epoch is not None:
            os.utime(d, (epoch, epoch))
    return sorted(paths)


def md5sums(dest, paths):
    L = []
    for p in paths:
        with open(os.path.join(dest, p.lstrip("/")), "rb") as f:
            L.append("%s  %s" % (hashlib.md5(f.read()).hexdigest(), p.lstrip("/")))
    return "\n".join(L) + "\n"


def sha256_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        for b in iter(lambda: f.read(1 << 20), b""):
            h.update(b)
    return h.hexdigest()


# ---------------------------------------------------------------- Bauen

def need_tool(name):
    if not shutil.which(name):
        raise PackageError("'%s' nicht gefunden (im Bau-Container der Distribution vorhanden; siehe tools/docker/)" % name)


def run(cmd, env=None, cwd=None):
    r = subprocess.run(cmd, env=env, cwd=cwd, capture_output=True, text=True, stdin=subprocess.DEVNULL)
    if r.returncode:
        raise PackageError("%s fehlgeschlagen (Rueckgabewert %d):\n%s" % (cmd[0], r.returncode, (r.stdout + r.stderr).strip()[-2000:]))
    return r.stdout + r.stderr


def build_deb(stage, files, cfg, distro, which, version, links, arch, out, epoch):
    need_tool("dpkg-deb")
    rel = dependencies(cfg, distro, which, links, version, "deb", glibc_version())
    name = cfg["packages"][which]["name"]
    tmp = tempfile.mkdtemp(prefix="vc-deb-")
    try:
        tree = os.path.join(tmp, "tree")
        paths = build_payload(stage, files, tree, epoch)
        os.makedirs(os.path.join(tree, "DEBIAN"))
        with open(os.path.join(tree, "DEBIAN", "control"), "w", encoding="utf-8", newline="\n") as f:
            f.write(render_control(cfg, which, version, arch, rel, installed_size_kib(stage, files)))
        with open(os.path.join(tree, "DEBIAN", "md5sums"), "w", encoding="utf-8", newline="\n") as f:
            f.write(md5sums(tree, paths))
        for d in ("DEBIAN",):
            os.chmod(os.path.join(tree, d), 0o755)
        fname = "%s_%s_%s.deb" % (name, deb_version(version), arch)
        env = dict(os.environ)
        if epoch is not None:
            env["SOURCE_DATE_EPOCH"] = str(epoch)
        run(["dpkg-deb", "--root-owner-group", "-Zxz", "--build", tree, os.path.join(out, fname)], env=env)
        return fname, paths
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def build_rpm(stage, files, cfg, distro, which, version, links, arch, out, epoch):
    need_tool("rpmbuild")
    rel = dependencies(cfg, distro, which, links, version, "rpm")
    name = cfg["packages"][which]["name"]
    tmp = tempfile.mkdtemp(prefix="vc-rpm-")
    try:
        payload = os.path.join(tmp, "payload")
        paths = build_payload(stage, files, payload, epoch)
        top = os.path.join(tmp, "top")
        for d in ("SPECS", "BUILD", "RPMS", "SRPMS", "BUILDROOT"):
            os.makedirs(os.path.join(top, d))
        spec = os.path.join(top, "SPECS", name + ".spec")
        with open(spec, "w", encoding="utf-8", newline="\n") as f:
            f.write(render_spec(cfg, which, version, rel, paths, epoch if epoch is not None else int(time.time())))
        cmd = ["rpmbuild", "-bb", "--define", "_topdir " + top, "--define", "payload " + payload, "--target", arch]
        env = dict(os.environ)
        if epoch is not None:
            env["SOURCE_DATE_EPOCH"] = str(epoch)
            cmd += ["--define", "use_source_date_epoch_as_buildtime 1", "--define", "clamp_mtime_to_source_date_epoch 1"]
        run(cmd + [spec], env=env)
        built = [os.path.join(d, f) for d, _, fs in os.walk(os.path.join(top, "RPMS")) for f in fs if f.endswith(".rpm")]
        if len(built) != 1:
            raise PackageError("rpmbuild hat %d Pakete erzeugt, erwartet 1" % len(built))
        fname = os.path.basename(built[0])
        shutil.copyfile(built[0], os.path.join(out, fname))
        return fname, paths
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


INSTALL_TXT = """VintageCoder %s (portables Archiv, Linux %s)

Ohne Installation starten:   usr/bin/vcc --version        (Daten werden unter usr/share/vintagecoder/data gefunden)
Installieren nach /usr/local: sudo cp -a usr/. /usr/local/
Handbuchseiten:              man -l usr/share/man/man1/vcc.1

Fuer die grafische Oberflaeche (usr/bin/vintagecoder) muessen die ueblichen Bibliotheken vorhanden sein (ALSA, xkbcommon-x11, GL/EGL,
Wayland, Xcursor, Xrandr, Xi). Bequemer sind die Pakete (.deb/.rpm) der Website, die diese Abhaengigkeiten melden.
"""


def make_tar(stage, path, version, machine, epoch=None):
    """tar.gz mit Oberordner vintagecoder-<version>-linux-<arch>; Inhalt: usr/, release-report.md (falls vorhanden), INSTALL.txt."""
    top = "vintagecoder-%s-linux-%s" % (tar_version(version), machine)
    mt = 0 if epoch is None else epoch

    def filt(ti):
        ti.uid = ti.gid = 0
        ti.uname = ti.gname = ""
        ti.mtime = mt
        return ti
    entries = []
    for d, dirs, files in os.walk(stage):
        dirs.sort()
        rel = os.path.relpath(d, stage).replace(os.sep, "/")
        entries.append((d, "" if rel == "." else rel))
        for f in sorted(files):
            p = os.path.join(d, f)
            entries.append((p, os.path.relpath(p, stage).replace(os.sep, "/")))
    with open(path, "wb") as raw:
        with gzip.GzipFile(filename="", mode="wb", fileobj=raw, mtime=0) as gz:
            with tarfile.open(fileobj=gz, mode="w", format=tarfile.GNU_FORMAT) as tf:
                for p, rel in entries:
                    tf.add(p, arcname=top + ("/" + rel if rel else ""), recursive=False, filter=filt)
                data = (INSTALL_TXT % (tar_version(version), machine)).encode("utf-8")
                ti = tarfile.TarInfo(top + "/INSTALL.txt")
                ti.size, ti.mtime, ti.mode = len(data), mt, 0o644
                tf.addfile(ti, io.BytesIO(data))
    return path


# ---------------------------------------------------------------- Pruefung der gebauten Pakete

def parse_dpkg_contents(text):
    """Ausgabe von `dpkg-deb -c`: Pfade der Dateien (keine Verzeichnisse, keine Verknuepfungen), absolut."""
    out = []
    for line in text.splitlines():
        parts = line.split(None, 5)
        if len(parts) < 6 or not parts[0].startswith("-"):
            continue
        name = parts[5].split(" -> ")[0]
        out.append("/" + name[2:] if name.startswith("./") else name if name.startswith("/") else "/" + name)
    return sorted(out)


def parse_rpm_list(text, dirs=()):
    """Ausgabe von `rpm -qlp`: Dateipfade ohne die Verzeichnisse (dirs: Menge besessener Verzeichnisse)."""
    return sorted(l.strip() for l in text.splitlines() if l.strip() and l.strip() not in set(dirs))


def verify_listing(expected, actual, what):
    """Vergleicht erwartete (aus dem Staging) und tatsaechliche Dateiliste eines Pakets; Fehlerliste."""
    e, a = set(expected), set(actual)
    errs = ["%s: fehlt im Paket: %s" % (what, p) for p in sorted(e - a)] + ["%s: fremd im Paket: %s" % (what, p) for p in sorted(a - e)]
    return errs


FORBIDDEN_NAMES = {"x64sc", "x64", "x128", "xplus4", "xvic", "xpet", "xcbm2", "xcbm5x0", "atari800", "altirra", "altirra64", "fujisan", "xmega65", "xemu"}


def forbidden_in(paths):
    """Pfade, die ROMs oder Emulator-Programme sein koennten (zusaetzlich zum Verbotsscan des Staging)."""
    bad = []
    for p in paths:
        base = re.sub(r"\.(exe|app|bin)$", "", os.path.basename(p).lower())
        if base in FORBIDDEN_NAMES or p.lower().endswith((".rom", ".crt")) and "/roms/" in p.lower():
            bad.append(p)
    return bad


# ---------------------------------------------------------------- Hauptfunktion

def package(stage, out, fmt, distro, version, arch=None, cfg=None, reproducible=False, machine=None):
    """Baut die Pakete; gibt eine Liste (Dateiname, Art) zurueck und schreibt neben jedes Paket <datei>.files (erwartete Dateiliste)."""
    cfg = cfg or load_config()
    stage = os.path.abspath(stage)
    out = os.path.abspath(out)
    machine = machine or (os.uname().machine if hasattr(os, "uname") else "x86_64")
    if fmt == "auto":
        if not distro:
            raise PackageError("--format auto braucht --distro")
        fmt = cfg["distros"][distro]["format"]
        formats = [fmt, "tar"]
    else:
        formats = [fmt]
    if any(f in ("deb", "rpm") for f in formats) and (not distro or distro not in cfg["distros"]):
        raise PackageError("--distro (%s) angeben" % ", ".join(DISTROS))
    for f in formats:
        if f in ("deb", "rpm") and cfg["distros"][distro]["format"] != f:
            raise PackageError("%s baut %s-Pakete, nicht %s" % (distro, cfg["distros"][distro]["format"], f))
    ide, links, _ = read_selection(stage)
    files = stage_files(stage)
    comps = split_components(files, ide)
    for w in ("vcc", "vc-lsp"):
        if "usr/bin/" + w not in files:
            raise PackageError("usr/bin/%s fehlt im Staging (stage mit --build oder --bin-dir)" % w)
    bad = forbidden_in(installed_paths(files))
    if bad:
        raise PackageError("Verbotene Dateien im Staging: " + ", ".join(bad))
    epoch = REPRODUCIBLE_EPOCH if reproducible else None
    os.makedirs(out, exist_ok=True)
    results = []
    for f in formats:
        if f == "tar":
            name = "vintagecoder-%s-linux-%s.tar.gz" % (tar_version(version), machine)
            make_tar(stage, os.path.join(out, name), version, machine, epoch)
            results.append((name, "tar"))
            continue
        for which, fl in comps.items():
            if f == "deb":
                fname, paths = build_deb(stage, fl, cfg, distro, which, version, links, deb_arch(arch or machine), out, epoch)
            else:
                fname, paths = build_rpm(stage, fl, cfg, distro, which, version, links, rpm_arch(arch or machine), out, epoch)
            with open(os.path.join(out, fname + ".files"), "w", encoding="utf-8", newline="\n") as lf:
                lf.write("\n".join(paths) + "\n")
            results.append((fname, f))
    sums_path = os.path.join(out, "SHA256SUMS")
    sums = {}
    if os.path.isfile(sums_path):
        with open(sums_path, encoding="utf-8") as sf:
            sums = {l.split("  ", 1)[1].strip(): l.split("  ", 1)[0] for l in sf if "  " in l}
    for name, _ in results:
        sums[name] = sha256_file(os.path.join(out, name))
    with open(sums_path, "w", encoding="utf-8", newline="\n") as sf:
        sf.write("".join("%s  %s\n" % (h, n) for n, h in sorted(sums.items())))
    return results
