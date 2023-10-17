.. currentmodule:: elektron

Command Line Interface
======================

Installing **elektron** installs the ``elektron`` script, a command line
interface, in your virtualenv. Executed from the terminal, this script gives
access to built-in, extension, and application-defined commands. The ``--help``
option will give more information about any commands and options.

**elektron** can output graphics to the console. You will need a terminal 
that has the graphics capabilities like `Kitty`_ or `wezterm`_

.. _Kitty: https://click.palletsprojects.com/
.. _wezterm: https://click.palletsprojects.com/


Application Discovery
---------------------

``--search``
    The given name is imported, automatically detecting an app (``app``
    or ``application``) or factory (``create_app`` or ``make_app``).

While ``--app`` supports a variety of options for specifying your
application, most use cases should be simple. Here are the typical values:

``--app hello``
    The given name is imported, automatically detecting an app (``app``
    or ``application``) or factory (``create_app`` or ``make_app``).

----

``--app`` has three parts: an optional path that sets the current working
directory, a Python file or dotted import path, and an optional variable
name of the instance or factory. If the name is a factory, it can optionally
be followed by arguments in parentheses. The following values demonstrate these
parts:

``--app src/hello``
    Sets the current working directory to ``src`` then imports ``hello``.

``--app hello.web``
    Imports the path ``hello.web``.

``--app hello:app2``
    Uses the ``app2`` Flask instance in ``hello``.

``--app 'hello:create_app("dev")'``
    The ``create_app`` factory in ``hello`` is called with the string ``'dev'``
    as the argument.

If ``--app`` is not set, the command will try to import "app" or
"wsgi" (as a ".py" file, or package) and try to detect an application
instance or factory.

Within the given import, the command looks for an application instance named
``app`` or ``application``, then any application instance. If no instance is
found, the command looks for a factory function named ``create_app`` or
``make_app`` that returns an instance.

If parentheses follow the factory name, their contents are parsed as
Python literals and passed as arguments and keyword arguments to the
function. This means that strings must still be in quotes.


