"""Details of how to handle file modification events"""

from typing import Union, List, Dict

from watchdog.events import FileModifiedEvent, FileCreatedEvent, FileDeletedEvent


FILE_CHANGE_EVENT = Union[FileModifiedEvent, FileCreatedEvent, FileDeletedEvent]


def event_is_valid(event: FILE_CHANGE_EVENT):
    """Event needs translating to database update"""
    return not event.is_directory and str(event.src_path).lower().endswith(".md")


def filter_event_list(events: List[FILE_CHANGE_EVENT]) -> List[FILE_CHANGE_EVENT]:
    """
    Collapse filesystem events into final actions:
    - created/modified -> upsert
    - deleted -> delete
    Last event per file wins.
    """
    final_events: Dict[Union[str, bytes], FILE_CHANGE_EVENT] = {}

    for event in events:
        final_events[event.src_path] = event

    return list(final_events.values())
