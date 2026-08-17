"""Handle changes to note directory and forward them to database updates"""

from typing import Annotated

from fastmcp import FastMCP
from fastmcp.tools.tool import ToolResult
from pydantic import Field
from starlette.requests import Request
from starlette.responses import JSONResponse

from src.config import MCP_PORT
from src.backend.file_io import (
    create_new_folder,
    delete_note_file,
    delete_notes_folder,
    get_dirs_and_md_files,
    move_file_or_folder,
    rename_basename,
    resolve_note_path,
)
from src.backend.directory_watcher import PyFileHandler
from src.backend.parse_markdown import (
    IMarkdownFile,
    clean_path_filter,
    parse_frontmatter,
)
from src.backend.logger import LOGGER
from src.backend.mcp_interface import init_db, init_column_types, McpResponse
from src.backend.output_schema import (
    DIRECTORY_OUTPUT_SCHEMA,
    EMPTY_OUTPUT_SCHEMA,
    NOTES_OUTPUT_SCHEMA,
    UPSERT_OUTPUT_SCHEMA,
)
from src.backend.resolved_folders import ResolvedFolder

MCP = FastMCP("Notezilla")


@MCP.custom_route("/tools", methods=["GET"])
async def list_tools_endpoint(_request: Request) -> JSONResponse:
    """MCP to show all available tools"""
    tools = await MCP.list_tools()
    return JSONResponse(
        [
            {
                "name": t.name,
                "description": t.description,
                "inputSchema": t.parameters,
                "outputSchema": t.output_schema,
            }
            for t in tools
        ]
    )


@MCP.tool(output_schema=UPSERT_OUTPUT_SCHEMA)
def upsert_note(
    path: Annotated[
        str, Field(description='Relative path for the note e.g. "folder/filename.md"')
    ],
    contents: Annotated[str, Field(description="The markdown body of the note")],
    fields: Annotated[
        dict,
        Field(
            description="Dictionary of metadata fields to convert into a YAML header"
        ),
    ],
) -> ToolResult:
    """Create or update a note file with a YAML frontmatter header.

    Args:
        path: Relative path for the note e.g. "folder/filename.md"
        contents: The markdown body of the note
        fields: Dictionary of metadata fields to convert into a YAML header
    """
    warnings: list[str] = []
    result = IMarkdownFile.construct_from_data(
        path, contents, fields, ResolvedFolder.NOTES
    )
    if result:
        _, new_file_created = result
        return McpResponse.upsert(new_file_created, warnings)
    return McpResponse.upsert_error(f"Failed to upsert note at '{path}'.", warnings)


@MCP.tool(output_schema=EMPTY_OUTPUT_SCHEMA)
def delete_note(
    path: Annotated[
        str,
        Field(
            description='Relative path of the note to delete e.g. "folder/filename.md"'
        ),
    ],
) -> ToolResult:
    """Delete a note file.

    Args:
        path: Relative path of the note to delete e.g. "folder/filename.md"
    """
    warnings: list[str] = []
    resolved = resolve_note_path(path, ResolvedFolder.NOTES)
    if resolved and delete_note_file(resolved, ResolvedFolder.NOTES):
        return McpResponse.empty(warnings)
    return McpResponse.empty_error(
        f"Failed to delete note at '{path}'. Ensure the path is valid.",
        warnings,
    )


@MCP.tool(output_schema=EMPTY_OUTPUT_SCHEMA)
def new_dir(
    path: Annotated[
        str,
        Field(
            description='Relative path of the folder to create e.g. "folder" or "folder/subfolder"'
        ),
    ],
) -> ToolResult:
    """Create a new folder within the note folder.

    Creates missing parent directories. Fails if the path already exists
    or is outside the note folder.

    Args:
        path: Relative path of the folder to create e.g. "folder" or "folder/subfolder"
    """
    warnings: list[str] = []
    if create_new_folder(path, ResolvedFolder.NOTES):
        return McpResponse.empty(warnings)
    return McpResponse.empty_error(
        f"Failed to create folder at '{path}'. "
        "Ensure the path is inside the note folder and does not already exist.",
        warnings,
    )


