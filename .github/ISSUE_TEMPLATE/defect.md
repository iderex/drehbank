---
name: Defect
about: Something computes the wrong answer, refuses a valid input, or breaks
title: ''
labels: bug
assignees: ''
---

## What is wrong

One sentence. If a number is wrong, say which number and what it should be.

## The evidence

The command you ran, exactly, and what it printed. Include the input, or enough
of it to reproduce, and the version:

```
paste the command and its output here
```

If the evidence is a number, the command that produced it belongs next to it. If
you did not measure something, say that rather than estimating it.

## The machine

Operating system, core count, memory, and the toolchain version from
`rustup show active-toolchain`. A numerical result without its machine is not
reproducible, and a result that differs between platforms is a defect class of
its own here.

## What done means

The condition somebody else can check to see that this is fixed. A test that
fails today and passes afterwards is the best form of it.
