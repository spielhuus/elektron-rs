
from elektron import Draw, Element, Dot, Line, Label



draw = Draw(["/usr/share/kicad/symbols"])
draw + Element("R1", "Device:R", unit=1, value="100k").rotate(90)
draw + Element("C2", "Device:C", value="100n").rotate(90)
draw + Element("D1", "Device:LED", value="LED").rotate(180).anchor(2)
draw.plot(filename="src/docs/_static/draw1.svg", scale=2)

draw = Draw(["/usr/share/kicad/symbols"])
draw.pos((100.0, 100.0))
(draw
    + Label("INPUT").rotate(180)
    + Element("R1", "Device:R", value="10k").rotate(90)
    + Element("U1", "Amplifier_Operational:TL072", unit=1, 
              value="TL072", Spice_Primitive="X", Spice_Model="TL072c").mirror('x').anchor(2)
    + Line().up().at("U1", "1").length(5*2.54)
    + Element("R2", "Device:R", value="10k").tox("U1", "2").rotate(270)
    + Line().toy("U1", "2").dot("end")

    + Element("GND", "power:GND", unit=1, value="Gnd").at("U1", "3")

    + Dot().at("U1", "1") + Line()
    + Label("OUTPUT")
    
 )
draw.plot(filename="src/docs/_static/draw2.svg", scale=2)


## long tailed pair
draw = (Draw(["/usr/share/kicad/symbols"])
  + Label("X").rotate(180)
  + Element("Q1", "Transistor_BJT:BC547", unit=1, value="BC547",
                 Spice_Netlist_Enabled="Y",
                 Spice_Primitive="Q",
                 Spice_Model="BC547B").anchor(2)

  + Line().at("Q1", "1").up().length(5.08)
  + (dot_out_a := Dot())
  + Line().up().length(5.08)
  + Element("R1", "Device:R", unit=1, value="15k").rotate(180)
  + Line().up().length(5.08)
  + Element("+15V", "power:+15V", value="+15V")

  + Line().at("Q1", "3").down().length(5.08)
  + Line().right().length(10.16)
  + (dot1 := Dot())
  + Line().right().length(10.16)
  + Line().up().length(5.08)
  + Element("Q2", "Transistor_BJT:BC547", unit=1, value="BC547",
                 Spice_Netlist_Enabled="Y",
                 Spice_Primitive="Q",
                 Spice_Model="BC547B").anchor(3).mirror('x').rotate(180)

  + Line().at("Q2", "1").up().length(5.08)
  + (dot_out_b := Dot())
  + Line().up().length(5.08)
  + Element("R2", "Device:R", unit=1, value="15k").rotate(180)
  + Line().up().length(5.08)
  + Element("+15V", "power:+15V", value="+15V")

  + Element("GND", "power:GND", value="GND").at("Q2", "2")

  + Element("R3", "Device:R", unit=1, value="33k").at(dot1)
  + Line().down().length(2.54)
  + (dot2 := Dot())
  + Line().down().length(2.54)
  + Element("R4", "Device:R", unit=1, value="15k")
  + Element("-15V", "power:-15V", value="-15V").rotate(180)

  + Line().at(dot2).left().length(10.16)
  + Line().down().length(2.54)
  + Element("Q3", "Transistor_BJT:BC547", unit=1, value="BC547",
                 Spice_Netlist_Enabled="Y",
                 Spice_Primitive="Q",
                 Spice_Model="BC547B").anchor(3).mirror('x')

  + Element("GND", "power:GND", value="GND").at("Q3", "1")
  + Line().at("Q3", "2") 
  + Label("Y").rotate(180)

  + Line().at(dot_out_a).left().length(5.08)
  + Label("OUTa").rotate(180)

  + Line().at(dot_out_b).right().length(5.08)
  + Label("OUTb"))
draw.plot(filename="src/docs/_static/draw3.svg", scale=2)



