# SPDX-License-Identifier: AGPL-3.0-or-later
# Copyright (C) 2026 Agent-IX
"""quire-observation's documentation figures, rendered by the docs-figures skill.

Regenerate from the repository root with the agent-ix/dev-tools skill:

    python3 -m pip install fonttools
    python3 <dev-tools>/skills/docs-figures/scripts/render.py docs/images/figures.py

Add --check to confirm the committed SVGs are current. The fields shown in the
record figure are the members of `AdmittedRecord` in src/lib.rs.
"""

from docs_figures import FONTS, SMALL, TITLE, Figure  # noqa: F401

STYLE = "ix-docs"

# One role per kind of thing, fixed across the Quire repositories.
ROLES = [
    ("code", "authored source", "box"),
    ("logic", "compiler stage", "box"),
    ("model", "checked package / IR", "box"),
    ("evidence", "outcome", "box"),
    ("ink", "input", "box"),
]


def hero():
    """A raw record plus explicit selections go in; one of three outcomes comes out."""
    f = Figure(
        "hero",
        980,
        330,
        "A raw record arrives with the package, binding, subject, scope and limits the caller "
        "selected. admit() checks every selection. The result is a qualified observation that "
        "rules can be evaluated over, an incomplete answer naming what is missing, or a typed "
        "refusal.",
    )
    f.box(0, 10, 190, 56, "ink", "raw record", "signal · value · times")
    f.box(0, 82, 190, 56, "ink", "binding", "source · schema · unit")
    f.box(0, 154, 190, 56, "ink", "subject", "who it is about")
    f.box(0, 226, 190, 56, "ink", "scope", "population · window")
    for y in (38, 110, 182, 254):
        f.line([(190, y), (240, y)], "ink", arrow=False)
    f.line([(240, 38), (240, 254)], "ink", arrow=False)
    f.line([(240, 142), (276, 142)], "ink")
    f.box(280, 114, 170, 56, "logic", "admit", "every selection exact")
    f.line([(450, 142), (500, 142)], "logic")
    f.line([(500, 42), (500, 242)], "logic", arrow=False)
    f.line([(500, 42), (530, 42)], "model")
    f.line([(500, 142), (530, 142)], "evidence")
    f.line([(500, 242), (530, 242)], "evidence")
    f.box(534, 14, 200, 56, "model", "qualified observation", "records + every premise")
    f.box(534, 114, 200, 56, "evidence", "incomplete", "names what is missing")
    f.box(534, 214, 200, 56, "evidence", "refused", "typed cause")
    f.line([(734, 42), (790, 42)], "model")
    f.box(794, 14, 186, 56, "ink", "rules", "evaluated over it")
    f.text(534, 296, "A MISSING VALUE IS NEVER BOOLEAN FALSE", "muted", "mono", SMALL)
    f.legend(0, 324, [ROLES[1], ROLES[2], ("evidence", "no truth value", "box"), ROLES[4]])
    return f


