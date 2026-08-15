"""Tests for MCP response wiring (isError, structured content)."""

import pytest
from mcp.types import CallToolResult

from src.backend.mcp_interface import McpResponse
from src.backend.output_schema import EmptyResponse


class TestMcpResponseIsError:
    """Error results must set CallToolResult.isError; success must not."""

    def test_error_sets_is_error_true(self):
        result = McpResponse.error("boom", EmptyResponse(warnings=[]))
        wire = result.to_mcp_result()

        assert isinstance(wire, CallToolResult)
        assert wire.isError is True
        assert wire.content[0].text == "Error: boom"

    def test_success_does_not_set_is_error(self):
        result = McpResponse.success(EmptyResponse(warnings=[]))
        wire = result.to_mcp_result()

        # Success returns content + structured_content tuple, not CallToolResult
        assert not isinstance(wire, CallToolResult) or wire.isError is False
        assert result.content[0].text == "Success"
        assert result.structured_content == {"warnings": []}


class TestMcpResponseWarnings:
    """Typed helpers include the warnings list in structured content."""

    def test_empty_success_includes_warnings(self):
        result = McpResponse.empty(["path normalised"])

        assert result.structured_content == {"warnings": ["path normalised"]}

    def test_empty_error_includes_warnings(self):
        result = McpResponse.empty_error("boom", [])

        assert result.structured_content == {"warnings": []}


if __name__ == "__main__":
    pytest.main([__file__, "-x", "--verbose"])
