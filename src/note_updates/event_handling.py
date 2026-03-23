"""Details of how to handle file modification events"""

from typing import Union, List, Dict

from watchdog.events import FileModifiedEvent, FileCreatedEvent, FileDeletedEvent


FileChangeEvent = Union[FileModifiedEvent, FileCreatedEvent, FileDeletedEvent]


def event_is_valid(event: FileChangeEvent):
    """Event needs translating to database update"""
    return not event.is_directory and str(event.src_path).lower().endswith(".md")


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
