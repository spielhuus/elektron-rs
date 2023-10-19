"""
This is the "example" module.

The example module supplies one function, factorial().  For example,

>>> factorial(5)
120
"""

from .elektron import PyDraw
from .elektron import Line, Dot, Label, Element, Nc, C, R, Gnd, Power, Feedback, Simulation, Circuit
import sys, os
import tempfile
import pcbnew

import argparse
from .elektron import make_bom, plot, convert, make_erc, make_drc, make_spice, search

def main():
    # create the top-level parser
    parser = argparse.ArgumentParser(prog='elektron')
    sub_parsers = parser.add_subparsers(help='sub-command help', dest="command")

    # create the sub parsers
    parser_bom = sub_parsers.add_parser('bom', help='generate bom from kicad schema')
    parser_bom.add_argument('--input', type=str, required=True, help='path to kicad schema')
    parser_bom.add_argument('--output', type=str, required=False, help='output as json file')
    parser_bom.add_argument('--group', action='store_true', help='group symbols')
    parser_bom.add_argument('--partlist', type=str, required=False, help='path to partlist file')

    parser_plot = sub_parsers.add_parser('plot', help='plot kicad schema or PCB')
    parser_plot.add_argument('--input', type=str, required=True, help='path to kicad schema')
    parser_plot.add_argument('--output', type=str, required=False, help='output as json file')

    parser_convert = sub_parsers.add_parser('convert', help='Convert a markdown notebook file.')
    parser_convert.add_argument('--input', type=str, required=True, help='path to input markdown file.')
    parser_convert.add_argument('--output', type=str, required=False, help='path to the output markdown file.')

    parser_erc = sub_parsers.add_parser('erc', help='Run the ERC checks for a schema file.')
    parser_erc.add_argument('--input', type=str, required=True, help='path to input schema file.')
    parser_erc.add_argument('--output', type=str, required=False, help='the result from the checks')
    
    parser_drc = sub_parsers.add_parser('drc', help='Run the the DRC checks for a PCB file.')
    parser_drc.add_argument('--input', type=str, required=True, help='path to input PCB file.')
    parser_drc.add_argument('--output', type=str, required=False, help='the result from the checks')
    
    parser_spice = sub_parsers.add_parser('spice', help='Create the spice netlist for the schema.')
    parser_spice.add_argument('--input', type=str, required=True, help='path to input Schema file.')
    parser_spice.add_argument('--output', type=str, required=False, help='the result from the checks, default print to console.')
    parser_spice.add_argument('--path', action='append', required=True, dest='spice_path', help='The spice library path.')

    parser_search = sub_parsers.add_parser('search', help='Search Kicad Symbol.')
    parser_search.add_argument('--term', type=str, required=True, help='The search term.')
    parser_search.add_argument('--path', action='append', required=True, dest='sym_path', help='The symbol library path.')

    args = parser.parse_args()

    try:
        if(args.command == 'plot'):
                plot(args.input, args.output)

        elif(args.command == 'bom'):
            make_bom(args.input, args.group, args.output, args.partlist)
        elif(args.command == 'convert'):
            convert(args.input, args.output)
        elif(args.command == 'erc'):
            make_erc(args.input, args.output)
        elif(args.command == 'drc'):
            make_drc(args.input, args.output)
        elif(args.command == 'spice'):
            make_spice(args.input, args.spice_path, args.output)
        elif(args.command == 'search'):
            search(args.term, args.sym_path)
        else:
            print(args)
    except Exception as inst:
        print(inst)
    os._exit(0)



PLOTS = []

def plots():
    return PLOTS

def reset():
    PLOTS.clear()