@MCP.tool(output_schema=EMPTY_OUTPUT_SCHEMA)
def delete_folder(
    path: Annotated[
        str,
        Field(
            description='Relative path of the folder to delete e.g. "folder" or "folder/subfolder"'
        ),
    ],
) -> ToolResult:
    """Recursively delete a folder and its contents within the note folder.

    Args:
        path: Relative path of the folder to delete e.g. "folder" or "folder/subfolder"
    """
    warnings: list[str] = []
    if delete_notes_folder(path, ResolvedFolder.NOTES):
        return McpResponse.empty(warnings)
    return McpResponse.empty_error(
        f"Failed to delete folder at '{path}'. "
        "Ensure the path is a valid directory inside the note folder.",
        warnings,
    )


@MCP.tool(output_schema=EMPTY_OUTPUT_SCHEMA)
def move_dir(
    src: Annotated[
        str,
        Field(
            description='Relative path of the file or folder to move e.g. "folder" or "folder/note.md"'
        ),
    ],
    dst: Annotated[
        str,
        Field(
            description='Relative path of the destination directory e.g. "archive" or ""'
        ),
    ],
) -> ToolResult:
    """Move a file or directory into a destination folder within the note folder.

    Args:
        src: Relative path of the file or folder to move e.g. "folder" or "folder/note.md"
        dst: Relative path of the destination directory e.g. "archive" or ""
    """
    warnings: list[str] = []
    if move_file_or_folder(src, dst, ResolvedFolder.NOTES):
        return McpResponse.empty(warnings)
    return McpResponse.empty_error(
        f"Failed to move '{src}' into '{dst}'. "
        "Ensure src exists, dst is a directory inside the note folder, "
        "and dst is not inside src.",
        warnings,
    )


@MCP.tool(output_schema=EMPTY_OUTPUT_SCHEMA)
def rename_dir(
    path: Annotated[
        str,
        Field(
            description='Relative path of the file or folder to rename e.g. "folder" or "folder/note.md"'
        ),
    ],
    new_name: Annotated[
        str,
        Field(
            description=(
                'New basename for the file or folder e.g. "renamed" or "renamed.md". '
                "For files, omitting the extension keeps the source extension."
            )
        ),
    ],
) -> ToolResult:
    """Rename a file or directory within the note folder.

    When renaming a file, if new_name has no extension the source extension
    is preserved (e.g. note.md renamed to "renamed" becomes renamed.md).

    Args:
        path: Relative path of the file or folder to rename e.g. "folder" or "folder/note.md"
        new_name: New basename for the file or folder e.g. "renamed" or "renamed.md"
    """
    warnings: list[str] = []
    if rename_basename(path, new_name, ResolvedFolder.NOTES):
        return McpResponse.empty(warnings)
    return McpResponse.empty_error(
        f"Failed to rename '{path}' to '{new_name}'. "
        "Ensure the path exists, the destination does not already exist, "
        "and both stay inside the note folder.",
        warnings,
    )


@MCP.tool(output_schema=DIRECTORY_OUTPUT_SCHEMA)
def get_dir_contents(
    path: Annotated[
        str,
        Field(description='Relative path of the directory to list e.g. "folder" or ""'),
    ] = "",
) -> ToolResult:
    """List immediate child folders and file names under a directory in the note folder.

    Args:
        path: Relative path of the directory to list e.g. "folder".
    """
    warnings: list[str] = []
    folders, files, error = get_dirs_and_md_files(path, ResolvedFolder.NOTES)
    if error:
        return McpResponse.directory_error(error, folders, files, warnings)
    return McpResponse.directory(folders, files, warnings)


@MCP.tool(output_schema=NOTES_OUTPUT_SCHEMA)
def get_note(
    path: Annotated[
        str,
        Field(description='Relative path of the note e.g. "folder/filename.md"'),
    ],
) -> ToolResult:
    """Get a single note by its file path.

    Reads the markdown file on disk (source of truth after save), not the
    search index, so a save followed by open always shows the latest content.

    Args:
        path: Relative path of the note e.g. "folder/filename.md"
    """
    warnings: list[str] = []
    note = IMarkdownFile.construct_from_path(path, ResolvedFolder.NOTES)
    if note is None:
        return McpResponse.notes_error(f"Note not found at '{path}'", warnings)
    return McpResponse.notes([note], warnings)


