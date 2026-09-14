"""Global configuration of application"""

NOTE_FOLDER = "./notes"  # Determines where notes are loaded from
TEMPLATE_FOLDER = "./response_templates"  # Determines where templates are loaded from
MCP_PORT = 8020

# If no file-change ping arrives within this many seconds, write an SSE
# comment on GET /events so proxies and browsers do not close the idle stream.
FILE_CHANGE_SSE_IDLE_TIMEOUT_SECONDS = 15.0

# Chroma databse setup
DATABASE_FOLDER = "./chroma_db"
COLLECTION_NAME = "notes"  # Database table name
BATCH_SIZE = 100  # Limit on simulataneous upload
MAX_DB_BATCH_ITERATION = 10_000_000  # Upper bound on scanning database batches
EMBEDDING_MODEL = "all-MiniLM-L6-v2"  # Sentence-transformer model for ChromaDB RAG
