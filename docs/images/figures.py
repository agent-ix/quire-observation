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
        "refusal. A missing value is never read as false.",
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
    ("ORCHESTRATION", [("quire-verification", True), ("quire-driver", True), ("quire-integration", True)]),
    ("BACKENDS", [("quire-contract-codegen", False), ("quire-contract-runtime", False),
                  ("quire-analyze", False), ("quire-protocol", True), ("quire-mltl", False)]),
    ("LANGUAGE", [("quire-specification", True), ("quire-spec-language", False), ("quire-contract-ir", False)]),
    ("FOUNDATIONS", [("quire-exact", False), ("quire-semantic-value", False), ("quire-walk", False),
                     ("quire-canonical", False), ("quire-verification-contracts", False),
                     ("quire-observation", False)]),
]


def ecosystem(here="quire-observation"):
    """The Quire repositories in bands; each band builds on the ones below."""
    f = Figure(
        "ecosystem",
        980,
        364,
        "The Quire repositories in four bands. Foundations: quire-exact, quire-semantic-value, "
        "quire-walk, quire-canonical, quire-verification-contracts, quire-observation. Language: "
        "quire-specification, quire-spec-language, quire-contract-ir. Backends: "
        "quire-contract-codegen, quire-contract-runtime, quire-analyze, quire-protocol, "
        "quire-mltl. Orchestration: quire-verification, quire-driver, quire-integration.",
    )
    y = 0
    gap = 10
    for band, repos in REPOS:
        f.text(0, y + 12, band, "muted", "mono", SMALL)
        widths = [FONTS["title"].width(name, TITLE) + 24 for name, _ in repos]
        spare = (980 - sum(widths) - gap * (len(repos) - 1)) / len(repos)
        x = 0
        for (name, private), w in zip(repos, widths):
            w = round(w + spare)
            role = "logic" if name == here else "ink"
            f.box(x, y + 20, w, 50, role, name, "private" if private else "public", dashed=private)
            x += w + gap
        y += 88
    f.legend(0, 360, [("logic", "this repository", "box"), ("ink", "private", "dash")])
    return f


FIGURES = [hero, record, ecosystem]