class Draw:
    """Draw a schematic
    :param library_path: List of strings with the location of the Kicad symbol libraries.
    """

    def __init__(self, library_path, **kwargs):
        self.el = PyDraw(library_path, **kwargs)

    def add(self, item):
        """Add an element to the Schema.
        
        :param item: The element to add.
        """
        self.el.add(item)

    def next(self, key):
        """Get next Reference for Symbol.

        When the next method is called with "R" it will for example return R1.
            
        :param key: Symbol key.
        :returns: The next reference for key.
        """ 
        return self.el.next(key)
    
    def counter(self, key, counter):
        """Set next Reference for Symbol.

        After manually added references the counter can be ajdusted.
            
        :param key: Symbol key.
        :param counter: Counter for the key.
        """ 
        self.el.counter(key, counter)

    def last(self, key):
        """Get last Reference for Symbol.

        :param key: Symbol key.
        :returns: The last reference for key.

        """
        return self.el.last(key)

    def property(self, regex, key, value):
        """Set the property for symbols.

        :param regex: Reference regex
        :param key: The property key
        :param value: The property value
        :param id: id of the property
        """
        self.el.property(regex, key, value)

    def erc(self):
        """Run the ERC check for the schema.

        :returns: List of errors, if there are some.
        """
        return self.el.erc()

    def write(self, filename):
        '''Write the schema to a Kicad schema file.

        :param filename: filename of the file to create.
        '''
        self.el.write(filename)

    def plot(self, **kwargs):
        '''Plot the schema.

        :param filename: Name of the file to create, when no file is given the plot is outputed 
        :param id: refid for SVG files.
        :param scale: Scale the image. 
        :param pages: Pages to plot, if unset all pages will be plotted.
        :param netlist: Show the netlist names.
        :param theme: Select a theme
        '''
        global PLOTS
        PLOTS = self.el.plot(**kwargs)

    def circuit(self, pathlist):
        """Get the spice circuit for the schematic
        
        :param pathlist: Location of the spice models.
        """
        return self.el.circuit(pathlist)

    def __add__(self, item):
        self.el.add(item)
        return self
    
    def pos(self, pos):
        """Set the current position

        :param pos: Position tuple.
        """
        self.el.pos(pos)
        return self

    def pop(self):
        """Get the last position from the Stack.
        """
        return self.el.pop()
    
    def peek(self):
        """Peek the last position from the Stack.
        """
        return self.el.peek()

class Pcb:

    def __init__(self, file, **kwargs):
        self.board= pcbnew.LoadBoard(file)

    def drc(self, output):
        pcbnew.WriteDRCReport(self.board, output, pcbnew.EDA_UNITS_MILLIMETRES, True)

    def gerber(self, output, plot_values=False, plot_references=True, 
               plot_invisible_text=False, via_on_mask_layer=True,
               use_gerber_attributes=False, use_gerber_x2_format=True,
               include_netlist_info=True, gerber_protel_extensions=True,
               disable_gerber_macros=False, create_gerber_jobfile=False,
               skip_plot_npth_pads=True, substract_mask_from_silk=False,
               sketch_pads_on_fab_layer=False):
        # Configure plotter
        pctl = pcbnew.PLOT_CONTROLLER(self.board)
        popt = pctl.GetPlotOptions()

        # Set some important plot options
        popt.SetPlotFrameRef(False)
        popt.SetPlotValue(plot_values)
        popt.SetPlotReference(plot_references)
        popt.SetPlotInvisibleText(plot_invisible_text)
        popt.SetPlotViaOnMaskLayer(via_on_mask_layer)  
        popt.SetAutoScale(False)
        popt.SetScale(1)
        popt.SetMirror(False)
        popt.SetUseGerberAttributes(use_gerber_attributes)
        popt.SetUseGerberX2format(use_gerber_x2_format)
        popt.SetIncludeGerberNetlistInfo(include_netlist_info)
        popt.SetUseGerberProtelExtensions(gerber_protel_extensions)
        popt.SetDisableGerberMacros(disable_gerber_macros)
        popt.SetCreateGerberJobFile(create_gerber_jobfile)
        # popt.SetExcludeEdgeLayer(False)
        popt.SetUseAuxOrigin(False)
        popt.SetSkipPlotNPTH_Pads(skip_plot_npth_pads)
        popt.SetSubtractMaskFromSilk(substract_mask_from_silk)
        popt.SetFormat(pcbnew.PLOT_FORMAT_GERBER)  
        popt.SetSketchPadsOnFabLayers(sketch_pads_on_fab_layer)
        popt.SetDrillMarksType(pcbnew.DRILL_MARKS_NO_DRILL_SHAPE)

        # Render Plot Files
        tempdir = tempfile.mkdtemp()
        popt.SetOutputDirectory(tempdir)
        
        plot_plan = [
            ( 'F.Cu', pcbnew.F_Cu, 'Front Copper' ),
            ( 'B.Cu', pcbnew.B_Cu, 'Back Copper' ),
            ( 'F.Paste', pcbnew.F_Paste, 'Front Paste' ),
            ( 'B.Paste', pcbnew.B_Paste, 'Back Paste' ),
            ( 'F.SilkS', pcbnew.F_SilkS, 'Front SilkScreen' ),
            ( 'B.SilkS', pcbnew.B_SilkS, 'Back SilkScreen' ),
            ( 'F.Mask', pcbnew.F_Mask, 'Front Mask' ),
            ( 'B.Mask', pcbnew.B_Mask, 'Back Mask' ),
            ( 'Edge.Cuts', pcbnew.Edge_Cuts, 'Edges' ),
            # ( 'Eco1.User', pcbnew.Eco1_User, 'Eco1 User' ),
            # ( 'Eco2.User', pcbnew.Eco2_User, 'Eco1 User' ),
        ]

        for layer_info in plot_plan:
            pctl.SetLayer(layer_info[1])
            pctl.OpenPlotfile(layer_info[0], pcbnew.PLOT_FORMAT_GERBER, layer_info[2])
            pctl.PlotLayer()

        # Render Drill Files
        drlwriter = pcbnew.EXCELLON_WRITER(self.board)
        drlwriter.SetMapFileFormat(pcbnew.PLOT_FORMAT_GERBER)
        drlwriter.SetFormat(True, pcbnew.EXCELLON_WRITER.DECIMAL_FORMAT, 3, 3)
        drlwriter.SetRouteModeForOvalHoles(False)
        drlwriter.CreateDrillandMapFilesSet( pctl.GetPlotDirName(), True, False );

        pctl.ClosePlot()

        # Archive files
        import zipfile
        files = os.listdir(tempdir)
        with zipfile.ZipFile(os.path.join(tempdir, "zip"), 'w', zipfile.ZIP_DEFLATED) as myzip:
            for file in files:
                myzip.write(os.path.join(tempdir, file), file)

        import shutil
        shutil.move(os.path.join(tempdir, "zip"), output)

        # Remove tempdir
        print(tempdir)
        #shutil.rmtree(tempdir)

    def plot(self, filename):
  
        plot_controller = pcbnew.PLOT_CONTROLLER(self.board)  
        plot_options = plot_controller.GetPlotOptions()  
          
        plot_options.SetOutputDirectory(filename)  
        plot_options.SetPlotFrameRef(False)  
        plot_options.SetDrillMarksType(pcbnew.PCB_PLOT_PARAMS.FULL_DRILL_SHAPE)  
        plot_options.SetSkipPlotNPTH_Pads(False)  
        plot_options.SetMirror(False)  
        plot_options.SetFormat(pcbnew.PLOT_FORMAT_SVG)  
        plot_options.SetSvgPrecision(4, False)  
        plot_options.SetPlotViaOnMaskLayer(True)  
          
        # plot_controller.OpenPlotfile("mask", pcbnew.PLOT_FORMAT_SVG, "Top mask layer")  
        plot_controller.SetColorMode(True)  
        plot_controller.SetLayer(pcbnew.F_Mask)  
        plot_controller.PlotLayer()  
        plot_controller.ClosePlot()  


