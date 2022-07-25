from typing import Dict
import numpy as np
from IPython.display import Image
from elektron.elektron import Draw  as RDraw # , ElementType
from elektron.elektron import get_bom, schema_plot, schema_netlist, search


class LogicException(Exception):
    """Exception is raised when a wrong logic is applied."""


class PinNotFoundError(Exception):
    """Exception is thrown when a Pin can not be found."""


class DrawElement():
    """Abstract class for draw elements."""

    def __init__(self) -> None:
        self.angle: float = 0.0
        self.direction: str = "right"
        self.length: float = 2.54
        self.reference: str | None = None
        self.pin: int = 0
        self.xy_pos: DrawElement | None = None
        self.x_pos: DrawElement | None = None
        self.y_pos: DrawElement | None = None
        self.atref: str|None = None
        self.atpin: int|None = None

    def rotate(self, angle: float):
        self.angle = angle
        return self

    def up(self):
        self.direction = "up"
        return self

    def down(self):
        self.direction = "down"
        return self

    def left(self):
        self.direction = "left"
        return self

    def right(self):
        self.direction = "right"
        return self

    def len(self, length: float):
        self.length = length
        return self

    def at(self, reference: str, pin: int):
        self.atref = reference
        self.atpin = pin
        return self

    def xy(self, element: "DrawElement"):
        self.xy_pos = element.pos
        return self

    def tox(self, element: "DrawElement"):
        self.x_pos = element.pos[0]
        return self

    def toy(self, element: "DrawElement"):
        self.y_pos = element.pos[1]
        return self


class Line(DrawElement):
    """Place a connection on the schematic."""

    def __init__(self) -> None:
        super().__init__()

    def pt(self):
        if self.direction == "down":
            return np.array((0.0, self.length))
        if self.direction == "up":
            return np.array((0.0, -self.length))
        if self.direction == "left":
            return np.array((-self.length, 0.0))
        # right
        return np.array((self.length, 0))


class Dot(DrawElement):
    """Place a junction on the schematic."""

    def __init__(self):
        super().__init__()
        self.pos = np.array((0.0, 0.0))


class Label(DrawElement):
    """Place a connection on the schematic."""

    def __init__(self, name: str):
        super().__init__()
        self.name = name


class Element(DrawElement):
    """Place a connection on the schematic."""

    def __init__(self, ref: str, library: str, value: str, unit: int, **kwargs):
        super().__init__()
        self.ref = ref
        self.library = library
        self.value = value
        self.pin = 1
        self.unit = unit
        self.properties = kwargs

    def anchor(self, pin: int):
        self.pin = pin
        return self


class Draw():
    """Place a connection on the schematic."""

    def __init__(self):
        self.schema = RDraw(["/usr/share/kicad/symbols/"])
        self.pos = np.array((0.0, 0.0))

    def add(self, element: DrawElement):

        if isinstance(element, Line):
            if element.atref and element.atpin: # from pin
                self.pos = self.schema.pin_pos(element.atref, element.atpin)
            elif element.reference and element.pin: # from pin
                self.pos = self.schema.pin_pos(element.reference, element.pin)
                self.schema.wire(self.pos, self.pos + element.pt())
                self.pos += element.pt()
            elif element.xy_pos is not None: # from coordinates
                self.pos = element.xy_pos
                self.schema.wire(self.pos, self.pos + element.pt())
                self.pos += element.pt()
            elif element.x_pos is not None: # to x pos
                self.schema.wire(self.pos, np.array([element.x_pos, self.pos[1]]))
                self.pos = np.array([element.x_pos, self.pos[1]])
            else:
                self.schema.wire(self.pos, self.pos + element.pt())
                self.pos += element.pt()


        elif isinstance(element, Dot):
            self.schema.junction(self.pos)
            element.pos = self.pos.copy()
        elif isinstance(element, Label):
            self.schema.label(element.name, self.pos, element.angle)
        elif isinstance(element, Element):
            if element.atref and element.atpin: # from pin
                self.pos = self.schema.pin_pos(element.atref, element.atpin)
            elif element.reference and element.pin:
                self.pos = self.schema.pin_pos(element.reference, element.pin)
            elif element.xy_pos is not None:
                self.pos = element.xy_pos

            self.schema.symbol(element.ref, element.value, element.library,
                               element.unit, self.pos, element.pin, element.angle, "", element.x_pos, element.properties)  # TODO mirror

            if element.x_pos is not None:
                self.pos = np.array([element.x_pos, self.pos[1]])

    def write(self, filename: str | None):
        self.schema.write(filename)

    def plot(self, filename, border: bool, scale: float) -> str:
        return self.schema.plot(filename, border, scale)

    def circuit(self):
        return self.schema.circuit()
