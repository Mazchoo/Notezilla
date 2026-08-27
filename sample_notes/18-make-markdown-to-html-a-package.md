---
phase: 6
title: Separate out markdown to html as a package
tags: [polish]
status: todo
---

# 18 - MCP toolcall experimentation

Make a cargo crate that replicates the behavior to convert a markdown to html.

## Steps

1. Make a PR for the radical in the html to pdf crate 
2. Publish the package with dependencies copied over
3. Copy over all functionality from src/frontend
4. Extend tests with regression tests
5. Add configuration class make conversion configurable

## Acceptance Criteria

- [ ] Cargo crate is published with tests to render all the different parts
- [ ] Pdf rendering trait is also included in the crate
- [ ] Configuration class is setup to change the colors and format of output files
