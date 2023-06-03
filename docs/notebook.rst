Notebook
========

-----------
Python Cell
-----------

Run any python code in a cell. The Python variables will be available to other cells or in any text.

::

  ```python
  myvar = "hello"
  ```

The variable can be accessed with ``${myvar}``.

Options
-------

- **echo** ``TRUE|FALSE`` Output the code cell.
- **results** ``hide`` Do not show the stdout outputs.

-----------------
Plot data with D3
-----------------

This cell will create the data and parameters to plot datas using D3.

First create some data for example in a python block.

::

  import numpy as np
  t = np.arange(0., 5., 0.2)
  output = t**3

  data = {
      "time": t,
      "output": output,
    }

and write them with the d3 cell: 

::

  ```{d3, data="py$data", x="time", y="input, output", width="800", height="600"}```

Options
-------

- **element** ``<str>`` The HTML element identification.
- **data** ``<str>`` The data Key, data used from python code can be accessed with ``py$VAR`` data from simulations with ``VAR.tran1``.
- **x** ``<str>`` Name of the x-axis data.
- **y** ``<str>`` Name of the y-axis data.
- **yRange** ``<str>`` name of a range of values. The dataset must contain data with the key name_N.
- **width** ``<int>`` Width in pixels.
- **heigt** ``<int>`` Height in pixels.
- **yType** ``scaleLonear, scaleLog ...`` Type of the axis, the names are the D3 methods. When set to scaleLog the range min can not be zero.
- **xDomain** ``<int, int>`` The x min and max domain. [min, max].
- **yDomain** ``<int, int>`` The y min and max domain. [min, max].

TODO
----

- Do not copy values for y when the keys are selected.
- Let the range_x/y be settable
- rename range_x/y to the names in the template.
- check if axis names exist 

## Links
