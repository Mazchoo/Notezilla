"""Definitions of interatactions between mcp and other objects"""

from typing import Any, Dict, List, Optional
from functools import cache

from typing_extensions import TypedDict

from src.backend.database_adapter import NoteDatabase
from src.backend.file_io import get_db_column_types
from src.field_enums import ColumnTypes


@cache
def init_db() -> NoteDatabase:
    """Lazily initialize the database"""
    return NoteDatabase()


@cache
def init_column_types() -> ColumnTypes:
    """Lazily get the column types"""
    return get_db_column_types()


class NoteQueryResult(TypedDict):
    """Defines return format of a query from mcp"""

    documents: List[str]
    metadatas: List[Dict[str, Any]]
    error: Optional[str]


class DirectoryContentsResult(TypedDict):
    """Defines content of directroy contents query"""

    folders: List[str]
    files: List[str]
    error: Optional[str]
