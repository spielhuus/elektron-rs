VERSION = 0.0.7
VENV = $(shell pwd)/.venv
PYTHON = $(VENV)/bin/python3
PIP = $(VENV)/bin/pip
MATURIN = $(VENV)/bin/maturin

$(TARGET): $(VENV)/bin/activate 
SOURCES = $(shell find src/**/src -name "*.rs") src/elektron/__init__.py

SPHINXOPTS    ?=
SPHINXBUILD   ?= ${VENV}/bin/sphinx-build
SOURCEDIR     = src/docs
BUILDDIR      = target/docs

PCBNEW := $(shell find /usr/lib/python**/ -name pcbnew.py)
PCBNEWSO := $(shell find /usr/lib/python**/ -name _pcbnew*.so)
PYVERSION := $(shell python3 --version|sed 's/.* \([0-9]\.[0-9]*\).*/\1/')

debug ?=

ifdef debug
  release :=develop
  target :=debug
  extension :=debug
else
  release :=install
  target :=release
  extension :=
endif

all: version build test doc ## run test, doc and build target

build: $(VENV)/bin/elektron ## build and local install.

version: 
	sed -i 's/^version = \".*\"$$/version = \"$(VERSION)\"/g' Cargo.toml
	sed -i 's/^version = \".*\"$$/version = \"$(VERSION)\"/g' pyproject.toml

$(VENV)/bin/activate: requirements.txt
	python3 -m venv $(VENV)
	$(PYTHON) -m pip install --upgrade pip
	$(PIP) install -r requirements.txt
	cd src/plotter && python src/css/generate.py build
	@[ -z "${PCBNEW}" ] && (echo "not linking pcbnew") || ln -s $(PCBNEW) $(VENV)/lib/python$(PYVERSION)/site-packages/pcbnew.py
	@[ -z "${PCBNEWSO}" ] && (echo "not linking pcbnew") || ln -s $(PCBNEWSO) $(VENV)/lib/python$(PYVERSION)/site-packages/_pcbnew.so

clean: ## remove all build files.
	cargo clean
	rm src/docs/_static/*.svg
	rm -rf $(VENV)
	rm -rf target

$(VENV)/bin/elektron: $(VENV)/bin/activate $(SOURCES)
	${MATURIN} develop

test: $(VENV)/bin/activate $(SOURCES) ## run all the test cases.
	cargo test --workspace

doc: $(VENV)/bin/activate $(SOURCES) ## create the rust and sphinx documentation.
	cargo doc --workspace --no-deps --lib
	$(PYTHON) src/docs/snippets.py
	$(SPHINXBUILD) "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) 

.PHONY: help

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

