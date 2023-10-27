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

all: $(VENV)/bin/elektron

$(VENV)/bin/activate: requirements.txt
	python3 -m venv $(VENV)
	$(PYTHON) -m pip install --upgrade pip
	$(PIP) install -r requirements.txt
	cd src/plotter && python src/css/generate.py build
	@[ -z "${PCBNEW}" ] && (echo "not linking pcbnew") || ln -s $(PCBNEW) $(VENV)/lib/python$(PYVERSION)/site-packages/pcbnew.py
	@[ -z "${PCBNEWSO}" ] && (echo "not linking pcbnew") || ln -s $(PCBNEWSO) $(VENV)/lib/python$(PYVERSION)/site-packages/_pcbnew.so

clean:
	cd src/ngspice && cargo clean
	cd src/sexp && cargo clean
	cd src/sexp_macro && cargo clean
	cd src/simulation && cargo clean
	cd src/reports && cargo clean
	cd src/plotter && cargo clean
	cd src/draw && cargo clean
	cd src/notebook && cargo clean
	cargo clean
	rm -rf $(VENV)
	rm -rf src/elektron_rs.egg-info
	rm -rf src/elektron/elektron.cpython-311-x86_64-linux-gnu.so
	rm -rf src/elektron/__pycache__
	rm -rf build
	rm -rf target
	rm -rf dist

$(VENV)/bin/elektron: $(VENV)/bin/activate $(SOURCES)
	# $(PYTHON) -m pip install -e .
	${MATURIN} develop

test: $(VENV)/bin/activate $(SOURCES)
	cargo test --workspace

doc: $(VENV)/bin/activate $(SOURCES)
	cargo doc --workspace --no-deps --lib
	$(SPHINXBUILD) "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) 
	
# sdist: $(VENV)/bin/activate
# 	$(PYTHON) setup.py sdist
