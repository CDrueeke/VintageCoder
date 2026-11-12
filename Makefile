# Linux-Pakete (.deb, .rpm, tar.gz) in Docker bauen und pruefen. Keine eigene Logik: nur tools/release.py und Docker (Konzept Release-Auswahl, 8.4).
#
#   make check                 FEATURES.md gegen Registry und Dateien pruefen
#   make plan                  zeigen, was ausgeliefert wuerde
#   make pkg-debian            Image bauen, Pakete bauen (Ergebnis in build/pakete/debian); ebenso pkg-ubuntu, pkg-fedora, pkg-opensuse
#   make test-debian           Pakete in einem frischen Container der Distribution installieren und starten (tools/linux-package-test.sh)
#   make pkg-all / test-all    alle vier Distributionen
#
# Optionen:  PLATFORM=linux/amd64 (Standard; verbindlich fuer Releases)   OUT=build/pakete   FEATURES=FEATURES.md
# Quellbaum: das Verzeichnis dieses Makefiles wird schreibgeschuetzt nach /src gemountet; unter Windows besser ein Linux-Dateisystem
# (WSL) benutzen, siehe tools/linux-build.sh und docs/release-auswahl.md.
DISTROS   := debian ubuntu fedora opensuse
PLATFORM  ?= linux/amd64
OUT       ?= $(CURDIR)/build/pakete
FEATURES  ?= FEATURES.md
BASE_debian   := debian:stable
BASE_ubuntu   := ubuntu:24.04
BASE_fedora   := fedora:latest
BASE_opensuse := opensuse/tumbleweed

.PHONY: check plan pkg-all test-all $(addprefix image-,$(DISTROS)) $(addprefix pkg-,$(DISTROS)) $(addprefix test-,$(DISTROS))

check: ; python3 tools/release.py check
plan:  ; python3 tools/release.py plan

pkg-all:  $(addprefix pkg-,$(DISTROS))
test-all: $(addprefix test-,$(DISTROS))

$(addprefix image-,$(DISTROS)): image-%:
	docker build --platform $(PLATFORM) -t vc-build-$* -f tools/docker/Dockerfile.$* tools/docker

$(addprefix pkg-,$(DISTROS)): pkg-%: image-%
	mkdir -p "$(OUT)/$*"
	docker run --rm --platform $(PLATFORM) \
	  -v "$(CURDIR)":/src:ro -v "$(OUT)/$*":/out \
	  -v vc-cargo-$*:/opt/cargo/registry -v vc-target-$*:/cache/target -e CARGO_TARGET_DIR=/cache/target -e FEATURES=$(FEATURES) \
	  vc-build-$* tools/linux-build.sh $* /out

$(addprefix test-,$(DISTROS)): test-%:
	docker run --rm --platform $(PLATFORM) \
	  -v "$(CURDIR)/tools":/tools:ro -v "$(OUT)/$*":/pakete:ro \
	  $(BASE_$*) bash /tools/linux-package-test.sh $* /pakete
