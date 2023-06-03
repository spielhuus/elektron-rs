import sys

import logging
import matplotlib
matplotlib.use('module://matplotlib-backend-kitty')
import matplotlib.pyplot as plt

from elektron import Circuit, Draw, Element, Label, Line, Dot, Simulation

draw = (Draw(["/usr/share/kicad/symbols"])
    + Label("INPUT").rotate(180)
    + Line().length(2.54) + (in_dot := Dot())
    + Element("R2", "Device:R", value="180k", unit=1).rotate(90)
    + Element("C1", "Device:C", value="68n", unit=1).rotate(90)
    + (u1_dot_in := Dot())
    + Element("U1", "4xxx:4069", value="U1", unit=1,
              Spice_Netlist_Enabled='Y',
              Spice_Primitive='X',
              Spice_Model='4069UB',
              Spice_Node_Sequence="1 2 14 7").anchor(1)
    + (u1_dot_out := Dot())
    + Element("C3", "Device:C", value="33n").rotate(90)
    + (u2_dot_in := Dot())
    + Element("U2", "4xxx:4069", value="U1", unit=1,
              Spice_Netlist_Enabled='Y',
              Spice_Primitive='X',
              Spice_Model='4069UB',
              Spice_Node_Sequence="1 2 14 7").anchor(1)
    + (u2_dot_out := Dot())
    + Element("C5", "Device:C", value="10u").rotate(90)
    + (out_dot := Dot()) + Line().length(2.54)
    + Label("OUTPUT"))

draw + Element("R1", "Device:R", value="1Meg").at(in_dot) + Element("GND", "power:GND", value="GND")
draw + Element("R5", "Device:R", value="100k").at(out_dot) + Element("GND", "power:GND", value="GND")

(draw + Line().up().at(u1_dot_out).length(12.7)
     + (feedback_dot_1 := Dot())
     + Element("C2", "Device:C", value="81p").rotate(270).tox(u1_dot_in)
     + (feedback_dot_2 := Dot())
     + Line().down().toy(u1_dot_in)

     + Line().up().at(feedback_dot_1).length(5.07)
     + (dot_pot := Dot())
     + Line().up().length(5.07)
     + Element("RV1", "Device:R_Potentiometer", value="1Meg",
               Spice_Netlist_Enabled='Y',
               Spice_Primitive='X',
               Spice_Model='Potentiometer').anchor(3).rotate(90).mirror('x')
     + Element("R3", "Device:R", value="100k").rotate(270).at("RV1", "1") #.tox(feedback_dot_2)
     + Line().down().toy(feedback_dot_2)

     + Line().at("RV1", "2").toy(dot_pot)
     + Line().tox(dot_pot))

(draw + Line().up().at(u2_dot_out).length(12.7)
      + (feedback_dot_3 := Dot())
      + Element("C4", "Device:C", value="100p").rotate(270).tox(u2_dot_in)
      + (feedback_dot_4 := Dot())
      + Line().down().toy(u2_dot_in)

      + Line().up().at(feedback_dot_3).length(12.7)
      + Element("R4", "Device:R", value="1Meg").rotate(270).tox(feedback_dot_4)
      + Line().down().toy(feedback_dot_4))

(draw + Element("U1", "4xxx:4069", value="CD4069", unit=7, on_schema=False).at((50, 50))
      + Element("+5V", "power:+5V", value="+5V", on_schema=False).at("U1", "14")
      + Element("GND", "power:GND", value="GND", on_schema=False).at("U1", "7")

      + Element("U2", "4xxx:4069", value="CD409", unit=7, on_schema=False).at((70, 50))
      + Element("+5V", "power:+5V", value="+5V", on_schema=False).at("U2", "14")
      + Element("GND", "power:GND", value="GND", on_schema=False).at("U2", "7"))

# draw.write("4069.kicad_sch")
draw.plot(filename="4069.svg", scale=10, netlist=True, theme="Mono")

# circuit = draw.circuit(["files/spice"])
# circuit.voltage("1", "+5V", "GND", "DC 5V")
# circuit.voltage("2", "INPUT", "GND", "DC 5 AC 2.5V SIN(0 1V 1k)")

# # circuit.save("4069.spice")
# simulation = Simulation(circuit)
# vectors = simulation.tran("1us", "2ms", "0ms")

# fig, ax = plt.subplots(figsize=(8, 6))
# ax.plot(vectors['time'], vectors['input'])
# ax.plot(vectors['time'], vectors['output'])
# plt.show()
