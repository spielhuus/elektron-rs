========
Notebook
========

.. toctree::
   :maxdepth: -1

   audio
   d3
   elektron
   latex

elektron offers a Python-based solution for managing electronic circuits, with the feature of creating programmatic notebooks. These notebooks enable users to integrate code, visualizations, and text seamlessly, enhancing documentation and facilitating reproducibility in electronic projects. With Elektron's programmatic notebooks, users can efficiently document their design process and analysis while promoting collaboration and iterative development.

Create a notebook

Elektron doesn't provide a built-in feature for rendering notebooks into HTML format. Markdown notebooks are converted to markdown, executing notebook commands along the way. To transform markdown into HTML, users must rely on external tools like Pandoc or Hugo. Thus, creating a notebook involves following the instructions provided by the chosen template system.

For hugo notebooks you can use the following template:

.. code-block:: markdown

   ---
   title: "My Notebook"
   ---

Execute a cell

To execute a cell you can use the following syntax:

.. |bt| raw:: html

    <code class="code docutils literal notranslate">```</code>

.. code-block:: markdown

  ```{python echo=TRUE}
  myvar = "hello"
  ```

This will execute the python code and store the result in the variable `myvar`.
The variable can be accessed with ``${myvar}``.

Global options

There are default options for the notebook cells.

- **echo** ``TRUE|FALSE`` Output the code cell.
- **results** ``hide`` Do not show the stdout outputs.

