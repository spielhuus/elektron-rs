Python API
==========

.. automodule:: elektron

Draw a schematic
----------------

.. autoclass:: elektron.Draw
  :members:

.. autoclass:: elektron.Dot
  :members:

.. autoclass:: elektron.Label
  :members:

.. autoclass:: elektron.Element
  :members:

.. class:: elektron.Line
  
  The Line represents a wire.

  .. function:: line(len) 
  
  The line length.

  :param: len the line length in mm.

  .. function:: at() 

  Draw the line from the position.

  The position can either be a Pin or Dot:

  ::
    Line().at(“REF”, “PIN_NUMBER”)

    dot = Dot() Line().at(dot)

  .. function:: down() 

  Line direction down.


  .. function:: left() 

  Line direction left.


  .. function:: right() 

  Line direction right.


  .. function:: tox() 

  Draw the line to the X position.

  The position can either be a Pin or a Dot.

  ::code-block::

    Line().tox(“REF”, “PIN_NUMBER”)
    
    dot = Dot() 
    Line().tox(dot)


  .. function:: toy() 

  Draw the line to the Y position.

  The position can either be a Pin or a Dot.

  ::code-block::

    Line().toy(“REF”, “PIN_NUMBER”)
    
    dot = Dot() 
    Line().toy(dot)

  .. function:: up() 

  Line direction up.

.. autoclass:: elektron.Circuit
  :members:

.. autoclass:: elektron.Simulation
  :members:

