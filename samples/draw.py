from elektron import Line, Dot, Label, Element, Draw, Simulation
draw = Draw(["/usr/share/kicad/symbols"])
draw.add(Label("INPUT").rotate(180))
draw.add(Line())
draw.add(Element("R1", "Device:R", value="100k", unit=1, Spice_Netlist_Enabled="Y").rotate(90))
draw.add(Element("C1", "Device:C", value="47n", unit=1).at("R1", "2").rotate(90))
draw.add(Line().at("C1", "2"))
draw.add(u1_dot_in := Dot())
draw.add(Line())
draw.add(Element("U1", "4xxx:4069", value="4069", unit=1, Spice_Primitive="X", Spice_Model="4069UB"))
draw.add(Line().at("U1", "2"))
draw.add(u1_dot_out := Dot())
draw.add(Line())
draw.add(Element("C2", "Device:C", value="10u", unit=1).rotate(90))
draw.add(Line().at("C2", "2"))
draw.add(out_dot := Dot())
draw.add(Line())
draw.add(Label("OUTPUT"))

draw.add(Line().up().at(u1_dot_out).length(12.7))
draw.add(Element("R2", "Device:R", value="100k", unit=1).rotate(270).tox(u1_dot_in))
draw.add(Line().down().toy(u1_dot_in))

draw.add(Element("R3", "Device:R", value="100k", unit=1).at(out_dot).rotate(180))
draw.add(Element("GND", "power:GND", value="GND", unit=1).at("R3", "1"))

draw.add(Element("U1", "4xxx:4069", value="U1", unit=7, Spice_Primitive="X", Spice_Model="4069UB", on_schema="no"))
draw.add(Element("GND", "power:GND", value="GND", unit=1, on_schema="no").at("U1", "14"))
draw.add(Element("GND", "power:+5V", value="+5V", unit=1, on_schema="no").at("U1", "7"))
res = draw.plot("draw.svg", False, 5)

draw.write("draw.kicad_sch")

circuit = draw.circuit(["samples/files/spice"])
circuit.voltage("1", "+5V", "GND", "5V")
circuit.voltage("2", "INPUT", "GND", "5V SIN(0, 2.5, 100)")
circuit.save(None)

simulation = Simulation(circuit)
buffer = simulation.tran("0.02ms", "10ms", "0")

