"""Python-facing blocks entrypoint.

This module is the stable import surface for block composition APIs.
PyO3-backed implementations live under bindings/python.
"""

from . import block

__all__ = ["block"]
