"""Encode and decode front matter for Chroma metadata storage."""

import json
from datetime import datetime, date
from typing import Any, Dict, List

from src.field_enums import ColumnTypes, ReservedFields, FieldTypes
from src.backend.note import NoteData


def cast_value(key: str, val, target_type: FieldTypes) -> dict:  # pylint: disable=too-many-return-statements
    """Cast a value to the target type and return as dict entries for a row"""
    if val is None:
        return {key: None}
    if target_type == FieldTypes.JSON:
        return {key: json.dumps(val, default=str)}
    if target_type == FieldTypes.LIST:
        parsed_list = val if isinstance(val, list) else [val]
        if not parsed_list:
            return {f"{key}\t": False}
        return {f"{key}\t{item}": True for item in parsed_list}
    if target_type == FieldTypes.DATE:
        return {key: val.isoformat() if isinstance(val, (datetime, date)) else str(val)}
    if target_type == FieldTypes.STRING:
        return {key: str(val)}
    if target_type == FieldTypes.FLOAT:
        return {key: float(val)}
    if target_type == FieldTypes.INT:
        return {key: int(val)}
    if target_type == FieldTypes.BOOL:
        return {key: bool(val)}
    return {key: val}


def decode_frontmatter(
    metadata: Dict[str, Any], column_types: ColumnTypes
) -> Dict[str, Any]:
    """Turn stored Chroma metadata keys back into a front matter dict."""
    fields: Dict[str, Any] = {}
    list_items: Dict[str, list[str]] = {}

    for key, val in metadata.items():
        if key.startswith("\n") or ReservedFields.contains(key):
            continue
        if "\t" in key:
            field, item = key.split("\t", 1)
            if val is True:
                list_items.setdefault(field, []).append(item)
            elif val is False:
                list_items.setdefault(field, [])
            continue

        field_type = column_types.get(key)
        if field_type in (FieldTypes.JSON, FieldTypes.JSON.value):
            fields[key] = json.loads(val) if isinstance(val, str) else val
        else:
            fields[key] = val

    for field, items in list_items.items():
        fields[field] = sorted(items)

    return fields


def notes_from_chroma(
    documents: List[str],
    metadatas: List[Dict[str, Any]],
    column_types: ColumnTypes,
) -> List[NoteData]:
    """Convert Chroma documents and metadata into NoteData records."""
    notes: List[NoteData] = []
    for text, metadata in zip(documents, metadatas):
        notes.append(
            NoteData(
                text=text,
                fields=decode_frontmatter(metadata, column_types),
                filename=str(metadata.get(ReservedFields.FILENAME, "")),
            )
        )
    return notes