if os.environ.get("MPLBACKEND") == "module://elektron":
    from io import BytesIO
    from subprocess import run

    from matplotlib import interactive, is_interactive
    from matplotlib._pylab_helpers import Gcf
    from matplotlib.backend_bases import (_Backend, FigureManagerBase)
    from matplotlib.backends.backend_agg import FigureCanvasAgg

    if sys.flags.interactive:
        interactive(True)

    class FigureManagerElektron(FigureManagerBase):

        @classmethod
        def _run(cls, *cmd):
            def f(*args, output=True, **kwargs):
                if output:
                    kwargs['capture_output'] = True
                    kwargs['text'] = True
                r = run(cmd + args, **kwargs)
                if output:
                    return r.stdout.rstrip()
            return f

        def show(self):

            # icat = __class__._run('kitty', '+kitten', 'icat')

            # if os.environ.get('MPLBACKEND_KITTY_SIZING', 'automatic') != 'manual':

            #     tput = __class__._run('tput')

            #     # gather terminal dimensions
            #     rows = int(tput('lines'))
            #     px = icat('--print-window-size')
            #     px = list(map(int, px.split('x')))

            #     # account for post-display prompt scrolling
            #     # 3 line shift for [\n, <matplotlib.axes…, >>>] after the figure
            #     px[1] -= int(3*(px[1]/rows))

            #     # resize figure to terminal size & aspect ratio
            #     dpi = self.canvas.figure.dpi
            #     self.canvas.figure.set_size_inches((px[0] / dpi, px[1] / dpi))
            
            with BytesIO() as buf:
                global PLOTS
                self.canvas.figure.savefig(buf, format='svg')
                PLOTS = [list(bytes(buf.getvalue()))]
                # icat('--align', 'left', output=False, input=buf.getbuffer())


    @_Backend.export
    class _BackendElektron(_Backend):

        FigureCanvas = FigureCanvasAgg
        FigureManager = FigureManagerElektron

        # Noop function instead of None signals that
        # this is an "interactive" backend
        mainloop = lambda: None

        # XXX: `draw_if_interactive` isn't really intended for
        # on-shot rendering. We run the risk of being called
        # on a figure that isn't completely rendered yet, so
        # we skip draw calls for figures that we detect as
        # not being fully initialized yet. Our heuristic for
        # that is the presence of axes on the figure.
        @classmethod
        def draw_if_interactive(cls):
            manager = Gcf.get_active()
            if is_interactive() and manager.canvas.figure.get_axes():
                cls.show()

        @classmethod
        def show(cls, *args, **kwargs):
            _Backend.show(*args, **kwargs)
            Gcf.destroy_all()
