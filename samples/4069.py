import sys

import logging
import matplotlib
matplotlib.use('module://matplotlib-backend-kitty')
import matplotlib.pyplot as plt

sys.path.append('src')
sys.path.append('../src')

#from elektron import Draw, Dot, Label, Line, Element, Schema
from elektron import Schema, Wire

# initialize the logger
logging.basicConfig(format='%(levelname)s:%(message)s', encoding='utf-8', level=logging.ERROR)
logging.getLogger().setLevel(logging.ERROR)

vectors = []
#writer = nl.SexpWriter()
#circuit = nl.Circuit()

schema = Schema()
wire = Wire()



schem_fig, schem_ax = plt.subplots(figsize=(8, 6))
plot = nl.SchemaPlot(schem_ax, 297, 210, 600, child=circuit)
with nl.draw(plot) as draw:
    draw.add(Label("INPUT").rotate(180))
    draw.add(Line())
    draw.add(in_dot := Dot())
    draw.add(Line())
    draw.add(Element("R2", "Device:R", value="100k").rotate(90))
    draw.add(Element("C1", "Device:C", value="68n").rotate(90))
    draw.add(Line())
    draw.add(u1_dot_in := Dot())
    draw.add(Line())
    draw.add(Element("U1", "4xxx:4069", value="U1", unit=1,
                 Spice_Netlist_Enabled='Y',
                 Spice_Primitive='X',
                 Spice_Model='4069UB').anchor(1))
    draw.add(Line())
    draw.add(u1_dot_out := Dot())
    draw.add(Element("C3", "Device:C", value="33n").rotate(90))
    draw.add(u2_dot_in := Dot())
    draw.add(Element("U2", "4xxx:4069", value="U1", unit=1,
                 Spice_Netlist_Enabled='Y',
                 Spice_Primitive='X',
                 Spice_Model='4069UB').anchor(1))
    draw.add(u2_dot_out := Dot())
    draw.add(Element("C5", "Device:C", value="10u").rotate(90))
    draw.add(Line())
    draw.add(out_dot := Dot())
    draw.add(Line())
    draw.add(Label("OUTPUT"))

    draw.add(Element("R1", "Device:R", value="1Meg").at(in_dot))
    draw.add(Element("GND", "power:GND"))

    draw.add(Element("R5", "Device:R", value="100k").at(out_dot))
    draw.add(Element("GND", "power:GND"))

    draw.add(Line().up().at(u1_dot_out).length(draw.unit*4))
    draw.add(feedback_dot_1 := Dot())
    draw.add(Element("C2", "Device:C", value="81p").rotate(270).tox(u1_dot_in))
    draw.add(feedback_dot_2 := Dot())
    draw.add(Line().down().toy(u1_dot_in))

    draw.add(Line().up().at(feedback_dot_1).length(draw.unit*2))
    draw.add(dot_pot := Dot())
    draw.add(Line().up().length(draw.unit*2))
    draw.add(Element("RV1", "Device:R_Potentiometer", value="1Meg",
                 Spice_Netlist_Enabled='Y',
                 Spice_Primitive='X',
                 Spice_Model='Potentiometer').anchor(3).rotate(90).mirror('x'))
    draw.add(Element("R3", "Device:R", value="100k").rotate(270).at(nl.pins(draw.RV1[0])['1']).tox(feedback_dot_2))
    draw.add(Line().down().toy(feedback_dot_2))

    draw.add(Line().toy(dot_pot).at(nl.pins(draw.RV1[0])['2']))
    draw.add(Line().tox(dot_pot))

    draw.add(Line().up().at(u2_dot_out).length(draw.unit*4))
    draw.add(feedback_dot_3 := Dot())
    draw.add(Element("C4", "Device:C", value="100p").rotate(270).tox(u2_dot_in))
    draw.add(feedback_dot_4 := Dot())
    draw.add(Line().down().toy(u2_dot_in))

    draw.add(Line().up().at(feedback_dot_3).length(draw.unit*4))
    draw.add(Element("R4", "Device:R", value="1Meg").rotate(270).tox(feedback_dot_4))
    draw.add(Line().down().toy(feedback_dot_4))

    draw.add(Element("U1", "4xxx:4069", unit=7, on_schema=False).at((102, 50)))
    draw.add(Element("+5V", "power:+5V", on_schema=False).at(nl.pins(draw.U1[1])['14']))
    draw.add(Element("GND", "power:GND", on_schema=False).at(nl.pins(draw.U1[1])['7']).rotate(180))

    draw.add(Element("U2", "4xxx:4069", unit=7, on_schema=False).at((110, 50)))
    draw.add(Element("+5V", "power:+5V", on_schema=False).at(nl.pins(draw.U2[1])['14']))
    draw.add(Element("GND", "power:GND", on_schema=False).at(nl.pins(draw.U2[1])['7']).rotate(180))

pot = Potentiometer("Potentiometer", 100000000, 1)
circuit.subcircuit(pot)
circuit.V("1", "+5V", "GND", "DC 5V")
circuit.V("2", "INPUT", "GND", "DC 5 AC 2.5V SIN(0 1V 1k)")
print(circuit)

with nl.spice(circuit) as spice:
    #for s in np.arange( 1, 0, -0.01 ):
    pot.wiper(0.2)
    vectors = spice.transient('10u', '10m', '0m')

plt.show()

fig, ax = plt.subplots(figsize=(8, 6))
ax.plot(vectors['time']*1000, vectors['input'])
ax.plot(vectors['time']*1000, vectors['output'])
plt.show()
