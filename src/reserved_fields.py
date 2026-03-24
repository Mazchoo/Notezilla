"""Database helper"""

from enum import Enum


class ReservedFields(Enum):
    """Indicates certain yaml fields that will be ignored"""

    PATH = "path"
    FILENAME = "filename"
    TEXT = "text"

    @classmethod
    def contains(cls, s: str) -> bool:
        """Returns true if a member of this enum"""
        return s in [e.value for e in cls]
