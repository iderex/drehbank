Which issue this closes, or which one it is part of.

## What changed

What the change does and what failure it prevents. Where it corrects something,
what was wrong and how it was found.

## The means, and why it fits

The language, the format and the tool this artefact is made of, and one sentence
on why they fit. Say it even when the answer is the obvious one, because a means
carried over from habit is an assumption about this change rather than a
decision about it.

## The evidence

Commands and their output, run at the commit being pushed. A claim you cannot
back with a command is written as a claim.

```
paste the commands and their output here
```

## Questions this change has to have answered

Answer each one. "No" is an answer; leaving it out is not.

- Does it change a numerical result on an unchanged input? If so, on which case,
  by how much, and why is the new answer the right one? This is a breaking
  change even when nothing about the types moves.
- Does it change a convention or the meaning of a field in the file format?
  Same rule.
- Does it add a guard? Then show the guard refusing what it names, and show the
  suite going red when the guard is removed.
- Does it add a numerical path? Then name the test that could tell it from a
  wrong one.
- Does it depend on a decision in `docs/decisions/`? Name the file. If it
  contradicts one, the decision moves first, in its own change.
- Does it invalidate anything in the documentation, the conventions page most of
  all? A code change can make that page wrong without touching it.
- Does it add a dependency? Name it, say why it is there and what would have to
  be written by hand to remove it.

## What this does not do

The part somebody would otherwise assume. What is untested, unmeasured, or left
for another issue. A negative statement here is worth more than a positive one
elsewhere, and it does not get quietly upgraded later.
