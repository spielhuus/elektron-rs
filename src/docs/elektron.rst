********
elektron
********

The notebooks elektron cell can be used to automate the handling of the Kicad files. The cell commands can be used in the front matter section of the pages. The results will replace the input cell.

To use the notebooks elektron commands, add the following code to the front matter of the page.

.. code-block::

    ---
    date: 2023-06-20
    author: "spielhuus"
    title: "hall"
    version: 1
    ```{elektron, command="bom", input=["main", "mount"], group=TRUE, partlist="../../lib/partlist.yaml"}```
    ```{elektron, command="erc", input=["main", "mount"]}```
    ```{elektron, command="drc", input=["main", "mount"]}```
    ```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```
    ```{elektron, command="pcb", input=["main", "mount", "panel"], border=TRUE}```
    ```{elektron, command="gerber", input=["main", "mount", "panel"]}```
    ---

Schema
======

Plota schematic to an image.

.. code-block:: markdown

  ```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```


**Options**:

There are default options for the notebook cells.

* **input**: the kicad project name of the schema. The schema filename must be: `{input}/{input}.kicad_sch`
* **border**: draw the border or crop the image to the content (default: false).
* **scale**: scale the image (default: 1.0).
* **pages**: list of the pages to plot (default: all).
* **theme**: the theme name [BlackWhite, Kicad2000, BlueGreenDark, BlueTone, EagleDark, Nord, SolarizedDark, SolarizedLight, WDark, WLight, BehaveDark]

**Result**:

Yaml section with the schema filename.

.. code-block:: yaml

   pcb:
     main: main_schema.svg
     mount: mount_schema.svg

PCB
===

Plot a pcb to an image.

.. code-block::

   ```{elektron, command="pcb", input=["main", "mount"], border=TRUE, theme="Mono"}```

**Options**:

* **input**: The kicad project name of the PCB. The PCB filename must be: {input}/{input}.kicad.pcb
* **border**: Draw the border or crop the image to the content.
* **theme**: The theme name [Mono, Kicad2000]

**Result**:

Yaml section with the schema filename.

.. code-block:: yaml

   schema:
     main: main_pcb.svg
     mount: mount_pcb.svg

BOM
===

write the BOM in JSON or Excel file format. The Excel file can be used for import to mouser.

.. code-block:: markdown

   ```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```

**Variables**

:: Variables: 
- input: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch
- border: draw the border or crop the image to the content.
- theme: 

ERC
===

Run the ERC checks for kicad schematic files.

.. code-block:: markdown

   ```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```

**Variables**

:: Variables: 
- input: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch

DRC
===

Run the DRC checks for kicad PCB files.

.. code-block:: markdown

   ```{elektron, command="drc", input=["main", "mount"]}```

**Variables**

* **input**: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch

gerber
======

Output the gerber files and package them into a single zip file.

.. code-block:: markdown

   ```{elektron, command="gerber", input=["main", "mount", "panel"]}```

**Variables**

* **input**: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch

