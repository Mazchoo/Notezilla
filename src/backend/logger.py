"""A global logging module that supports rich logging"""

import logging

from rich.logging import RichHandler


logging.basicConfig(
    handlers=[RichHandler()],
    level=logging.INFO,
    format="%(message)s",
    datefmt="[%x %X]",
)
LOGGER = logging.getLogger("notezilla")
