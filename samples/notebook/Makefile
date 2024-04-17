VENV = ../.venv
PYTHON = $(VENV)/bin/python3
PIP = $(VENV)/bin/pip

.PHONY: help test doc clean pyre Makefile

MARKDOWN_FILES=$(shell find content/ -iname '*.rmd' | sed 's/ /\\ /g')
THEME=mono

all: site

debug ?=

ifdef debug
  DEBUG=ELEKTRON_DEBUG=true
endif

# $(TARGET): $(VENV)/bin/activate 

# $(VENV)/bin/activate: requirements.txt
# 	python3 -m venv $(VENV)
# 	$(PYTHON) -m pip install --upgrade pip
# 	$(PIP) install -r requirements.txt

clean:
	rm -rf $(VENV)
	rm -rf public
	rm -rf resources
	$(foreach var,$(MARKDOWN_FILES), rm -f $(subst .rmd,.md,$(var));)
	$(foreach var,$(MARKDOWN_FILES), rm -rf $(dir $(var))_files;)

build: $(MARKDOWN_FILES) #$(VENV)/bin/activate
	$(foreach var,$(MARKDOWN_FILES),$(DEBUG) ELEKTRON_SPICE=spice ELEKTRON_SYMBOLS=/usr/share/kicad/symbols:/home/etienne/elektrophon/lib/symbols ELEKTRON_THEME=$(THEME) MPLBACKEND='module://elektron' elektron convert --input $(var) --output $(subst .rmd,.md,$(var));)
	
site: build
	hugo

serve: build
	hugo serve

