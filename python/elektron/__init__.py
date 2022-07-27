from typing import Dict
import numpy as np
from IPython.display import display, SVG, Image
from elektron.elektron import Draw  as RDraw # , ElementType
from elektron.elektron import get_bom, schema_plot, schema_netlist, search

from base64 import standard_b64encode
import sys
def serialize_gr_command(**cmd):
    payload = cmd.pop('payload', None)
    cmd = ','.join(f'{k}={v}' for k, v in cmd.items())
    ans = []
    w = ans.append
    w(b'\033_G'), w(cmd.encode('ascii'))
    if payload:
        w(b';')
        w(payload)
    w(b'\033\\')
    return b''.join(ans)
def write_chunked(**cmd):
    data = standard_b64encode(cmd.pop('data'))
    while data:
        chunk, data = data[:4096], data[4096:]
        m = 1 if data else 0
        sys.stdout.buffer.write(serialize_gr_command(payload=chunk, m=m,
                                                    **cmd))
        sys.stdout.flush()
        cmd.clear()

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

    def at(self, reference: str, pin: int=0):
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

    def pt(self, pos):
        if self.y_pos is not None:
            return np.array((pos[0], self.y_pos))
        if self.x_pos is not None:
            return np.array((self.x_pos, pos[1]))
        if self.xy_pos is not None:
            return self.xy_pos

        if self.direction == "down":
            return pos + np.array((0.0, self.length))
        if self.direction == "up":
            return pos + np.array((0.0, -self.length))
        if self.direction == "left":
            return pos + np.array((-self.length, 0.0))
        # right
        return pos + np.array((self.length, 0))


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
        self.mirror_axis = ""

    def anchor(self, pin: int):
        self.pin = pin
        return self

    def mirror(self, axis: str):
        self.mirror_axis = axis
        return self

class Draw():
    """Place a connection on the schematic."""

    def __init__(self):
        self.schema = RDraw(["/usr/share/kicad/symbols/"])
        self.pos = np.array((0.0, 0.0))

    def add(self, element: DrawElement):

        if isinstance(element, Line):
            if element.atref and element.atpin is not None: # from pin
                if isinstance(element.atref, Dot):
                    self.pos = element.atref.pos
                else:
                    self.pos = self.schema.pin_pos(element.atref, element.atpin)
                
                self.schema.wire(self.pos, element.pt(self.pos))
                self.pos = element.pt(self.pos)

            elif element.reference and element.pin: # from pin
                self.pos = self.schema.pin_pos(element.reference, element.pin)
                self.schema.wire(self.pos, element.pt(self.pos))
                self.pos = element.pt(self.pos)
            else:
                self.schema.wire(self.pos, element.pt(self.pos))
                self.pos = element.pt(self.pos)


        elif isinstance(element, Dot):
            self.schema.junction(self.pos)
            element.pos = self.pos.copy()
        elif isinstance(element, Label):
            self.schema.label(element.name, self.pos, element.angle)
        elif isinstance(element, Element):
            if element.atref and element.atpin is not None: # from pin
                if isinstance(element.atref, Dot):
                    self.pos = element.atref.pos
                else:
                    self.pos = self.schema.pin_pos(element.atref, element.atpin)

            elif element.reference and element.pin:
                self.pos = self.schema.pin_pos(element.reference, element.pin)
            elif element.xy_pos is not None:
                self.pos = element.xy_pos

            self.schema.symbol(element.ref, element.value, element.library,
                               element.unit, self.pos, element.pin, element.angle,
                               element.mirror_axis, element.x_pos, element.properties)

            if element.x_pos is not None:
                self.pos = np.array([element.x_pos, self.pos[1]])

    def write(self, filename: str | None):

        self.schema.write(filename)

    def plot(self, filename, border: bool, scale: float):
        if filename is None and sys.stdout.isatty():
            print("called from tty")
            image = self.schema.plot(filename, border, scale, "png")
            write_chunked(a='T', f=100, data=bytearray(image))
        elif filename is None:
            image = self.schema.plot(filename, border, scale, "png")
            return bytearray(image)
        else:
            filetype = ""
            if filename.endswith(".png"):
                filetype = "png"
            elif filename.endswith(".svg"):
                filetype = "svg"
            elif filename.endswith(".pdf"):
                filetype = "pdf"
            else:
                raise TypeError(f"filetype not found {filename}")
            self.schema.plot(filename, border, scale, filetype)

        return self

    def _repr_svg_(self):
        ''' SVG representation for Jupyter '''
        print("call repr svg")
        return bytearray(self.schema.plot(filename, border, scale, "svg"))

    def _repr_png_(self):
        ''' PNG representation for Jupyter '''
        print("call repr")
        return bytearray(self.schema.plot(filename, border, scale, "png"))
    def _repr_(self):
        ''' PNG representation for Jupyter '''
        print("call repr png")
        return bytearray(self.schema.plot(filename, border, scale, "png"))

    def circuit(self):
        return self.schema.circuit()

