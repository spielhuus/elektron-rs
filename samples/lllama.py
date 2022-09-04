from elektron import Line, Dot, Label, Element, Draw, Simulation, Circuit
draw = Draw(["/usr/share/kicad/symbols"])
draw.add(Label("INPUT").rotate(180))
draw.add(Line())
draw.add(in_dot := Dot())
draw.add(Line())
draw.add(Element("R2", "Device:R", value="100k", unit=1).rotate(90))
draw.add(Element("C1", "Device:C", value="68n", unit=1).at("R2", "2").rotate(90))
draw.add(Line().at("C1", "2"))
draw.add(u1_dot_in := Dot())
draw.add(Line())
draw.add(Element("U1", "4xxx:4069", value="U1", unit=1,
                 Spice_Netlist_Enabled='Y',
                 Spice_Primitive='X',
                 Spice_Model='4069UB').anchor(1))
draw.add(Line().at("U1", "2"))
draw.add(u1_dot_out := Dot())
draw.add(Element("C3", "Device:C", value="33n", unit=1).rotate(90))
draw.add(Line().at("C3", "2").right())
draw.add(u2_dot_in := Dot().at("C3", "2"))
draw.add(Element("U2", "4xxx:4069", value="U1", unit=1,
                 Spice_Netlist_Enabled='Y',
                 Spice_Primitive='X',
                 Spice_Model='4069UB').anchor(1))
draw.add(Line().at("U2", "2"))
draw.add(u2_dot_out := Dot())
draw.add(Element("C5", "Device:C", value="10u", unit=1).rotate(90))
draw.add(Line().at("C5", "2"))
draw.add(out_dot := Dot())
draw.add(Line())
draw.add(Label("OUTPUT"))

draw.add(Element("R1", "Device:R", value="1Meg", unit=1).anchor(2).at(in_dot))
draw.add(Element("GND", "power:GND", value="GND", unit=1).at("R1", "2"))

draw.add(Element("R5", "Device:R", value="100k", unit=1).anchor(2).at(out_dot))
draw.add(Element("GND", "power:GND", value="GND", unit=1).at("R5", "2"))

draw.add(Line().up().length(12.7).at(u1_dot_out))
draw.add(feedback_dot_1 := Dot())
draw.add(Element("C2", "Device:C", value="81p", unit=1).rotate(270).tox(u1_dot_in))
draw.add(feedback_dot_2 := Dot())
draw.add(Line().down().toy(u1_dot_in))

draw.add(Line().up().at(feedback_dot_1).length(5.08))
draw.add(dot_pot := Dot())
draw.add(Line().up().length(5.08))
draw.add(Element("RV1", "Device:R_Potentiometer", value="100k", unit=1,
                 Spice_Netlist_Enabled='Y',
                 Spice_Primitive='X',
                 Spice_Model='Potentiometer').anchor(3).rotate(90).mirror('x'))
draw.add(Element("R3", "Device:R", value="100k", unit=1)
        .rotate(270).at("RV1", "1").tox(feedback_dot_2))
draw.add(Line().down().toy(feedback_dot_2))

draw.add(Line().toy(dot_pot).at("RV1", "2"))
draw.add(Line().tox(dot_pot))

draw.add(Line().up().at(u2_dot_out).length(12.7))
draw.add(feedback_dot_3 := Dot())
draw.add(Element("C4", "Device:C", value="100p", unit=1).rotate(270).tox(u2_dot_in))
draw.add(feedback_dot_4 := Dot())
draw.add(Line().down().toy(u2_dot_in))

draw.add(Line().up().at(feedback_dot_3).length(10.16))
draw.add(Element("R4", "Device:R", value="1Meg", unit=1).rotate(270).tox(feedback_dot_4))
draw.add(Line().down().toy(feedback_dot_4))


draw.add(Element("U1", "4xxx:4069", value="U1", unit=7, Spice_Primitive="X", Spice_Model="4069UB", on_schema="no"))
draw.add(Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U1", "14"))
draw.add(Element("GND", "power:+5V", value="+5V", unit=1, on_schema="no").at("U1", "7"))

draw.add(Element("U2", "4xxx:4069", value="U1", unit=7, Spice_Primitive="X", Spice_Model="4069UB", on_schema="no"))
draw.add(Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U2", "14"))
draw.add(Element("GND", "power:+5V", value="+5V", unit=1, on_schema="no").at("U2", "7"))

draw.write("llama.kicad_sch")
res = draw.plot(None, False, 3)
print(res)

circuit = draw.circuit(['/home/etienne/elektron/samples/files/spice/'])
circuit.voltage("1", "+5V", "GND", "5V")
circuit.voltage("2", "INPUT", "GND", "5V SIN(0, 2.5, 100)")
pot = Circuit("Potentiometer", ['/home/etienne/elektron/samples/files/spice/'])
pot.resistor("R1", "n0", "n1", "100k")
pot.resistor("R2", "n1", "n2", "100k")
circuit.subcircuit("Potentiometer", ["n0", "n1", "n2"], pot)

circuit.save(None)
simulation = Simulation(circuit)
simulation.tran("1us", "10ms", "0")
# circuit.plot("output", "draw_output.svg")

