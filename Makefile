VENV = .venv
PYTHON = $(VENV)/bin/python3
PIP = $(VENV)/bin/pip

$(TARGET): $(VENV)/bin/activate 
SOURCES = $(shell find src -name "*.rs") elektron/__init__.py

SPHINXOPTS    ?=
SPHINXBUILD   ?= sphinx-build
SOURCEDIR     = docs
BUILDDIR      = target/docs

PCBNEW := $(shell find /usr/lib -name pcbnew.py)
PCBNEWSO := $(shell find /usr/lib -name _pcbnew*.so)
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
	python src/plot/css/generate.py build
	@[ -z "${PCBNEW}" ] && (echo "not linking pcbnew") || ln -s $(PCBNEW) $(VENV)/lib/python$(PYVERSION)/site-packages/pcbnew.py
	@[ -z "${PCBNEWSO}" ] && (echo "not linking pcbnew") || ln -s $(PCBNEWSO) $(VENV)/lib/python$(PYVERSION)/site-packages/_pcbnew.so

clean:
	cargo clean
	#python src/plot/css/generate.py clean
	rm -rf build
	rm -rf elektron.egg-info
	rm -rf $(VENV)
	rm -rf elektron_python.egg-info
	rm -rf elektron/elektron.cpython-310-x86_64-linux-gnu.so
	rm -rf elektron/__pycache__
	rm -rf dist

$(VENV)/bin/elektron: $(VENV)/bin/activate $(SOURCES)
	$(PYTHON) setup.py $(release)

test: $(VENV)/bin/activate $(SOURCES)
	cargo test

doc: $(VENV)/bin/activate $(SOURCES)
	cargo doc --lib --no-deps
	$(SPHINXBUILD) "$(SOURCEDIR)" "$(BUILDDIR)" $(SPHINXOPTS) 
	
sdist: $(VENV)/bin/activate
	$(PYTHON) setup.py sdist


