========
elektron
========

With elektron, users can seamlessly craft electronic schematics using Python code, simulate circuits for testing and validation, and export them to `KiCad`_ for further refinement or PCB layout. Moreover, elektron simplifies the production process by generating production files directly from `KiCad`_. To enhance workflow management, elektron enables users to create notebooks for organizing and documenting their designs, which can be effortlessly transformed into web pages for easy sharing and collaboration.

Features: 

* Draw schematic diagrams with python code.
* Export diagrams to kicad
* Export diagrams to a spice netlist
* Run ngspice simulation
* Plot schema and pcb from kicad files.
* Output BOM in JSON or Excel file format. The Excel file can be used for import to mouser.
* Run ERC and DRC checks for kicad files.
* Convert markdown notebooks and execute commands.


Why consider another program when there are already several excellent options available? For instance, `schemdraw`_ offers a pleasant interface for drawing schematics, while `PySpice`_ facilitates circuit simulation. Additionally, numerous projects support working with `KiCad`_ production files. However, despite these offerings, workflow integration often remains cumbersome. Here's where elektron steps in. Built around the `KiCad`_ data model, elektron streamlines the process. Now, the circuit snippet created within elektron's notebook can seamlessly transition to simulation and export within `KiCad`_.

Indices and tables
==================

* :ref:`genindex`
* :ref:`modindex`
* :ref:`search`

.. toctree::
   :hidden:
   :includehidden:

   installation
   cli
   draw
   simulation
   notebook
   python

.. _KiCad: https://kicad.org
.. _schemdraw: https://github.com/RonSheely/schemdraw?tab=readme-ov-file
.. _PySpice: https://github.com/PySpice-org/PySpice
