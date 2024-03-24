![GitHub Tag](https://img.shields.io/github/v/tag/spielhuus/elektron-rs)
[![Deploy](https://github.com/spielhuus/elektron-rs/actions/workflows/CI.yml/badge.svg)](https://github.com/spielhuus/elektron-rs/actions/workflows/CI.yml)
[![Test](https://github.com/spielhuus/elektron-rs/actions/workflows/test.yml/badge.svg)](https://github.com/spielhuus/elektron-rs/actions/workflows/test.yml)
[![PyPI](https://img.shields.io/pypi/v/elektron-rs)](https://pypi.org/project/elektron-rs/)
[![Read the Docs](https://readthedocs.org/projects/elektron/badge/?version=latest)](https://elektron.readthedocs.io/en/latest/)

# elektron

The elektron package is a continuous integration and simulation tool for electronics projects.

Features
* Create output files from the command line
* Run erc and drc checks
* Create bom as json and Excel files.
* Programmatically create schemas with python code.
* Run spice simulation
* Convert markdown notebook files

## Installation

The elekron package can be installed locally or by using a [docker](https://github.com/spielhuus/elektron-docker) image.

Install the [Kicad](http://kicad.org) packages:

ubuntu 

```
apt-get install kicad kicad-symbols kicad-packages3d python3 python3-pip python3-venv
```

arch linux

```
pacman -Sy kicad kicad-library kicad-library-3d python python-pip
```
Install elektron from [PyPI](https://pypi.org/project/elektron-rs/)

```
python -m venv --system-site-packages .venv
pip install elektron-rs
```

The `--system-site-packages` option is needed to make elektron find the pcbnew packages.

Install the [osifont](https://github.com/hikikomori82/osifont).

```
mkdir -p /usr/local/share/fonts/TT/
curl -L "https://github.com/hikikomori82/osifont/blob/master/osifont-lgpl3fe.ttf?raw=true" -o /usr/local/share/fonts/TT/osifont-lgpl3fe.ttf
```

## Install from source

Install the required packages, note that Kicad and osifont are needed:

ubuntu 
```
apt-get install build-essential git cargo pkg-config libcairo2-dev libpango1.0-dev libngspice0-dev libpoppler-glib-dev libssl-dev libclang-14-dev
alias python='python3'
```

arch linux

```
pacman -Sy base-devel git clang python rustup graphite cairo pango ngspice poppler-glib
rustup default stable
```

Get and compile the code:

```
git clone https://github.com/spielhuus/elektron-rs
cd elektron-rs
make all
```

The `make` command will create the executable `elektron` in `.venv/bin`.

## Example usage

```
source .venv/bin/activate
elektron plot --input your_schema.kicad_sch --output schema.svg

```

