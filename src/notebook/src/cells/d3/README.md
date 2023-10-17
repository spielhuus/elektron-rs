# Plot data with D3

This cell will create the data and parameters to plot datas using D3.

## Usage

First create some datas for example in a python block.


```
``````python
import numpy as np
t = np.arange(0., 5., 0.2)
output = t**3

data = {
    "time": t,
    "output": output,
  }
``````
```

and write them with the d3 cell: 

```
``````{d3, data="py$data", x="time", y="input, output", width="800", height="600"}``````
```

## parameters

### element: 
- The output element name

### data: 
- The data key.

### x: 
- The name of the x-axis.

### y: 
- The name(s) of the y-axis. When unset all axis are shown.

### yRange: 
- name of a range of values. The dataset must contain datas with the key name_N

### width: 
- width in pixels.

### height
- height in pixels.

### yType: 
- [scaleLinear, ScaleLog, ...] 
- Type of the axis, the names are the D3 methods. When set to scaleLog the range min can not be zero. 

### xType: 
- same as yType.

xDomain: [min, max] the x min and max domain.
yDomain: [min, max] the y min and max domain.

## TODO

- Do not copy values for y when the keys are selected.
- Let the range_x/y be settable
- rename range_x/y to the names in the template.
- check if axis names exist 

## Links
