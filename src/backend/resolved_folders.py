"""Resolved root folders used by path-scoped file I/O."""

from enum import Enum
from pathlib import Path

from src.config import NOTE_FOLDER, TEMPLATE_FOLDER


class ResolvedFolder(Enum):
    """Root folders that path-scoped file I/O may operate under."""

    NOTES = Path(NOTE_FOLDER).resolve()
    TEMPLATES = Path(TEMPLATE_FOLDER).resolve()
