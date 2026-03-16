Phase 1: Core Python Library & FastMCP API
The foundation of the project is the ability to sync the filesystem with a vector representation.

File Watcher & Parser: Implement a service to monitor the "Filesystem Notes" folder and parse Markdown frontmatter for tags and paths.

Vector Integration: Set up a local vector database ChromaDB to index the text content.

FastMCP Implementation: Build the Python API using FastMCP.

Tools: Notes, update_note, delete_note.

Resources: Semantic search endpoint and metadata-based filtering.

Schema Design: Define the JSON structure for notes to ensure consistent mapping between the .md files and the DB.

Phase 2: Local LLM Integration (Ollama)
Connecting the "brain" to the library to allow for automated note generation.

Template Engine: Create a system to store Markdown templates with placeholders for LLM injection.

Ollama Connector: Build the logic to send system prompts and context (retrieved from Phase 1) to a local Ollama instance.

Summarization Logic: Implement the "structured summary" principle to ensure LLM outputs are saved directly into the Notezilla file tree.

Phase 3: Blazor GUI Development
This is where the user interacts with the system. Using Blazor allows for a robust desktop-like experience.

Use Mud Blazor for the styling.

Layout & Navigation: Create a side-bar file tree based on the "ordered file tree path."

Markdown Editor: Integrate a side-by-side editor (e.g., using Markdig for rendering) with live preview and Mermaid diagram support.

Search Interface: Build the UI for both keyword and semantic (vector) search queries.

Generation Hub: A dedicated view to select a template, trigger Ollama, and review the generated Markdown before saving.

Phase 4: Markdown Rendering
This object of this step is to get fast markdown rendering with a Web Assembly component.

Make a Rust Webassembly Bindgen Component that accepts byte strings as input (and converts to unicode). Use an appropriate Rust library to handle rendering, mermaid diagrams and code syntax highlighting should be supported.

Include Component in Blazor App with svg and html copied to clipboard usecases.

Phase 5: Sync & Polish
Bi-directional Sync: Ensure that manual edits in the Blazor GUI trigger immediate updates in the Vector DB via the FastMCP service.

Mermaid Rendering: Ensure the GUI correctly renders diagrams to satisfy the "extending user memory" principle.

Error Handling: Robust handling for when the local LLM is offline or the vector index needs a rebuild.