@MCP.tool(output_schema=DIRECTORY_OUTPUT_SCHEMA)
def get_template_dir_contents(
    path: Annotated[
        str,
        Field(description='Relative path of the directory to list e.g. "folder" or ""'),
    ] = "",
) -> ToolResult:
    """List immediate child folders and file names under a directory in the template folder.

    Args:
        path: Relative path of the directory to list e.g. "folder".
    """
    warnings: list[str] = []
    folders, files, error = get_dirs_and_md_files(path, ResolvedFolder.TEMPLATES)
    if error:
        return McpResponse.directory_error(error, folders, files, warnings)
    return McpResponse.directory(folders, files, warnings)


@MCP.tool(output_schema=NOTES_OUTPUT_SCHEMA)
def get_template(
    path: Annotated[
        str,
        Field(description='Relative path of the template e.g. "animal_facts.md"'),
    ],
) -> ToolResult:
    """Get a single template by its file path.

    Args:
        path: Relative path of the template e.g. "animal_facts.md"
    """
    warnings: list[str] = []
    template = IMarkdownFile.construct_from_path(path, ResolvedFolder.TEMPLATES)
    if template is None:
        return McpResponse.notes_error(f"Template not found at '{path}'", warnings)
    return McpResponse.notes([template], warnings)


@MCP.tool(output_schema=NOTES_OUTPUT_SCHEMA)
def search_notes(
    text: Annotated[
        str,
        Field(
            description=(
                "Natural language query to search for. A blank query returns "
                "notes without a text search, applying only path and front "
                "matter filters if given."
            )
        ),
    ],
    frontmatter: Annotated[
        str,
        Field(
            description=(
                "Optional YAML front matter filter. Known columns become a Chroma "
                "metadata where clause; unknown columns are omitted and reported "
                'as warnings. List fields e.g. tags: ["cheese"] match notes that '
                "contain those values."
            )
        ),
    ] = "",
    path_filter: Annotated[
        str,
        Field(
            description=(
                "Optional comma-separated path prefix filter. Only notes whose "
                "filenames start with any listed path are returned "
                '(e.g. "2026/02, folder" or "./folder*"). Spaces around each '
                "path are trimmed. Empty segments are omitted and reported as "
                "warnings. Do not introduce by default, adds overhead "
                "to the search."
            )
        ),
    ] = "",
    n_results: Annotated[
        int, Field(description="Maximum number of results to return")
    ] = 10,
    offset: Annotated[
        int,
        Field(
            description=(
                "Pagination offset: number of matching notes to skip before "
                "returning results (e.g. offset=10, n_results=10 yields results[10:20])"
            )
        ),
    ] = 0,
) -> ToolResult:
    """Search notes by content. A blank query skips text search and returns
    notes matching the optional path and front matter filters.

    Args:
        text: Natural language query to search for; blank skips text search
        frontmatter: Optional YAML front matter used as a metadata filter
        path_filter: Optional comma-separated path prefixes; matching filenames are returned
        n_results: Maximum number of results to return
        offset: Pagination offset; skip this many matches before returning
    """
    warnings: list[str] = []
    try:
        column_types = init_column_types()
        frontmatter_filter = parse_frontmatter(frontmatter, column_types, warnings)
        cleaned_filter = clean_path_filter(path_filter, warnings)
        notes = init_db().query_by_text(
            text,
            column_types,
            n_results,
            where=frontmatter_filter,
            offset=offset,
            path_filter=cleaned_filter or None,
        )
        return McpResponse.notes(notes, warnings)
    except ValueError as e:
        return McpResponse.notes_error(f"Type error: {e}", warnings)
    except Exception as e:  # pylint: disable=broad-except
        LOGGER.exception("DB error in search_notes")
        return McpResponse.notes_error(f"DB error: {e}", warnings)


if __name__ == "__main__":
    test_observer = PyFileHandler.construct_observer(
        init_db(), init_column_types(), 200
    )
    try:
        MCP.run(transport="streamable-http", port=MCP_PORT)
    except KeyboardInterrupt:
        test_observer.stop()
    finally:
        test_observer.join()
