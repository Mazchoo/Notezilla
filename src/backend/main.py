"""Handle changes to note directory and forward them to database updates"""

from typing import Annotated

from fastmcp import FastMCP
from fastmcp.tools.tool import ToolResult
from pydantic import Field
from starlette.requests import Request
from starlette.responses import JSONResponse

from src.config import MCP_PORT, NOTE_FOLDER
from src.backend.file_io import (
    delete_note_file,
    delete_notes_folder,
    get_dirs_and_md_files,
    get_normalised_path,
)
from src.backend.directory_watcher import PyFileHandler
from src.backend.parse_markdown import IMarkdownFile
from src.backend.logger import LOGGER
from src.backend.mcp_interface import init_db, init_column_types, McpResponse
from src.backend.output_schema import (
    DIRECTORY_OUTPUT_SCHEMA,
    EMPTY_OUTPUT_SCHEMA,
    EmptyResponse,
    NOTES_OUTPUT_SCHEMA,
    UPSERT_OUTPUT_SCHEMA,
)

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
    note_path = f"{NOTE_FOLDER}/{path}"
    result = IMarkdownFile.construct_from_data(note_path, contents, fields)
    if result:
        _, new_file_created = result
        return McpResponse.upsert(new_file_created)
    return McpResponse.upsert_error(f"Failed to upsert note at '{note_path}'.")


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
    note_path = f"{NOTE_FOLDER}/{path}"
    if delete_note_file(note_path):
        return McpResponse.success(EmptyResponse())
    return McpResponse.error(
        f"Failed to delete note at '{note_path}'. Ensure the path is valid.",
        EmptyResponse(),
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
    folder_path = f"{NOTE_FOLDER}/{path}"
    if delete_notes_folder(folder_path):
        return McpResponse.success(EmptyResponse())
    return McpResponse.error(
        f"Failed to delete folder at '{folder_path}'. "
        "Ensure the path is a valid directory inside the note folder.",
        EmptyResponse(),
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
    dir_path = f"{NOTE_FOLDER}/{path}"
    folders, files, error = get_dirs_and_md_files(dir_path)
    if error:
        return McpResponse.directory_error(error, folders, files)
    return McpResponse.directory(folders, files)


@MCP.tool(output_schema=NOTES_OUTPUT_SCHEMA)
def get_note(
    path: Annotated[
        str,
        Field(description='Relative path of the note e.g. "folder/filename.md"'),
    ],
) -> ToolResult:
    """Get a single note by its file path.

    Args:
        path: Relative path of the note e.g. "folder/filename.md"
    """
    note_path = f"{NOTE_FOLDER}/{path}"
    normed_path = get_normalised_path(note_path)
    if normed_path is None:
        return McpResponse.notes_error(f"Path not recognised in note folder {path}")
    try:
        notes = init_db().query_by_id(normed_path, init_column_types())
        if not notes:
            return McpResponse.notes_error(f"Note not found at '{normed_path}'")
        return McpResponse.notes(notes)
    except ValueError as e:
        return McpResponse.notes_error(f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        LOGGER.exception("DB error in get_note")
        return McpResponse.notes_error(f"DB error: {e}")


@MCP.tool(output_schema=NOTES_OUTPUT_SCHEMA)
def search_notes_by_field(
    field: Annotated[
        str, Field(description='Metadata field name to filter on e.g. "filename"')
    ],
    value: Annotated[str, Field(description="Exact value the field must equal")],
    n_results: Annotated[
        int, Field(description="Maximum number of results to return")
    ] = 10,
) -> ToolResult:
    """Find notes where a metadata field exactly matches a value.

    Args:
        field: Metadata field name to filter on e.g. "filename"
        value: Exact value the field must equal
        n_results: Maximum number of results to return
    """
    try:
        notes = init_db().query_by_field(field, value, init_column_types(), n_results)
        return McpResponse.notes(notes)
    except ValueError as e:
        return McpResponse.notes_error(f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        LOGGER.exception("DB error in search_notes_by_field")
        return McpResponse.notes_error(f"DB error: {e}")


@MCP.tool(output_schema=NOTES_OUTPUT_SCHEMA)
def search_notes_by_tag(
    field: Annotated[str, Field(description='List metadata field name e.g. "tags"')],
    value: Annotated[str, Field(description="Value that the list must contain")],
    n_results: Annotated[
        int, Field(description="Maximum number of results to return")
    ] = 10,
) -> ToolResult:
    """Find notes where a list metadata field contains a given value.

    Args:
        field: List metadata field name e.g. "tags"
        value: Value that the list must contain
        n_results: Maximum number of results to return
    """
    try:
        notes = init_db().query_field_contains(
            field, value, init_column_types(), n_results
        )
        return McpResponse.notes(notes)
    except ValueError as e:
        return McpResponse.notes_error(f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        LOGGER.exception("DB error in search_notes_by_tag")
        return McpResponse.notes_error(f"DB error: {e}")


@MCP.tool(output_schema=NOTES_OUTPUT_SCHEMA)
def search_notes_by_text(
    text: Annotated[str, Field(description="Natural language query to search for")],
    n_results: Annotated[
        int, Field(description="Maximum number of results to return")
    ] = 10,
) -> ToolResult:
    """Semantically search notes by their content.

    Args:
        text: Natural language query to search for
        n_results: Maximum number of results to return
    """
    try:
        notes = init_db().query_by_text(text, init_column_types(), n_results)
        return McpResponse.notes(notes)
    except ValueError as e:
        return McpResponse.notes_error(f"Type error: {e}")
    except Exception as e:  # pylint: disable=broad-except
        LOGGER.exception("DB error in search_notes_by_text")
        return McpResponse.notes_error(f"DB error: {e}")


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
