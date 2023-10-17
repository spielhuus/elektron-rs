---
title: "svf circuit"
summary: "draw a filter circuit and do frequency analysis."
---

# Create circuit

This will create the data and parameters to plot datas using D3.

## Usage

First create some datas for example in a python block.

```python
from elektron import Circuit, Draw, Element, Label, Line, Dot, Simulation
import numpy as np

draw = (Draw(["/usr/share/kicad/symbols"])
  + Label("INPUT").rotate(180)
  + Element("R1", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(90)
  + Element("C1", "Device:C", value="220n", Spice_Netlist_Enabled="Y").rotate(90)
  + (u1_dot_in := Dot())
  + Element("U1", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u1_dot_out := Dot())

  + Element("R3", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(90)
  + (u2_dot_in := Dot())
  + Element("U2", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u2_dot_out := Dot())

  + Element("R5", "Device:R", value="10k", Spice_Netlist_Enabled="Y").rotate(90)
  + (u3_dot_in := Dot())
  + Element("U3", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u3_dot_out := Dot())

  + Element ("R6", "Device:R", value="10k", Spice_Netlist_Enabled="Y").rotate(90)
  + (u4_dot_in := Dot())
  + Element("U4", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7")
  + (u4_dot_out := Dot())

  + Line().up().length(12.7).at(u1_dot_out)
  + Element("R2", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(u1_dot_in)
  + (r2_out := Dot())
  + Line().down().toy(u1_dot_in)

  + Line().up().length(12.7).at(u2_dot_out)
  + Label("HP")
  + Element("R4", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(u2_dot_in)
  + (r4_out := Dot())
  + Line().down().toy(u2_dot_in)

  + Line().up().length(12.7).at(u3_dot_out)
  + Label("BP")
  + (bp := Dot())
  + Element("C3", "Device:C", value="10n", Spice_Netlist_Enabled="Y").rotate(270).tox(u3_dot_in)
  + Line().down().toy(u3_dot_in)

  + Line().up().length(12.7).at(u4_dot_out)
  + Label("LP")
  + (lp := Dot())
  + Element("C4", "Device:C", value="10n", Spice_Netlist_Enabled="Y").rotate(270).tox(u4_dot_in)
  + Line().down().toy(u4_dot_in)

  + Line().up().length(10.16).at(lp)
  + Element("R7", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(r4_out)
  + Line().down().toy(r4_out)

  + Line().up().length(20.32).at(bp)
  + Element("R8", "Device:R", value="100k", Spice_Netlist_Enabled="Y").rotate(270).tox(r2_out)
  + Line().down().toy(r2_out)

  + Element("U1", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((50.8, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U1", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U1", "14")

  + Element("U2", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((71.12, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U2", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U2", "14")

  + Element("U3", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((91.44, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U3", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U3", "14")

  + Element("U4", "4xxx:4069", value="4069", unit=7, Spice_Model="4069UB", Spice_Node_Sequence="1 2 14 7", on_schema="no").at((111.76, 50.8))
  + Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U4", "7")
  + Element("+5V", "power:+5V", value="+5V", unit=1, on_schema="no").at("U4", "14"))

print("get circuit")
draw.write("svf.kicad_sch")
circuit = draw.circuit(["spice"])
circuit.voltage("1", "+5V", "GND", "DC 15V")
circuit.voltage("2", "INPUT", "GND", "AC 2V SIN(0 2V 1k)")
circuit.control('''
ac dec 10 100 10K

*let r_act = 1k
*let r_step = 2k
*let r_stop = 200k
*while r_act le r_stop
*  alter R7 r_act
*  alter R8 r_act
*  ac dec 10 100 10K
*  let r_act = r_act + r_step
*end
*tran 1us 10ms
''')

print("get simulation")
simulation = Simulation(circuit)
svf_data = simulation.tran("1us", "10ms", "0ms")
print(len(svf_data))
svf = simulation.run()

for key, value in svf.items():
  if key.startswith("ac"):
    for k, v in value.items():
      if k == "frequency":
        svf[key][k] = v[1:]
      else:
        svf[key][k] = 20*np.log10(np.absolute(v))[1:]

draw.plot(scale=6)
```
{{< figure cap="Figure 6: State variable filter" align="center" path="_files/jsoupkvkqaxzetqgteyrjxnnsqxarx.svg">}}

This is the first setup with the 4069 as a voltage follower. C1 and C2 are DC blocking capacitors. When we choose R1 and R2 as 100kOhm we would expect a gain of one.

{{< d3 key="svf_ac" x="frequency" y="bp,hp,lp" yRange="" ySize="0" xDomain="125.89254117941672, 10000.000000000007" yDomain="-48.44932852660952, 6.115253317994574" width="600" height="400" yType="scaleLinear" xType="scaleLog" colors="Red,Green,Blue" xLabel="" yLabel="" range="" align="center" cap="Figure 7: State variable filter simulation">}}
{{< /d3 >}}

