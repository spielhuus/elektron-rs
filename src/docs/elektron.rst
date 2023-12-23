elektron commands
=================

.. toctree::
   :maxdepth: 4

The notebooks elektron command can be used to automate the handling of the Kicad results. The commands must be used in the front matter section of the pages. The results will relace the commannds.

.. code-block:: yaml
   :force:

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


-----------
plot schema
-----------

.. code-block:: md

  ```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```

Variables
---------

* **input**: the kicad project name of the schema. The schema filename must be: `{input}/{input}.kicad_sch`
* **border**: draw the border or crop the image to the content (default: false).
* **scale**: scale the image (default: 1.0).
* **pages**: list of the pages to plot (default: all).
* **theme**: the theme name [BlackWhite, Kicad2000, BlueGreenDark, BlueTone, EagleDark, Nord, SolarizedDark, SolarizedLight, WDark, WLight, BehaveDark]

--------
plot pcb
--------

```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```

Variables
---------

* input: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch
* border: draw the border or crop the image to the content.
* theme: the theme name [Mono, Kicad2000]

----------
create bom
----------

```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```

:: Variables: 
- input: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch
- border: draw the border or crop the image to the content.
- theme: 

Themes:
- Mono
- Kicad-2000
- .....


----------
erc checks
----------

```{elektron, command="schema", input=["main", "mount"], border=TRUE, theme="Mono"}```

:: Variables: 
- input: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch
- border: draw the border or crop the image to the content.
- theme: 

Themes:
- Mono
- Kicad-2000
- .....


----------
drc checks
----------

.. code-block:: md

   ```{elektron, command="drc", input=["main", "mount"]}```

Variables: 
``````````

* **input**: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch


------------
gerber files
------------

Output the gerber files and package them into a single zip file.

.. code-block:: md

   ```{elektron, command="gerber", input=["main", "mount", "panel"]}```

Variables: 
``````````

* **input**: the kicad project name of the schema. The schema filename must be: {input}/{input}.kicad.sch
* .....

