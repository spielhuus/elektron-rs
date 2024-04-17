Draw Circuits
=============

The elektron module allows for drawing circuits. The drawing follows the basic  contains Basic Elements pre-defined for use in a drawing. A common import structure is:

.. code-block:: markdown

   from elektron import Draw, Element, Label, Line, Nc, Dot, C, R, Gnd, Feedback, Power

To make a circuit diagram, use a context manager (with statement) on a schemdraw.Drawing. Then any schemdraw.elements.Element instances created within the with block added to the drawing:

.. code-block:: python

    draw = Draw(["/usr/share/kicad/symbols"])
    draw + Element("R1", "Device:R", unit=1, value="100k").rotate(90)
    draw + Element("C2", "Device:C", value="100n").rotate(90)
    draw + Element("D1", "Device:LED", value="LED").rotate(180).anchor(2)
    draw.plot(filename="schematic.svg", scale=2)

.. figure:: /_static/draw1.svg
   :alt: alternate text
   :align: center

   first useless circuit