def record():
    """What an admitted record carries, grouped by the question each field answers."""
    groups = [
        ("WHAT", "signal", "trigger · unit"),
        ("WHO", "source + subject", "typed subject"),
        ("WHEN", "position + times", "event · ingestion"),
        ("CLOCK", "clock, uncertainty", "revision · nanos"),
        ("PROVENANCE", "internal | external", "fact's origin"),
    ]
    f = Figure(
        "record",
        980,
        200,
        "An admitted record carries the signal, trigger, unit and schema; the source and typed "
        "subject; its clock position with event time and ingestion time kept separate; the clock "
        "and its uncertainty; whether it is internal or external; and a valuation that is either "
        "present with a type or explicitly missing.",
    )
    w, gap = 184, 15
    for i, (label, title, sub) in enumerate(groups):
        x = i * (w + gap)
        f.text(x, 12, label, "muted", "mono", SMALL)
        f.box(x, 22, w, 56, "ink", title, sub)
        f.line([(x + w // 2, 78), (x + w // 2, 108)], "ink", arrow=False)
    f.line([(w // 2, 108), (4 * (w + gap) + w // 2, 108)], "ink", arrow=False)
    f.line([(490, 108), (490, 124)], "ink")
    f.box(330, 128, 320, 56, "model", "admitted record", "identity derived, not trusted")
    f.box(690, 128, 290, 56, "model", "valuation", "present + type  |  missing")
    f.line([(650, 156), (687, 156)], "model", arrow=False)
    return f


REPOS = [
    (1, 0, "quire-driver", "the quire command", True),
    (2, 0, "quire-integration", "tests the whole chain", True),
    (3, 0, "quire-verification", "plans verification", True),
    (0, 1, "quire-specification", "the standard", True),
    (1, 1, "quire-spec-language", "compiler · evaluator", False),
    (2, 1, "quire-contract-ir", "contract model", False),
    (3, 1, "quire-contract-codegen", "code · tests · proofs", False),
    (4, 1, "quire-contract-runtime", "runs generated checks", False),
    (0, 2, "quire-observation", "qualified observations", False),
    (1, 2, "quire-protocol", "protocol conformance", True),
    (2, 2, "quire-analyze", "SMT consistency", False),
    (0, 3, "quire-mltl", "MLTL over observations", False),
]
FEEDS = [
    ("quire-specification", "quire-spec-language"),
    ("quire-spec-language", "quire-contract-ir"),
    ("quire-contract-ir", "quire-contract-codegen"),
    ("quire-contract-codegen", "quire-contract-runtime"),
    ("quire-spec-language", "quire-protocol"),
    ("quire-contract-ir", "quire-analyze"),
    ("quire-observation", "quire-protocol"),
    ("quire-observation", "quire-mltl"),
]
SHARED = ["quire-exact", "quire-semantic-value", "quire-walk", "quire-canonical",
          "quire-verification-contracts"]


def ecosystem(here="quire-observation"):
    """The Quire repositories: the main chain, its branches, and what runs and shares it."""
    f = Figure(
        "ecosystem",
        980,
        476,
        "The Quire repositories. The main chain runs from the standard, quire-specification, to "
        "the compiler, quire-spec-language, then quire-contract-ir, quire-contract-codegen and "
        "quire-contract-runtime. quire-spec-language also feeds quire-protocol, and "
        "quire-contract-ir feeds quire-analyze. quire-observation feeds quire-protocol and "
        "quire-mltl. Above the chain, quire-driver runs it, quire-integration tests it and "
        "quire-verification plans verification. Below it, shared libraries used throughout: "
        "quire-exact, quire-semantic-value, quire-walk, quire-canonical and "
        "quire-verification-contracts.",
    )
    w, h, step = 176, 50, 201
    rows = {0: 20, 1: 120, 2: 210, 3: 290}
    where = {}
    for col, row, name, sub, private in REPOS:
        x, y = col * step, rows[row]
        where[name] = (x, y)
        role = "logic" if name == here else "ink"
        f.box(x, y, w, h, role, name, sub, dashed=private)
    for a, b in FEEDS:
        (ax, ay), (bx, by) = where[a], where[b]
        if ay == by:
            f.line([(ax + w, ay + h / 2), (bx - 3, by + h / 2)], "model")
        else:
            f.line([(ax + w / 2, ay + h), (bx + w / 2, by - 3)], "model")
    f.text(0, 12, "ORCHESTRATION", "muted", "mono", SMALL)
    f.text(0, 112, "THE CHAIN", "muted", "mono", SMALL)
    f.text(0, 382, "SHARED LIBRARIES · USED THROUGHOUT", "muted", "mono", SMALL)
    gap = 10
    widths = [FONTS["title"].width(name, TITLE) + 24 for name in SHARED]
    spare = (980 - sum(widths) - gap * (len(SHARED) - 1)) / len(SHARED)
    x = 0
    for name, w0 in zip(SHARED, widths):
        w0 = round(w0 + spare)
        f.box(x, 390, w0, h, "logic" if name == here else "ink", name)
        x += w0 + gap
    f.legend(0, 472, [("logic", "this repository", "box"), ("ink", "private", "dash"),
                      ("model", "feeds", "fill")])
    return f


FIGURES = [hero, record, ecosystem]
