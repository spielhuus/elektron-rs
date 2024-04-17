---
title: "plot"
Summary: "plot data with d3"
---

This will create the data and parameters to plot datas using D3.

## Usage

First create some datas for example in a python block.

```python
import numpy as np
t = np.arange(0., 5., 0.2)
output = t**3

data_test = {
    "time": t,
    "output": output,
  }
```

and write them with the d3 cell: 
{{< d3 key="data_test" x="time" y="output" yRange="" ySize="-2" xDomain="0, 4.800000000000001" yDomain="0, 110.59200000000006" width="800" height="600" yType="scaleLinear" xType="scaleLinear" colors="Red" xLabel="" yLabel="" range="" >}}
{{< /d3 >}}
