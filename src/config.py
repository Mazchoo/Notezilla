"""Global configuration of application"""

NOTE_FOLDER = "./notes"  # Determines where notes are loaded from
OLLAMA_PORT = 11434
MCP_PORT = 8020

# Chroma databse setup
DATABASE_FOLDER = "./chroma_db"
COLLECTION_NAME = "notes"  # Database table name
BATCH_SIZE = 100  # Limit on simulataneous upload
MAX_DB_ITERATION = 10_000_000  # Upper bound on get() pages when scanning collection ids
EMBEDDING_MODEL = "all-MiniLM-L6-v2"  # Sentence-transformer model for ChromaDB RAG
