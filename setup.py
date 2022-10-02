#!/usr/bin/env python
from setuptools import setup
from setuptools_rust import RustBin
from setuptools_rust import Binding, RustExtension
setup(
    name="elektron-python",
    version="0.1",
    packages=["elektron"],
    rust_extensions=[
        RustExtension("elektron.elektron"),
        RustBin("elektron"),
    ],
    zip_safe=False,
)
