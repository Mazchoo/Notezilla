"""Definitions of interatactions between mcp and other objects"""

from typing import Any, Dict, List, Optional
from functools import cache

from typing_extensions import TypedDict

from src.note_updates.database_adapter import NoteDatabase
from src.note_updates.file_io import get_db_column_types


@cache
def init_db() -> NoteDatabase:
    """Lazily initialize the database"""
    return NoteDatabase()


@cache
def init_column_types() -> Dict[str, Any]:
    """Lazily get the column types"""
    return get_db_column_types()


class NoteQueryResult(TypedDict):
    """Defines return format of a query from mcp"""

    documents: List[str]
    metadatas: List[Dict[str, Any]]
    error: Optional[str]
