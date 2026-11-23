"""A global logging module that supports rich logging"""

import logging

from rich.logging import RichHandler

from src.config import LOG_FILE

LOGGER = logging.getLogger("notezilla")
LOGGER.setLevel(logging.INFO)
LOGGER.addHandler(
    RichHandler(
        level=logging.INFO,
        log_time_format="[%x %X]",
    )
)
if LOG_FILE:
    _file_handler = logging.FileHandler(LOG_FILE, encoding="utf-8", delay=True)
    _file_handler.setLevel(logging.INFO)
    _file_handler.setFormatter(
        logging.Formatter(
            "%(asctime)s %(levelname)-8s %(name)s: %(message)s",
            datefmt="%Y-%m-%d %H:%M:%S",
        )
    )
    LOGGER.addHandler(_file_handler)
LOGGER.propagate = False
