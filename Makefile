VENV = .venv
PYTHON = $(VENV)/bin/python3
PIP = $(VENV)/bin/pip

$(TARGET): $(VENV)/bin/activate 
SOURCES = $(shell find src -name "*.rs") elektron/__init__.py

debug ?=

$(info debug is $(debug))

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

clean:
	rm -rf build
	rm -rf elektron.egg-info
	rm -rf target
	rm -rf $(VENV)
	rm -rf elektron/elektron.cpython-310-x86_64-linux-gnu.so
	rm -rf elektron/__pycache__

$(VENV)/bin/elektron: $(VENV)/bin/activate $(SOURCES)
	$(PYTHON) setup.py $(release)

test: $(VENV)/bin/activate
	cargo test

doc: $(VENV)/bin/activate
	cargo doc --no-deps
