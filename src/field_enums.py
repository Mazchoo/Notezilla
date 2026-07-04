"""Database interface helper"""

from typing import Dict, List, Self, Any

from enum import StrEnum


class ReservedFields(StrEnum):
    """Indicates certain yaml fields that will be ignored"""

    PATH = "path"
    FILENAME = "filename"
    TEXT = "text"

    @classmethod
    def contains(cls, s: str) -> bool:
        """Returns true if a member of this enum"""
        return s in [e.value for e in cls]

    @classmethod
    def excluded_from_metadata(cls) -> frozenset[str]:
        """
        Row keys that are not stored in Chroma metadata.
        Path is primary key and text is document content.
        """
        return frozenset({cls.PATH, cls.TEXT})


class FieldTypes(StrEnum):
    """Indicates all allowed types for front matter fields"""

    NULL = "null"
    BOOL = "bool"
    INT = "int"
    FLOAT = "float"
    DATE = "date"
    LIST = "list"
    STRING = "str"
    JSON = "json"

    @property
    def hierarchy(self) -> List[Self]:
        """
        Fields ordered by how permissive they are,
        each instance is more general case of the next
        """
        return [
            FieldTypes.NULL,
            FieldTypes.DATE,
            FieldTypes.BOOL,
            FieldTypes.INT,
            FieldTypes.FLOAT,
            FieldTypes.LIST,
            FieldTypes.STRING,
            FieldTypes.JSON,
        ]  # pyright: ignore

    def get_priority(self) -> int:
        """Fields with a higher priority are more permissive and subsume other types"""
        return self.hierarchy.index(self)

    def get_default(self) -> Any:  # pylint: disable=too-many-return-statements
        """Return an example value for a give field type"""
        if self == FieldTypes.NULL:
            return None
        if self == FieldTypes.BOOL:
            return False
        if self == FieldTypes.INT:
            return 0
        if self == FieldTypes.FLOAT:
            return 0.0
        if self == FieldTypes.DATE:
            return "2000-01-01"
        if self == FieldTypes.LIST:
            return []
        if self == FieldTypes.STRING:
            return ""
        if self == FieldTypes.JSON:
            return {}
        return None


ColumnTypes = Dict[str, FieldTypes]
