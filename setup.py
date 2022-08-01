#!/usr/bin/env python
import sys

from setuptools import setup
from setuptools_rust import RustBin
from setuptools_rust import RustExtension

setup(
    rust_extensions=[
        RustExtension("elektron.elektron"),
        RustBin("elektron"),
    ],
    zip_safe=False,
)
