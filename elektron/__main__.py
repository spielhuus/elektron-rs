import argparse
import sys

import elektron

def main() -> int:

    parser = argparse.ArgumentParser(
        description='Nukleus electronic processing.')
    parser.add_argument('--bom', dest='action', action='append_const', const='bom',
                        help='Output the BOM as JSON')
    parser.add_argument('--plot', dest='action', action='append_const', const='plot',
                        help='Plot the Schema')
    parser.add_argument('--dump', dest='action', action='append_const', const='dump',
                        help='Dump the Schema content.')
    parser.add_argument('--netlist', dest='action', action='append_const', const='netlist',
                        help='Dump the Schema netlist.')
    parser.add_argument('--search', dest='action', action='append_const', const='search',
                        help='search in the symbol library.')
    parser.add_argument('--erc', dest='action', action='append_const', const='erc',
                        help='Run ERC test on schema.')
    parser.add_argument('--pcb', dest='action', action='append_const', const='pcb',
                        help='Plot the Board')
    parser.add_argument('--input', dest='input', required=False,
                        help='The input filename.')
    parser.add_argument('--output', dest='output',
                        help='The output filename.')
    parser.add_argument('--plotter', dest='plotter',
                        help='Select the ploter backend', default='PlotSvgWrite')
    parser.add_argument("--term", dest='term', nargs="?")

    args = parser.parse_args()

    if 'search' in args.action:
        elektron.search(args.term, ['/usr/share/kicad/symbols'])
    if 'bom' in args.action:
        elektron.get_bom(args.input, args.output, True)
    if 'plot' in args.action:
        elektron.schema_plot(args.input, args.output, True, 1)
    if 'netlist' in args.action:
        elektron.schema_netlist(args.input, args.output)

    return 0

if __name__ == '__main__':
    sys.exit(main())
