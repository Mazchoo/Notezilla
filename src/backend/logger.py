"""A global logging module that supports rich logging"""

import logging

from rich.logging import RichHandler

LOGGER = logging.getLogger("notezilla")
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(
    RichHandler(
        level=logging.INFO,
        log_time_format="[%x %X]",
    )
)
LOGGER.propagate = False
