# Term Prep Plus
This is a terminal-based study program where the user can study for a given exam in JSON format.

## Assets
The subdirectory `assets` is the location where the JSON files should be stored for studying.
This program will attempt to create an assets directory if none exists prior to its initial
execution.

## Format of JSON Study Files
There are two `structs` that are used in the program:
* `Exam` - the high-level exam that the user is studying for.
* `Question` - represents each question that is present in the exam. An `Exam` comprises many
   questions.

The `structs` mentioned above establish the contract that the JSON format must encompass in order
to function.  Below is the formatting that must be observed in the JSON file to study using Term
Prep Plus:

### `Exam`
The following is the general format that the high-level `Exam` must use; it's essentially the
name of the exam (for example, `"CompTIA Network+ (N10-008)"`) and a list of `question`s:
```json
{
  "name": "Exam_Name",
  "questions": [
    ...
  ]
}
```

### `Question`
Each `Exam` consists of multiple questions, where each `question` has a `prompt` (the question
to be answered); the 4 `choices` or options for multiple-choice answers; the `answer` (the correct
choice from `choices` - this should be formatted exactly the same as the one in `choices`); the
`refs` or list of references so the user can refer to those if they get it incorrect.
```json
{
  "prompt": "What is the capital of France?",
  "choices": ["Berlin", "Paris", "London", "France"],
  "answer": "Paris",
  "refs": ["https://en.wikipedia.org/wiki/Paris"]
}
```

### Example JSON file
```json
{
  "name": "CompTIA Network+ (N10-008)",
  "questions": [
    {
      "prompt": "What is Layer 1 of the OSI Model?",
      "choices": [
        "Data Link",
        "Network",
        "Session",
        "Physical"
      ],
      "answer": "Physical",
      "explanation": "The 1st layer of the OSI model is the Physical layer",
      "refs": [
        "The Official CompTIA Network+ Student Guide (Exam N10-008). Pg 4-8.",
        "CompTIA Network+ Study Guide: Exam N10-008, 5th Edition. Pg 34-51."
      ]
    },
    {
      "prompt": "What is Layer 2 of the OSI Model?",
      "choices": [
        "Application",
        "Data Link",
        "Network",
        "Transport"
      ],
      "answer": "Data Link",
      "explanation": "The 2nd layer of the OSI model is the Data Link layer",
      "refs": [
        "The Official CompTIA Network+ Student Guide (Exam N10-008). Pg 4-8.",
        "CompTIA Network+ Study Guide: Exam N10-008, 5th Edition. Pg 34-51."
      ]
    }
  ]
}
```