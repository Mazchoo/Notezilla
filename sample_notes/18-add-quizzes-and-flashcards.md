---
phase: 6
title: Quizzes and flashes
tags: [novelty]
status: todo
---

# 18 - Quizzes and Flashcards

Add new markdown sections that convert pure html and css objects

## Steps

1. Make a ```quiz markdown attribute that converts the following into HTML that reveals a choice and explanation

```quiz
What is the capital of Peru?
  A. London
    That is not the right answer
  B. Paris
    Nope, try again
->C. Lima
    That is correct
  D. Bagdad
    Are you serious?

What is oldest tree in the world?
...
``` 

2. Make a ```flashcard markdown attribute that converts the following into HTML that can cycle through flashcards
Each flashcard has a correct answer that can be revealed

```flashcards
What is the capital of Peru?
  Lima

What is the French for waffle?
  gaufre
``` 

3. PDF export for quiz shows all answers and indicates correctness afterwards
4. PDF export for flash cards shows all flash cards in a grid

## Acceptance Criteria

- [ ] Quiz has html behavior in locked editor and export
- [ ] Flashcards have html behavior in locked editor and export
- [ ] Quiz has html behavior in PDF export
- [ ] Flashcards have html behavior in PDF export
