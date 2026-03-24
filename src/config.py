"""Global configuration of application"""

NOTE_FOLDER = "./notes"  # Determines where notes are loaded from
OLLAMA_PORT = 11434

# Chroma databse setup
DATABASE_FOLDER = "./chroma_db"
COLLECTION_NAME = "notes"  # Database table name
BATCH_SIZE = 100  # Upload batch size
