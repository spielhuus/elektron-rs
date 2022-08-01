VENV = .venv
PYTHON = $(VENV)/bin/python3
PIP = $(VENV)/bin/pip

all: build

$(TARGET): $(VENV)/bin/activate 

$(VENV)/bin/activate: requirements.txt
	python3 -m venv $(VENV)
	$(PYTHON) -m pip install --upgrade pip
	$(PIP) install -r requirements.txt

clean:
	rm -rf build
	rm -rf $(VENV)

build: $(VENV)/bin/activate
	$(PYTHON) setup.py install

test: $(VENV)/bin/activate
	cargo test
