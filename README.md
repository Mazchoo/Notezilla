# Notezilla

## Summary

Python library that can store and track markdown files for LLM usage. The notes are stored in folders and can be tagged with metadata which let the text contents naturally map to a database.

## Core Principles

This project was generated from the following observations
* Extending LLM memory is also about extending the user memory, usage of mermaid diagrams helps alot of the user ultimately makes decisions about what to make
* Markdown is becoming an acceptable readable format that both LLM's and people can use to break down tasks
* Summarising LLM results in a structured way seems like a better way to act upon results of LLM conversations

## Notezilla GUI

A PyQT6 GUI provides:
* A way to use search queries to find notes
* Generate new notes from a local LLM using templates and edit them
* Side by side rendering and editing of markdown
