import json
import sys
import os

template_file = 'src/css/css.template'
path = 'src/css/{theme}.json'
outfile = 'src/css/{theme}.css'
themes = [
        'behave-dark', 
        'black-white', 
        'blue-green-dark', 
        'blue-tone', 
        'eagle-dark',
        'kicad_2020', 
        'nord', 
        'solarized-dark', 
        'solarized-light', 
        'wdark', 
        'wlight', 
]

def build():
    for schema in themes:
        with open(outfile.format(theme=schema), 'w') as out:

            with open(str(path.format(theme=schema))) as file:
                colors = json.load(file)
                if "schematic" in colors:
                    with open(template_file) as tfile:
                        template = tfile.read()
                        result = template.format(**colors)
                        out.write(result)

def clean():
    for schema in themes:
        os.remove(str(outfile.format(theme=schema)))

def main():
    if len(sys.argv) == 1:
        print("call with arguments build or clean")
    elif sys.argv[1] == "build":
        build()
    elif sys.argv[1] == "clean":
        clean()
    else:
        print("call with arguments build or clean")

if __name__ == "__main__":
    main()
