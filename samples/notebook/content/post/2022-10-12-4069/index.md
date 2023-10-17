---
title: "test circuit"
summary: "draw a circuit and plot the simulation."
---

# Create circuit

This will create the data and parameters to plot datas using D3.

## Usage

First create some datas for example in a python block.

{{< figure align="center" cap="Figure 4: Linear amplifier" path="_files/qmpaninlqxgbnslvvymopnaqhrvdmb.svg">}}
And plot the results using D3:

{{< d3 key="buffer" x="time" y="input" yRange="output" ySize="40" xDomain="0, 0.01" yDomain="-2.499999903682053, 2.897406367761909" width="600" height="400" yType="scaleLinear" xType="scaleLinear" colors="Red,Green" xLabel="" yLabel="" range="" align="center" cap="Figure 5: linear amplifier simulation">}}
{{< /d3 >}}
