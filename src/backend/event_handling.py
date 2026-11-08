"""Details of how to handle file modification events"""

from typing import Union, List, Dict

from watchdog.events import (
    FileModifiedEvent,
    FileCreatedEvent,
    FileDeletedEvent,
    DirDeletedEvent,
)


FileChangeEvent = Union[
    FileModifiedEvent, FileCreatedEvent, FileDeletedEvent, DirDeletedEvent
]


def event_is_valid(event: FileChangeEvent):
    """Event needs translating to database update"""
    return not event.is_directory and str(event.src_path).endswith((".md", ".MD"))


def event_is_folder_removal(event: FileChangeEvent):
    """
    Deleted path that may be a folder, so notes stored under it need removing.
    Windows reports folder deletions as file events and cannot stat a deleted
    path, so every non-markdown deletion is treated as a possible folder.
    """
    return event.event_type == "deleted" and not str(event.src_path).endswith(
        (".md", ".MD")
    )


def filter_event_list(events: List[FileChangeEvent]) -> List[FileChangeEvent]:
    """
    Collapse filesystem events into final actions:
    - created/modified -> upsert
    - deleted -> delete
    Last event per file wins.
    """
    final_events: Dict[Union[str, bytes], FileChangeEvent] = {}

    for event in events:
        final_events[event.src_path] = event

    return list(final_events.values())
