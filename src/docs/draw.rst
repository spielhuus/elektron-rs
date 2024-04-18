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

Now we want to try to draw a circuit that makes a little bit more sense. We draw an inverting opamp amplifier.

.. code-block:: python

   draw = Draw(["/usr/share/kicad/symbols"])
   (draw
       + Label("INPUT").rotate(180)
       + Element("R1", "Device:R", value="10k").rotate(90)
       + Element("U1", "Amplifier_Operational:TL072", unit=1, 
                 value="TL072", Spice_Primitive="X", Spice_Model="TL072c").mirror('x').anchor(2)
       + Line().up().at("U1", "1").length(5*2.54)
       + Element("R2", "Device:R", value="10k").tox("U1", "2").rotate(270)
       + Line().toy("U1", "2")

       + Element("GND", "power:GND", unit=1, value="Gnd").at("U1", "3")

       + Dot().at("U1", "1") + Line()
       + Label("OUTPUT")
   )

.. figure:: /_static/draw2.svg
   :alt: alternate text
   :align: center

   inverting amplifier




